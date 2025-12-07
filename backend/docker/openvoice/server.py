#!/usr/bin/env python3
"""
OpenVoice V2 REST API Server

Provides HTTP endpoints for voice cloning and speech synthesis.
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
logger = logging.getLogger('openvoice-server')

app = Flask(__name__)
CORS(app)

# Global model instances
tone_color_converter = None
tts_model = None
device = None

# Voice storage
VOICE_DIR = Path('/app/voices')
VOICE_DIR.mkdir(exist_ok=True)


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
    """Load OpenVoice and MeloTTS models."""
    global tone_color_converter, tts_model, device

    device = get_device()
    logger.info(f"Loading models on device: {device}")

    try:
        from openvoice.api import ToneColorConverter
        from openvoice import se_extractor
        from melo.api import TTS

        # Load tone color converter
        ckpt_converter = 'checkpoints_v2/converter'
        tone_color_converter = ToneColorConverter(
            f'/app/openvoice/{ckpt_converter}/config.json',
            device=device
        )
        tone_color_converter.load_ckpt(f'/app/openvoice/{ckpt_converter}/checkpoint.pth')
        logger.info("Tone color converter loaded")

        # Load MeloTTS for base synthesis
        tts_model = TTS(language='EN', device=device)
        logger.info("MeloTTS loaded")

        logger.info("All models loaded successfully")

    except Exception as e:
        logger.error(f"Failed to load models: {e}")
        raise


@app.route('/health', methods=['GET'])
def health():
    """Health check endpoint."""
    cuda_available = torch.cuda.is_available()
    gpu_name = torch.cuda.get_device_name(0) if cuda_available else None

    return jsonify({
        'status': 'healthy',
        'model': 'openvoice_v2',
        'cuda_available': cuda_available,
        'gpu': gpu_name,
        'device': str(device)
    })


@app.route('/info', methods=['GET'])
def info():
    """Return model information."""
    return jsonify({
        'model': 'OpenVoice V2',
        'license': 'MIT',
        'capabilities': ['voice_cloning', 'tts'],
        'supported_languages': ['EN', 'ZH', 'JP', 'KR'],
        'sample_rate': 24000
    })


@app.route('/extract_voice', methods=['POST'])
def extract_voice():
    """
    Extract voice embedding from reference audio.

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

        # Save audio to temp file
        with tempfile.NamedTemporaryFile(suffix='.wav', delete=False) as tmp:
            audio_file.save(tmp.name)
            tmp_path = tmp.name

        try:
            from openvoice import se_extractor

            # Extract speaker embedding
            target_se, audio_name = se_extractor.get_se(
                tmp_path,
                tone_color_converter,
                vad=True
            )

            # Convert to serializable format
            embedding_data = target_se.cpu().numpy().tobytes()
            embedding_b64 = base64.b64encode(embedding_data).decode('utf-8')

            result = {
                'success': True,
                'embedding': embedding_b64,
                'embedding_shape': list(target_se.shape),
                'transcript': transcript
            }

            # Save if name provided
            if voice_name:
                voice_path = VOICE_DIR / f"{voice_name}.json"
                voice_data = {
                    'embedding': embedding_b64,
                    'shape': list(target_se.shape),
                    'transcript': transcript,
                    'model': 'openvoice_v2'
                }
                with open(voice_path, 'w') as f:
                    json.dump(voice_data, f)
                result['saved_as'] = voice_name
                logger.info(f"Voice saved as: {voice_name}")

            return jsonify(result)

        finally:
            os.unlink(tmp_path)

    except Exception as e:
        logger.error(f"Voice extraction failed: {e}")
        return jsonify({'error': str(e)}), 500


