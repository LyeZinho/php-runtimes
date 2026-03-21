#!/usr/bin/env bash
set -euo pipefail

echo "Installing MinGW-w64 cross-compilation toolchain..."

# Detect package manager
if command -v apt-get >/dev/null 2>&1; then
    sudo apt-get update
    sudo apt-get install -y \
        mingw-w64 \
        gcc-mingw-w64-x86-64 \
        g++-mingw-w64-x86-64 \
        binutils-mingw-w64-x86-64 \
        mingw-w64-tools
elif command -v pacman >/dev/null 2>&1; then
    sudo pacman -S mingw-w64-gcc
elif command -v dnf >/dev/null 2>&1; then
    sudo dnf install mingw64-gcc
else
    echo "Unsupported package manager. Install MinGW-w64 manually."
    exit 1
fi

# Verify installation
echo "Verifying installation..."
x86_64-w64-mingw32-gcc --version
x86_64-w64-mingw32-g++ --version
x86_64-w64-mingw32-strip --version

echo "MinGW-w64 toolchain installed successfully."
