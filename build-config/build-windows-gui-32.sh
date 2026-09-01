#!/usr/bin/env bash
set -euo pipefail

# Build 32-bit Windows GUI targeting original Pentium (no SSE/SSE2).
# Must be run from the repo root. Requires cargo cross and nightly toolchain.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

# Ensure the i686 target is installed (cargo cross needs it even for JSON target builds)
if ! rustup target list --installed | grep -q i686-pc-windows-gnu; then
    echo "Installing i686-pc-windows-gnu target..."
    rustup target add i686-pc-windows-gnu
fi

# Find the MinGW-w64 toolchain (downloaded by cargo cross)
search_base="${CROSS_COMPILER_DIR:-/tmp/rust-cross-compiler}"

mingw_dir=$(find "$search_base" -maxdepth 1 -type d -name "i686-w64-mingw32*" 2>/dev/null | head -1)

if [ -z "$mingw_dir" ] || [ ! -d "$mingw_dir/bin" ]; then
    echo "MinGW-w64 toolchain not found. Downloading via cargo cross..."
    cargo cross build --target i686-pc-windows-gnu --features gui --bin rustid-gui --release 2>/dev/null || true
    mingw_dir=$(find "$search_base" -maxdepth 1 -type d -name "i686-w64-mingw32*" 2>/dev/null | head -1)
fi

if [ -z "$mingw_dir" ] || [ ! -d "$mingw_dir/bin" ]; then
    echo "Could not find MinGW-w64 toolchain in $search_base. Set CROSS_COMPILER_DIR or run 'cargo cross build --target i686-pc-windows-gnu' first." >&2
    exit 1
fi

mingw_bin="$mingw_dir/bin"
mingw_lib="$mingw_dir/i686-w64-mingw32/lib"
gcc_lib=$(find "$mingw_dir/lib/gcc/i686-w64-mingw32" -maxdepth 1 -type d ! -path "*/i686-w64-mingw32" | sort -V | tail -1)

echo "Using MinGW at: $mingw_bin"

export PATH="$mingw_bin:$PATH"
export CARGO_TARGET_I586_PC_WINDOWS_GNU_LINKER="i686-w64-mingw32-gcc"
export CC_i586_pc_windows_gnu="i686-w64-mingw32-gcc"
export AR_i586_pc_windows_gnu="i686-w64-mingw32-ar"
export RUSTFLAGS="-L $mingw_lib -L $gcc_lib"

exec cargo +nightly build \
    -Z json-target-spec \
    -Z build-std=std,panic_abort,core,alloc \
    --target build-config/i586-pc-windows-gnu.json \
    --features gui \
    --bin rustid-gui \
    --release
