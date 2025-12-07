#!/usr/bin/env python3
"""
OpenF5-TTS REST API Server

Provides HTTP endpoints for voice cloning and speech synthesis using F5-TTS.
Uses Apache 2.0 licensed OpenF5 weights (trained on Emilia-YODAS dataset).

Designed for use with open-tts-rs CLI.

Copyright (c) 2025 Michael A Wright
MIT License
"""

import os
import io
import json
import base64
import tempfile
import logging
import hashlib
from pathlib import Path

from flask import Flask, request, jsonify, send_file
from flask_cors import CORS
import torch
import numpy as np
import soundfile as sf

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('openf5-server')

app = Flask(__name__)
CORS(app)

# Global model instance
f5_model = None
device = None

# Voice storage (stores reference audio + transcript)
VOICE_DIR = Path('/app/voices')
VOICE_DIR.mkdir(exist_ok=True)

# Sample rate for F5-TTS
SAMPLE_RATE = 24000


def get_device():
    """Detect and return the best available device."""
    if torch.cuda.is_available():
        device_name = torch.cuda.get_device_name(0)
        logger.info(f"Using CUDA device: {device_name}")
        return 'cuda:0'
    else:
        logger.warning("CUDA not available, using CPU")
        return 'cpu'


def load_models():
    """Load F5-TTS model."""
    global f5_model, device

    device = get_device()
    logger.info(f"Loading F5-TTS model on device: {device}")

    try:
        # Import F5-TTS
        import sys
        sys.path.insert(0, '/app/f5-tts/src')

        from f5_tts.api import F5TTS

        # Load model with OpenF5 weights
        f5_model = F5TTS(
            model_type="F5-TTS",
            ckpt_file="/app/models/F5TTS_Base/model_1200000.safetensors",
            vocab_file="/app/models/F5TTS_Base/vocab.txt",
            device=device
        )

        logger.info("F5-TTS model loaded successfully")

    except Exception as e:
        logger.error(f"Failed to load F5-TTS model: {e}")
        raise


@app.route('/health', methods=['GET'])
def health():
    """Health check endpoint."""
    cuda_available = torch.cuda.is_available()
    gpu_name = torch.cuda.get_device_name(0) if cuda_available else None

    return jsonify({
        'status': 'healthy',
        'model': 'openf5_tts',
        'license': 'Apache 2.0',
        'cuda_available': cuda_available,
        'gpu': gpu_name,
        'device': str(device)
    })


@app.route('/info', methods=['GET'])
def info():
    """Return model information."""
    return jsonify({
        'model': 'OpenF5-TTS',
        'license': 'Apache 2.0',
        'weights': 'OpenF5 (Emilia-YODAS trained)',
        'capabilities': ['voice_cloning', 'tts', 'emotion_preservation'],
        'supported_languages': ['EN', 'ZH'],
        'sample_rate': SAMPLE_RATE,
        'note': 'Uses flow-matching for high-quality voice cloning'
    })


@app.route('/extract_voice', methods=['POST'])
def extract_voice():
    """
    Store reference audio for voice cloning.

    F5-TTS uses reference audio directly during synthesis (no embedding extraction).
    This endpoint stores the audio and transcript for later use.

    Expects multipart form data:
    - audio: WAV file (3-30 seconds recommended)
    - transcript: Text transcript of the audio
    - name: (optional) Name to save the voice as
    """
    try:
        if 'audio' not in request.files:
            return jsonify({'error': 'No audio file provided'}), 400

        audio_file = request.files['audio']
        transcript = request.form.get('transcript', '')
        voice_name = request.form.get('name')

        if not transcript:
            return jsonify({'error': 'Transcript is required'}), 400

        # Read audio data
        audio_data, sample_rate = sf.read(audio_file)

        # Resample if needed
        if sample_rate != SAMPLE_RATE:
            import librosa
            audio_data = librosa.resample(
                audio_data,
                orig_sr=sample_rate,
                target_sr=SAMPLE_RATE
            )
            sample_rate = SAMPLE_RATE

        # Convert to mono if stereo
        if len(audio_data.shape) > 1:
            audio_data = np.mean(audio_data, axis=1)

        # Encode audio as base64
        buffer = io.BytesIO()
        sf.write(buffer, audio_data, SAMPLE_RATE, format='WAV')
        audio_b64 = base64.b64encode(buffer.getvalue()).decode('utf-8')

        # Create voice ID from content hash
        voice_id = hashlib.md5(
            (audio_b64 + transcript).encode()
        ).hexdigest()[:16]

        result = {
            'success': True,
            'voice_id': voice_id,
            'duration': len(audio_data) / SAMPLE_RATE,
            'transcript': transcript,
            'note': 'F5-TTS uses reference audio directly (no embedding)'
        }

        # Save if name provided
        if voice_name:
            voice_path = VOICE_DIR / f"{voice_name}.json"
            voice_data = {
                'audio_b64': audio_b64,
                'transcript': transcript,
                'sample_rate': SAMPLE_RATE,
                'duration': len(audio_data) / SAMPLE_RATE,
                'model': 'openf5_tts'
            }
            with open(voice_path, 'w') as f:
                json.dump(voice_data, f)

            # Also save raw audio for direct use
            audio_path = VOICE_DIR / f"{voice_name}.wav"
            sf.write(str(audio_path), audio_data, SAMPLE_RATE)

            result['saved_as'] = voice_name
            logger.info(f"Voice saved as: {voice_name}")

        # Return audio for temporary use
        result['audio'] = audio_b64

        return jsonify(result)

    except Exception as e:
        logger.error(f"Voice extraction failed: {e}")
        return jsonify({'error': str(e)}), 500


