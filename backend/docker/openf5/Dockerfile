# OpenF5-TTS Docker Container
# Target: NVIDIA RTX 5060 (Blackwell) - Requires CUDA 12.8+
#
# IMPORTANT: Uses Apache 2.0 licensed OpenF5 weights (NOT the CC-BY-NC F5-TTS weights)
#
# Copyright (c) 2025 Michael A Wright
# MIT License

FROM nvidia/cuda:12.8.0-cudnn-devel-ubuntu24.04

LABEL maintainer="open-tts-rs"
LABEL description="OpenF5-TTS Server for Blackwell GPUs (Apache 2.0 weights)"
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
    git-lfs \
    wget \
    curl \
    ffmpeg \
    libsndfile1 \
    libsndfile1-dev \
    libportaudio2 \
    portaudio19-dev \
    build-essential \
    cmake \
    espeak-ng \
    && rm -rf /var/lib/apt/lists/*

# Set python3.12 as default
RUN update-alternatives --install /usr/bin/python3 python3 /usr/bin/python3.12 1 \
    && update-alternatives --install /usr/bin/python python /usr/bin/python3.12 1

# Initialize git-lfs
RUN git lfs install

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

# Install F5-TTS dependencies
RUN pip install --no-cache-dir \
    numpy \
    scipy \
    librosa \
    soundfile \
    transformers \
    accelerate \
    einops \
    einx \
    x-transformers \
    vocos \
    jieba \
    pypinyin \
    cached_path \
    flask \
    flask-cors \
    gunicorn \
    huggingface_hub

# Clone F5-TTS repository
RUN git clone https://github.com/SWivid/F5-TTS.git /app/f5-tts
WORKDIR /app/f5-tts

# Install F5-TTS
RUN pip install --no-cache-dir -e .

# Download OpenF5 weights (Apache 2.0 licensed - IMPORTANT!)
# These are the commercially-usable weights trained on Emilia-YODAS dataset
RUN python -c "\
from huggingface_hub import hf_hub_download; \
import os; \
os.makedirs('/app/models/F5-TTS', exist_ok=True); \
print('Downloading OpenF5 Apache 2.0 weights...'); \
hf_hub_download(repo_id='SWivid/F5-TTS', filename='F5TTS_Base/model_1200000.safetensors', local_dir='/app/models'); \
hf_hub_download(repo_id='SWivid/F5-TTS', filename='F5TTS_Base/vocab.txt', local_dir='/app/models'); \
print('Download complete')"

# Copy server script
WORKDIR /app
COPY server.py /app/server.py

# Create voices directory
RUN mkdir -p /app/voices

# Expose port
EXPOSE 9288

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=120s --retries=3 \
    CMD curl -f http://localhost:9288/health || exit 1

# Run server
CMD ["python", "server.py"]
