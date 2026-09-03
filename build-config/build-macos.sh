#!/usr/bin/env bash
set -euo pipefail

# Build the macOS GUI (rustid-gui) as a universal "fat" binary and package it
# as Rustid.app, zipped as rustid-macos.zip.
#
# Works two ways:
#   * Natively on macOS: Apple's clang builds both x86_64 and arm64 slices.
#   * Cross-compiling from Linux: requires osxcross providing the Apple SDK +
#     linker wrappers (<triple>-clang). See MACOS.md.
# Must be run from the repo root.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

# Ensure the Apple targets are installed
for target in x86_64-apple-darwin aarch64-apple-darwin; do
    if ! rustup target list --installed | grep -q "$target"; then
        echo "Installing $target target..."
        rustup target add "$target"
    fi
done

BUILD_DIR="target/macos"
OUT="${BUILD_DIR}/rustid-gui-macos"

rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

# On a native macOS host, .cargo/config.toml points the Darwin targets at the
# osxcross <triple>-clang wrappers, which do not exist natively. Natively we
# build both slices with the system clang, so override the linker for both the
# target slices and the host (build scripts run on the host arch).
if [[ "$(uname -s)" == "Darwin" ]]; then
    export CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER=clang
    export CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER=clang
fi

# Build the GUI for each slice.
echo "Building x86_64 slice..."
cargo build --target x86_64-apple-darwin --features gui --bin rustid-gui --release

echo "Building aarch64 slice..."
cargo build --target aarch64-apple-darwin --features gui --bin rustid-gui --release

# Merge into a universal binary. Prefer Apple's lipo when available, otherwise
# use tools/make_fat (the fat-macho helper in this repo).
SLICE_X86="target/x86_64-apple-darwin/release/rustid-gui"
SLICE_ARM="target/aarch64-apple-darwin/release/rustid-gui"

if command -v lipo >/dev/null 2>&1; then
    lipo -create "$SLICE_X86" "$SLICE_ARM" -output "$OUT"
else
    echo "lipo not found; using tools/make_fat..."
    cargo run --quiet --manifest-path tools/make_fat/Cargo.toml -- \
        "$SLICE_X86" "$SLICE_ARM" "$OUT"
fi

echo "Universal binary:"
file "$OUT"

# Assemble the .app bundle.
BUNDLE="$BUILD_DIR/Rustid.app"
mkdir -p "$BUNDLE/Contents/MacOS" "$BUNDLE/Contents/Resources"
cp "$OUT" "$BUNDLE/Contents/MacOS/rustid-gui"

# Minimal Info.plist (CFBundleExecutable must match the binary name).
cat > "$BUNDLE/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>Rustid</string>
    <key>CFBundleDisplayName</key><string>Rustid</string>
    <key>CFBundleIdentifier</key><string>net.timshomepage.rustid.gui</string>
    <key>CFBundleExecutable</key><string>rustid-gui</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>2.1.1</string>
    <key>CFBundleVersion</key><string>2.1.1</string>
    <key>LSMinimumSystemVersion</key><string>11.0</string>
    <key>NSHighResolutionCapable</key><true/>
    <key>NSPrincipalClass</key><string>NSApplication</string>
</dict>
</plist>
PLIST

# Zip the bundle for distribution (mirrors rustid-windows.zip).
mkdir -p target/dist
(cd "$BUILD_DIR" && zip -qr "$REPO_ROOT/target/dist/rustid-macos.zip" Rustid.app)

echo "Created target/dist/rustid-macos.zip"