@app.route('/synthesize', methods=['POST'])
def synthesize():
    """
    Synthesize speech using F5-TTS voice cloning.

    Expects JSON:
    - text: Text to synthesize
    - name: Name of a saved voice
    - OR audio: Base64 encoded reference audio
    - OR audio + transcript: Reference audio and its transcript
    - speed: (optional) Speech speed (default: 1.0)
    """
    try:
        data = request.get_json()

        if not data:
            return jsonify({'error': 'JSON body required'}), 400

        text = data.get('text')
        if not text:
            return jsonify({'error': 'Text is required'}), 400

        speed = data.get('speed', 1.0)

        # Get reference audio and transcript
        ref_audio_path = None
        ref_text = None
        temp_ref_file = None

        try:
            if 'name' in data:
                # Load saved voice
                voice_name = data['name']

                # Check for WAV file first
                wav_path = VOICE_DIR / f"{voice_name}.wav"
                json_path = VOICE_DIR / f"{voice_name}.json"

                if wav_path.exists():
                    ref_audio_path = str(wav_path)
                    with open(json_path) as f:
                        voice_data = json.load(f)
                    ref_text = voice_data['transcript']
                else:
                    return jsonify({'error': f"Voice '{voice_name}' not found"}), 404

            elif 'audio' in data:
                # Use provided audio
                audio_b64 = data['audio']
                ref_text = data.get('transcript', '')

                if not ref_text:
                    return jsonify({'error': 'Transcript required with audio'}), 400

                # Decode and save to temp file
                audio_bytes = base64.b64decode(audio_b64)
                temp_ref_file = tempfile.NamedTemporaryFile(
                    suffix='.wav', delete=False
                )
                temp_ref_file.write(audio_bytes)
                temp_ref_file.close()
                ref_audio_path = temp_ref_file.name

            else:
                return jsonify({'error': 'Either name or audio is required'}), 400

            # Generate with F5-TTS
            logger.info(f"Synthesizing: '{text[:50]}...' with voice reference")

            # Create temp output file
            with tempfile.NamedTemporaryFile(
                suffix='.wav', delete=False
            ) as tmp_out:
                output_path = tmp_out.name

            # Run synthesis
            audio_output, sr, _ = f5_model.infer(
                ref_file=ref_audio_path,
                ref_text=ref_text,
                gen_text=text,
                speed=speed
            )

            # Save output
            sf.write(output_path, audio_output, sr)

            # Read and return
            with open(output_path, 'rb') as f:
                audio_bytes = f.read()

            os.unlink(output_path)

            buffer = io.BytesIO(audio_bytes)
            return send_file(
                buffer,
                mimetype='audio/wav',
                as_attachment=True,
                download_name='output.wav'
            )

        finally:
            # Clean up temp reference file
            if temp_ref_file and os.path.exists(temp_ref_file.name):
                os.unlink(temp_ref_file.name)

    except Exception as e:
        logger.error(f"Synthesis failed: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({'error': str(e)}), 500


@app.route('/voices', methods=['GET'])
def list_voices():
    """List all saved voices."""
    voices = []
    for voice_file in VOICE_DIR.glob('*.json'):
        try:
            with open(voice_file) as f:
                data = json.load(f)
            voices.append({
                'name': voice_file.stem,
                'transcript': data.get('transcript', ''),
                'duration': data.get('duration', 0),
                'model': data.get('model', 'unknown')
            })
        except Exception as e:
            logger.warning(f"Could not read voice file {voice_file}: {e}")

    return jsonify({'voices': voices})


@app.route('/voices/<name>', methods=['DELETE'])
def delete_voice(name):
    """Delete a saved voice."""
    json_path = VOICE_DIR / f"{name}.json"
    wav_path = VOICE_DIR / f"{name}.wav"

    if not json_path.exists():
        return jsonify({'error': f"Voice '{name}' not found"}), 404

    json_path.unlink()
    if wav_path.exists():
        wav_path.unlink()

    return jsonify({'success': True, 'deleted': name})


if __name__ == '__main__':
    logger.info("Starting OpenF5-TTS server...")
    logger.info(f"PyTorch version: {torch.__version__}")
    logger.info(f"CUDA available: {torch.cuda.is_available()}")

    if torch.cuda.is_available():
        logger.info(f"CUDA version: {torch.version.cuda}")
        logger.info(f"GPU: {torch.cuda.get_device_name(0)}")

    load_models()

    logger.info("Server starting on port 9288...")
    app.run(host='0.0.0.0', port=9288, threaded=True)
