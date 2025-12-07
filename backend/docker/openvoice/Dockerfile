# OpenVoice V2 Docker Container
# Target: NVIDIA RTX 5060 (Blackwell) - Requires CUDA 12.8+
#
# Copyright (c) 2025 Michael A Wright
# MIT License

FROM nvidia/cuda:12.8.0-cudnn-devel-ubuntu24.04

LABEL maintainer="open-tts-rs"
LABEL description="OpenVoice V2 TTS Server for Blackwell GPUs"
LABEL cuda_version="12.8"
LABEL gpu_arch="Blackwell (sm_120)"

# Prevent interactive prompts
ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=UTC

# CUDA environment for Blackwell
ENV CUDA_HOME=/usr/local/cuda
ENV PATH=${CUDA_HOME}/bin:${PATH}
ENV LD_LIBRARY_PATH=${CUDA_HOME}/lib64:${LD_LIBRARY_PATH}
ENV TORCH_CUDA_ARCH_LIST="12.0"

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3.12 \
    python3.12-venv \
    python3.12-dev \
    python3-pip \
    git \
    wget \
    curl \
    ffmpeg \
    libsndfile1 \
    libsndfile1-dev \
    libportaudio2 \
    portaudio19-dev \
    build-essential \
    cmake \
    mecab \
    libmecab-dev \
    mecab-ipadic-utf8 \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust via rustup (Ubuntu's packaged Rust is too old for tokenizers)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
# Allow invalid_reference_casting lint (tokenizers source has this issue with newer Rust)
ENV RUSTFLAGS="-A invalid_reference_casting"

# Set python3.12 as default
RUN update-alternatives --install /usr/bin/python3 python3 /usr/bin/python3.12 1 \
    && update-alternatives --install /usr/bin/python python /usr/bin/python3.12 1

# Create app directory
WORKDIR /app

# Create and activate virtual environment
RUN python3 -m venv /app/venv
ENV PATH="/app/venv/bin:$PATH"

# Upgrade pip
RUN pip install --no-cache-dir --upgrade pip setuptools wheel

# Install PyTorch with CUDA 12.8 support (nightly for Blackwell)
# Blackwell (sm_120) requires PyTorch nightly builds
RUN pip install --no-cache-dir --pre torch torchaudio \
    --index-url https://download.pytorch.org/whl/nightly/cu128

# Install OpenVoice dependencies
# Install transformers (with tokenizers) from PyPI first to get prebuilt wheels
RUN pip install --no-cache-dir \
    transformers \
    numpy \
    scipy \
    librosa \
    soundfile \
    unidecode \
    phonemizer \
    pypinyin \
    cn2an \
    jieba \
    langid \
    gradio \
    faster-whisper \
    pydub \
    wavmark \
    whisper-timestamped \
    flask \
    flask-cors \
    gunicorn

# Clone and install OpenVoice
RUN git clone https://github.com/myshell-ai/OpenVoice.git /app/openvoice
WORKDIR /app/openvoice

# Install MeloTTS (required for OpenVoice base TTS)
RUN pip install --no-cache-dir git+https://github.com/myshell-ai/MeloTTS.git

# Download unidic dictionary for Japanese text processing
RUN python -m unidic download

# Install OpenVoice (--no-deps to avoid ancient pinned versions incompatible with Python 3.12)
RUN pip install --no-cache-dir --no-deps -e .

# Patch se_extractor.py to use CPU for WhisperModel (avoids cuDNN library issues)
RUN sed -i 's/device="cuda", compute_type="float16"/device="cpu", compute_type="int8"/g' /app/openvoice/openvoice/se_extractor.py

# Download OpenVoice V2 checkpoints from HuggingFace
RUN python -c "\
from huggingface_hub import snapshot_download; \
import shutil, os; \
print('Downloading OpenVoice V2 checkpoints...'); \
snapshot_download(repo_id='myshell-ai/OpenVoiceV2', local_dir='/app/openvoice/checkpoints_v2'); \
print('Download complete')"

# Pre-download MeloTTS models
RUN python -c "from melo.api import TTS; TTS(language='EN', device='cpu'); print('MeloTTS models cached')"

# Download required NLTK data for text processing
RUN python -c "import nltk; nltk.download('averaged_perceptron_tagger_eng'); print('NLTK data downloaded')"

# Copy server script
WORKDIR /app
COPY server.py /app/server.py

# Expose port
EXPOSE 9280

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:9280/health || exit 1

# Run server
CMD ["python", "server.py"]
