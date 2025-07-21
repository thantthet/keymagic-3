# Building KeyMagic 3 Packages

This document describes how to build KeyMagic 3 packages for Linux distributions.

## Quick Start

### Using Docker (Recommended)

The easiest way to build packages for multiple architectures is using Docker:

```bash
# Build for current architecture
./build-docker.sh

# Build for specific architecture
./build-docker.sh --amd64
./build-docker.sh --arm64

# Build for all architectures
./build-docker.sh --all-arch

# Build only specific package format
./build-docker.sh --amd64 --deb
./build-docker.sh --all-arch --rpm
```

Packages will be created in the `dist/` directory.

### Using Docker Compose

For building multiple architectures simultaneously:

```bash
# Build all architectures
docker-compose -f docker-compose.build.yml up

# Build specific architecture
docker-compose -f docker-compose.build.yml up build-amd64
docker-compose -f docker-compose.build.yml up build-arm64
```

### Native Build

If you prefer to build natively on your system:

```bash
# Install dependencies (Ubuntu/Debian)
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libibus-1.0-dev \
    libglib2.0-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    librsvg2-dev \
    rpm

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Tauri CLI
cargo install tauri-cli

# Build packages
./build-linux-package.sh
```

## Build Options

### Package Formats

- `--deb`: Build only Debian packages
- `--rpm`: Build only RPM packages
- Default: Build both formats

### Build Types

- `--debug`: Build debug version
- Default: Build release version

### Architectures (Docker only)

- `--amd64` / `--x86_64`: Build for x86_64
- `--arm64` / `--aarch64`: Build for ARM64
- `--all-arch`: Build for all architectures

## Output

Packages are created in the `dist/` directory:

- Debian packages: `keymagic3_0.0.1_<arch>.deb`
- RPM packages: `keymagic3-0.0.1-1.<arch>.rpm`

## Requirements

### For Docker Build

#### Docker Version
- Docker 19.03+ (for buildx support)
- Docker Desktop 4.0+ recommended for Mac/Windows

#### Multi-Architecture Support
- **Linux**: QEMU user-mode emulation must be installed
- **macOS**: Docker Desktop includes built-in emulation
- **Windows**: Docker Desktop with WSL2 backend includes emulation

#### Setup on Linux
If building cross-architecture on Linux, install QEMU:
```bash
# Install QEMU user-mode emulation
sudo apt-get update
sudo apt-get install -y qemu-user-static

# Verify multi-arch support
docker run --rm --privileged multiarch/qemu-user-static --reset -p yes

# Check available platforms
docker buildx ls
```

#### Memory Requirements
- Minimum 4GB RAM for single architecture build
- Recommended 8GB+ RAM for multi-architecture builds
- Rust compilation is memory-intensive

### For Native Build
- Ubuntu 22.04+ or equivalent
- Rust 1.70+
- Development libraries for IBus, GTK3, WebKitGTK

## Troubleshooting

### Docker Platform Issues

If you encounter platform errors, ensure Docker buildx is enabled:

```bash
docker buildx create --use
docker buildx inspect --bootstrap
```

### Missing Dependencies

For native builds, ensure all development packages are installed:

```bash
pkg-config --exists ibus-1.0 || echo "IBus dev packages missing"
pkg-config --exists gtk+-3.0 || echo "GTK3 dev packages missing"
```

### Cross-Compilation

The Docker approach handles cross-compilation automatically. For native cross-compilation, additional setup is required and not recommended.