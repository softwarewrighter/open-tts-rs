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
    && rm -rf /var/lib/apt/lists/*

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
RUN pip install --no-cache-dir \
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
    flask \
    flask-cors \
    gunicorn

# Clone and install OpenVoice
RUN git clone https://github.com/myshell-ai/OpenVoice.git /app/openvoice
WORKDIR /app/openvoice

# Install MeloTTS (required for OpenVoice base TTS)
RUN pip install --no-cache-dir git+https://github.com/myshell-ai/MeloTTS.git

# Install OpenVoice
RUN pip install --no-cache-dir -e .

# Download model checkpoints
RUN python -c "from openvoice.api import ToneColorConverter; \
    from melo.api import TTS; \
    print('Downloading models...')" || true

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
