#!/usr/bin/env bash
set -euo pipefail

# Build 32-bit Windows GUI targeting original Pentium (no SSE/SSE2).
# Must be run from the repo root. Requires nightly toolchain.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

# Ensure the i686 target is installed (rustc needs it for target spec)
if ! rustup target list --installed | grep -q i686-pc-windows-gnu; then
    echo "Installing i686-pc-windows-gnu target..."
    rustup target add i686-pc-windows-gnu
fi

# Ensure nightly rust-src is installed for -Z build-std
if ! rustup component list --installed --toolchain nightly | grep -q rust-src; then
    echo "Installing rust-src for nightly..."
    rustup component add rust-src --toolchain nightly
fi

# Find or download the MinGW-w64 cross toolchain
search_base="${CROSS_COMPILER_DIR:-/tmp/rust-cross-compiler}"
mkdir -p "$search_base"
dest_dir="$search_base/i686-w64-mingw32-cross"

if [ ! -d "$dest_dir/bin" ]; then
    existing_dir=$(find "$search_base" -maxdepth 1 -type d -name "i686-w64-mingw32*" 2>/dev/null | head -1)
    if [ -n "$existing_dir" ] && [ -d "$existing_dir/bin" ]; then
        dest_dir="$existing_dir"
    fi
fi

if [ ! -d "$dest_dir/bin" ]; then
    if command -v i686-w64-mingw32-gcc >/dev/null 2>&1; then
        gcc_bin=$(command -v i686-w64-mingw32-gcc)
        dest_dir="$(cd "$(dirname "$gcc_bin")/.." && pwd)"
    fi
fi

if [ ! -d "$dest_dir/bin" ]; then
    echo "Downloading MinGW-w64 cross toolchain..."
    tar_url="https://musl.cc/i686-w64-mingw32-cross.tgz"
    curl -fsSL "$tar_url" -o "$search_base/i686-w64-mingw32-cross.tgz"
    echo "Extracting MinGW-w64 cross toolchain..."
    tar -xf "$search_base/i686-w64-mingw32-cross.tgz" -C "$search_base"
    rm -f "$search_base/i686-w64-mingw32-cross.tgz"
fi

if [ ! -d "$dest_dir/bin" ]; then
    echo "Could not find MinGW-w64 toolchain in $search_base." >&2
    exit 1
fi

mingw_bin="$dest_dir/bin"
mingw_lib="$dest_dir/i686-w64-mingw32/lib"
gcc_lib=$(find "$dest_dir/lib/gcc/i686-w64-mingw32" -maxdepth 1 -type d ! -path "*/i686-w64-mingw32" 2>/dev/null | sort -V | tail -1)

echo "Using MinGW at: $mingw_bin"

export PATH="$mingw_bin:$PATH"
export CARGO_TARGET_I586_PC_WINDOWS_GNU_LINKER="i686-w64-mingw32-gcc"
export CC_i586_pc_windows_gnu="i686-w64-mingw32-gcc"
export AR_i586_pc_windows_gnu="i686-w64-mingw32-ar"
export WINDRES="i686-w64-mingw32-windres"
export RUSTFLAGS="-L $mingw_lib -L $gcc_lib"

cargo +nightly build \
    -Z json-target-spec \
    -Z build-std=std,panic_abort,core,alloc \
    --target build-config/i586-pc-windows-gnu.json \
    --features gui \
    --bin gui \
    --release

mkdir -p target/dist
cp target/i586-pc-windows-gnu/release/gui.exe target/dist/rustid_x86_32.exe 2>/dev/null || true

