FROM rust:1.93.0-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        bash \
        ffmpeg \
        procps \
        x11-utils \
        xauth \
        xterm \
        xvfb \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
