#!/bin/bash
# Script to build the library and run FFI tests

set -e

echo "=== Building KeyMagic Core library ==="
cd "$(dirname "$0")/../.."

# Build in release mode for better performance
cargo build --release -p keymagic-core

echo -e "\n=== Running Python FFI tests ==="
python3 keymagic-core/tests/test_ffi.py