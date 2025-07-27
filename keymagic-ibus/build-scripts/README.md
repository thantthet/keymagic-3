# KeyMagic Linux Build Scripts

This directory contains build scripts for creating Linux packages (DEB, RPM) and Docker-based builds for KeyMagic.

## Scripts

- `build-linux-package.sh` - Main build script for creating Linux packages
- `build-docker.sh` - Docker-based multi-architecture build script

## Usage

All scripts should be run from the project root directory:

```bash
# From project root
./keymagic-ibus/build-scripts/build-linux-package.sh [options]
./keymagic-ibus/build-scripts/build-docker.sh [options]
```

### build-linux-package.sh Options

- `--debug` - Build debug version
- `--all` - Build all package formats (default)
- `--deb` - Build only Debian package
- `--rpm` - Build only RPM package
- `--appimage` - Build only AppImage
- `--help` - Show help

### build-docker.sh Options

- `--amd64`, `--x86_64` - Build for x86_64 architecture
- `--arm64`, `--aarch64` - Build for ARM64 architecture
- `--all-arch` - Build for all architectures
- `--deb` - Build only Debian packages
- `--rpm` - Build only RPM packages
- `--debug` - Build debug version
- `--help` - Show help

## Docker Files

The Docker-related files are in the parent directory:
- `../Dockerfile.build` - Multi-architecture Dockerfile
- `../docker-compose.build.yml` - Docker Compose configuration

## Examples

```bash
# Build release packages for current architecture
./keymagic-ibus/build-scripts/build-linux-package.sh

# Build debug Debian package only
./keymagic-ibus/build-scripts/build-linux-package.sh --debug --deb

# Build for all architectures using Docker
./keymagic-ibus/build-scripts/build-docker.sh --all-arch

# Build ARM64 packages using Docker
./keymagic-ibus/build-scripts/build-docker.sh --arm64
```

## Output

All built packages will be placed in:
- `keymagic-ibus/dist/` - Final packages
- `keymagic-ibus/dist-amd64/` - AMD64 architecture builds (Docker)
- `keymagic-ibus/dist-arm64/` - ARM64 architecture builds (Docker)