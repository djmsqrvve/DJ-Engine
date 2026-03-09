FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm

RUN apt-get update \
    && export DEBIAN_FRONTEND=noninteractive \
    && apt-get install -y --no-install-recommends \
        pkg-config \
        libasound2-dev \
        libudev-dev \
        libwayland-dev \
        libxkbcommon-dev \
        libxkbcommon-x11-dev \
        libxcursor-dev \
        libxi-dev \
        libxrandr-dev \
        libxinerama-dev \
        libx11-dev \
        libx11-xcb-dev \
        libgl1-mesa-dev \
        libgl1-mesa-dri \
        libvulkan-dev \
        mesa-utils \
        mesa-vulkan-drivers \
        libxcb1-dev \
        libxcb-render0-dev \
        libxcb-shape0-dev \
        libxcb-xfixes0-dev \
        clang \
        lld \
        cmake \
        git \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*
