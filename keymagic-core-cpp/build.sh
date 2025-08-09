#!/bin/bash

# Simple build script for KeyMagic Core C++

set -e

echo "Building KeyMagic Core C++..."

# Create build directory
mkdir -p build

# Configure with CMake
cd build
cmake .. -DCMAKE_BUILD_TYPE=Release -DBUILD_TESTING=OFF

# Build
cmake --build . --config Release

echo "Build complete!"
echo "Library location: build/"