@app.route('/synthesize', methods=['POST'])
def synthesize():
    """
    Synthesize speech using extracted voice.

    Expects JSON:
    - text: Text to synthesize
    - embedding: Base64 encoded voice embedding (from extract_voice)
    - OR name: Name of a saved voice
    - language: (optional) Language code (default: EN)
    - speed: (optional) Speech speed (default: 1.0)
    """
    try:
        data = request.get_json()

        if not data:
            return jsonify({'error': 'JSON body required'}), 400

        text = data.get('text')
        if not text:
            return jsonify({'error': 'Text is required'}), 400

        language = data.get('language', 'EN')
        speed = data.get('speed', 1.0)

        # Get voice embedding
        if 'name' in data:
            # Load saved voice
            voice_path = VOICE_DIR / f"{data['name']}.json"
            if not voice_path.exists():
                return jsonify({'error': f"Voice '{data['name']}' not found"}), 404

            with open(voice_path) as f:
                voice_data = json.load(f)

            embedding_b64 = voice_data['embedding']
            shape = voice_data['shape']

        elif 'embedding' in data:
            embedding_b64 = data['embedding']
            shape = data.get('shape', [1, 256])

        else:
            return jsonify({'error': 'Either embedding or name is required'}), 400

        # Decode embedding
        embedding_bytes = base64.b64decode(embedding_b64)
        target_se = torch.from_numpy(
            np.frombuffer(embedding_bytes, dtype=np.float32).reshape(shape)
        ).to(device)

        # Generate base audio with MeloTTS
        speaker_ids = tts_model.hps.data.spk2id
        speaker_id = list(speaker_ids.values())[0]  # Use first speaker

        with tempfile.NamedTemporaryFile(suffix='.wav', delete=False) as tmp_base:
            tts_model.tts_to_file(
                text,
                speaker_id,
                tmp_base.name,
                speed=speed
            )
            base_path = tmp_base.name

        try:
            # Get source speaker embedding from base audio
            from openvoice import se_extractor
            source_se, _ = se_extractor.get_se(
                base_path,
                tone_color_converter,
                vad=False
            )

            # Apply tone color conversion
            with tempfile.NamedTemporaryFile(suffix='.wav', delete=False) as tmp_out:
                tone_color_converter.convert(
                    audio_src_path=base_path,
                    src_se=source_se,
                    tgt_se=target_se,
                    output_path=tmp_out.name
                )
                output_path = tmp_out.name

            try:
                # Read output audio
                audio_data, sample_rate = sf.read(output_path)

                # Convert to bytes
                buffer = io.BytesIO()
                sf.write(buffer, audio_data, sample_rate, format='WAV')
                buffer.seek(0)

                return send_file(
                    buffer,
                    mimetype='audio/wav',
                    as_attachment=True,
                    download_name='output.wav'
                )

            finally:
                os.unlink(output_path)

        finally:
            os.unlink(base_path)

    except Exception as e:
        logger.error(f"Synthesis failed: {e}")
        return jsonify({'error': str(e)}), 500


@app.route('/voices', methods=['GET'])
def list_voices():
    """List all saved voices."""
    voices = []
    for voice_file in VOICE_DIR.glob('*.json'):
        with open(voice_file) as f:
            data = json.load(f)
        voices.append({
            'name': voice_file.stem,
            'transcript': data.get('transcript', ''),
            'model': data.get('model', 'unknown')
        })

    return jsonify({'voices': voices})


@app.route('/voices/<name>', methods=['DELETE'])
def delete_voice(name):
    """Delete a saved voice."""
    voice_path = VOICE_DIR / f"{name}.json"
    if not voice_path.exists():
        return jsonify({'error': f"Voice '{name}' not found"}), 404

    voice_path.unlink()
    return jsonify({'success': True, 'deleted': name})


if __name__ == '__main__':
    logger.info("Starting OpenVoice V2 server...")
    logger.info(f"PyTorch version: {torch.__version__}")
    logger.info(f"CUDA available: {torch.cuda.is_available()}")

    if torch.cuda.is_available():
        logger.info(f"CUDA version: {torch.version.cuda}")
        logger.info(f"GPU: {torch.cuda.get_device_name(0)}")

    load_models()

    logger.info("Server starting on port 9280...")
    app.run(host='0.0.0.0', port=9280, threaded=True)
