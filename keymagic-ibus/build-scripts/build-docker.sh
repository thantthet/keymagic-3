#!/bin/bash
# Docker-based multi-architecture build script for KeyMagic

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
# Get project root (two levels up from build-scripts)
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

echo -e "${BLUE}KeyMagic 3 Docker Build System${NC}"
echo "================================="
echo ""

# Check Docker and buildx availability
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Docker is not installed or not in PATH${NC}"
    exit 1
fi

# Check if buildx is available
if ! docker buildx version &> /dev/null; then
    echo -e "${YELLOW}Docker buildx not found. Multi-arch builds may not work.${NC}"
    echo "Please update Docker or enable experimental features."
fi

# Default values
ARCHITECTURES=()
PACKAGE_FORMAT="all"
BUILD_TYPE="release"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --amd64|--x86_64)
            ARCHITECTURES+=("amd64")
            shift
            ;;
        --arm64|--aarch64)
            ARCHITECTURES+=("arm64")
            shift
            ;;
        --all-arch)
            ARCHITECTURES=("amd64" "arm64")
            shift
            ;;
        --deb)
            PACKAGE_FORMAT="deb"
            shift
            ;;
        --rpm)
            PACKAGE_FORMAT="rpm"
            shift
            ;;
        --debug)
            BUILD_TYPE="debug"
            shift
            ;;
        --help)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --amd64, --x86_64    Build for x86_64 architecture"
            echo "  --arm64, --aarch64   Build for ARM64 architecture"
            echo "  --all-arch           Build for all architectures (default if none specified)"
            echo "  --deb                Build only Debian packages"
            echo "  --rpm                Build only RPM packages"
            echo "  --debug              Build debug version"
            echo "  --help               Show this help"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Default to current architecture if none specified
if [ ${#ARCHITECTURES[@]} -eq 0 ]; then
    CURRENT_ARCH=$(uname -m)
    case $CURRENT_ARCH in
        x86_64)
            ARCHITECTURES=("amd64")
            ;;
        aarch64)
            ARCHITECTURES=("arm64")
            ;;
        *)
            echo -e "${RED}Unsupported architecture: $CURRENT_ARCH${NC}"
            exit 1
            ;;
    esac
fi

# Performance warning for emulation
CURRENT_ARCH=$(uname -m)
for ARCH in "${ARCHITECTURES[@]}"; do
    if [ "$CURRENT_ARCH" = "x86_64" ] && [ "$ARCH" = "arm64" ]; then
        echo -e "${YELLOW}Note: Building ARM64 on x86_64 uses emulation and will be slower${NC}"
    elif [ "$CURRENT_ARCH" = "aarch64" ] && [ "$ARCH" = "amd64" ]; then
        echo -e "${YELLOW}Note: Building x86_64 on ARM64 uses emulation and will be slower${NC}"
    fi
done
echo ""

# Build for each architecture
for ARCH in "${ARCHITECTURES[@]}"; do
    echo -e "${BLUE}Building for $ARCH architecture...${NC}"
    
    # Build Docker image
    echo -e "${YELLOW}Building Docker image for $ARCH...${NC}"
    docker build \
        --platform linux/${ARCH} \
        --build-arg ARCH=${ARCH} \
        -t keymagic-builder:${ARCH} \
        -f keymagic-ibus/Dockerfile.build \
        .
    
    # Run build in container
    echo -e "${YELLOW}Running build in container...${NC}"
    docker run \
        --rm \
        --platform linux/${ARCH} \
        -v "${PROJECT_ROOT}/keymagic-ibus/dist-${ARCH}:/build/keymagic-ibus/dist" \
        keymagic-builder:${ARCH} \
        ./keymagic-ibus/build-scripts/build-linux-package.sh --${PACKAGE_FORMAT} $([ "$BUILD_TYPE" = "debug" ] && echo "--debug")
    
    echo -e "${GREEN}✓ Build completed for $ARCH${NC}"
    echo ""
done

# Collect all packages
echo -e "${BLUE}Collecting packages...${NC}"
mkdir -p keymagic-ibus/dist
for ARCH in "${ARCHITECTURES[@]}"; do
    if [ -d "keymagic-ibus/dist-${ARCH}" ]; then
        cp -v keymagic-ibus/dist-${ARCH}/*.{deb,rpm} keymagic-ibus/dist/ 2>/dev/null || true
    fi
done

echo -e "${GREEN}Build complete!${NC}"
echo "Packages are available in the keymagic-ibus/dist/ directory:"
ls -la keymagic-ibus/dist/