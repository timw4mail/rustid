#!/usr/bin/env bash
set -euo pipefail

# Build ARM64 Windows GUI binary using LLVM MinGW/GNU target (aarch64-pc-windows-gnullvm).
# Must be run from the repo root.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

# Ensure the aarch64 target is installed
if ! rustup target list --installed | grep -q aarch64-pc-windows-gnullvm; then
    echo "Installing aarch64-pc-windows-gnullvm target..."
    rustup target add aarch64-pc-windows-gnullvm
fi

# Find or download llvm-mingw toolchain if clang is not in PATH
if ! command -v aarch64-w64-mingw32-clang >/dev/null 2>&1; then
    search_base="${CROSS_COMPILER_DIR:-/tmp/rust-cross-compiler}"
    mkdir -p "$search_base"
    dest_dir="$search_base/llvm-mingw"

    if [ ! -f "$dest_dir/bin/aarch64-w64-mingw32-clang" ]; then
        echo "Downloading llvm-mingw toolchain for Windows ARM64..."
        tar_url="https://github.com/mstorsjo/llvm-mingw/releases/download/20241119/llvm-mingw-20241119-ucrt-ubuntu-20.04-x86_64.tar.xz"
        curl -fsSL "$tar_url" -o "$search_base/llvm-mingw.tar.xz"
        echo "Extracting llvm-mingw..."
        tar -xf "$search_base/llvm-mingw.tar.xz" -C "$search_base"
        extracted=$(find "$search_base" -maxdepth 1 -type d -name "llvm-mingw-*" | head -1)
        if [ -n "$extracted" ] && [ "$extracted" != "$dest_dir" ]; then
            mv "$extracted" "$dest_dir"
        fi
        rm -f "$search_base/llvm-mingw.tar.xz"
    fi

    if [ -d "$dest_dir/bin" ]; then
        export PATH="$dest_dir/bin:$PATH"
    fi
fi

cargo build \
    --target aarch64-pc-windows-gnullvm \
    --features gui \
    --bin rustid-gui \
    --release

mkdir -p target/dist
if [ -f target/aarch64-pc-windows-gnullvm/release/rustid-gui.exe ]; then
    cp target/aarch64-pc-windows-gnullvm/release/rustid-gui.exe target/dist/rustid_arm64.exe
fi
