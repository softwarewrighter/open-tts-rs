#!/usr/bin/env python3
"""Patch F5-TTS to use soundfile instead of torchaudio.load (TorchCodec compatibility)"""

import sys

filepath = '/app/f5-tts/src/f5_tts/infer/utils_infer.py'

with open(filepath, 'r') as f:
    content = f.read()

old_code = 'audio, sr = torchaudio.load(ref_audio)'
new_code = '''# Patched: use soundfile instead of torchaudio (TorchCodec compatibility)
    import soundfile as _sf
    _audio_np, sr = _sf.read(ref_audio)
    audio = torch.from_numpy(_audio_np).float()
    if audio.dim() == 1:
        audio = audio.unsqueeze(0)
    else:
        audio = audio.T'''

if old_code in content:
    content = content.replace(old_code, new_code)
    with open(filepath, 'w') as f:
        f.write(content)
    print('Successfully patched utils_infer.py')
else:
    print('Warning: Could not find code to patch - may already be patched')
    sys.exit(0)
