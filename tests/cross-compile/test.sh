#!/usr/bin/env bash
set -euo pipefail

echo "Testing MinGW cross-compilation via Docker..."

# Create test directory
mkdir -p tests/cross-compile
cd tests/cross-compile

# Compile for Windows using Docker
docker run --rm -v $(pwd):/work dockcross/windows-x64 \
  x86_64-w64-mingw32.static-gcc -o test.exe test.c

# Check file type
file test.exe

# Test with Wine if available
if command -v wine >/dev/null 2>&1; then
    echo "Running with Wine..."
    wine test.exe
else
    echo "Wine not installed, skipping execution test."
fi

echo "Docker cross-compilation test completed."
