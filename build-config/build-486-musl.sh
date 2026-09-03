#!/usr/bin/env bash
set -euo pipefail

# Build 32-bit Linux musl targeting i486 (no SSE/SSE2).
# Must be run from the repo root. Requires the nightly toolchain with rust-src.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

# Ensure nightly rust-src is installed for -Z build-std
if ! rustup component list --installed --toolchain nightly | grep -q rust-src; then
    echo "Installing rust-src for nightly..."
    rustup component add rust-src --toolchain nightly
fi

# Find or download the i486 musl cross toolchain
search_base="${CROSS_COMPILER_DIR:-/tmp/rust-cross-compiler}"
mkdir -p "$search_base"
dest_dir="$search_base/i486-linux-musl-cross"

if [ ! -d "$dest_dir/bin" ]; then
    echo "Downloading i486 musl cross toolchain..."
    tar_url="https://musl.cc/i486-linux-musl-cross.tgz"
    curl -fsSL "$tar_url" -o "$search_base/i486-linux-musl-cross.tgz"
    echo "Extracting i486 musl cross toolchain..."
    tar -xf "$search_base/i486-linux-musl-cross.tgz" -C "$search_base"
    rm -f "$search_base/i486-linux-musl-cross.tgz"
fi

if [ ! -d "$dest_dir/bin" ]; then
    echo "Could not find i486 musl toolchain in $search_base. Set CROSS_COMPILER_DIR or run 'cargo cross check --target i486-unknown-linux-musl' first." >&2
    exit 1
fi

musl_bin="$dest_dir/bin"
musl_sysroot_lib="$dest_dir/i486-linux-musl/lib"
gcc_lib=$(find "$dest_dir/lib/gcc/i486-linux-musl" -maxdepth 1 -type d ! -path "*/i486-linux-musl" 2>/dev/null | sort -V | tail -1)

echo "Using i486 musl toolchain at: $musl_bin"

export PATH="$musl_bin:$PATH"
export CARGO_TARGET_I486_UNKNOWN_LINUX_MUSL_LINKER="i486-linux-musl-gcc"
export CC_i486_unknown_linux_musl="i486-linux-musl-gcc"
export AR_i486_unknown_linux_musl="i486-linux-musl-ar"
export RUSTFLAGS="-L $musl_sysroot_lib -L $gcc_lib"

cargo +nightly build \
    -Z json-target-spec \
    -Z build-std=std,panic_abort,core,alloc \
    --target build-config/i486-unknown-linux-musl.json \
    --release

mkdir -p target/dist
cp target/i486-unknown-linux-musl/release/rustid target/dist/rustid_486 2>/dev/null || true