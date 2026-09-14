#!/usr/bin/env bash
set -euo pipefail

# Build a standalone Linux AppImage for rustid GUI.
# Must be run from the repo root or build-config directory.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

ARCH="$(uname -m)"
echo "Building Linux GUI binary (${ARCH})..."
cargo build --features gui --bin gui --release

# Staging AppDir
APPDIR="target/appimage/AppDir"
DIST_DIR="target/dist"
OUTPUT_APPIMAGE="$DIST_DIR/rustid-${ARCH}.AppImage"

echo "Preparing AppDir layout at $APPDIR..."
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/metainfo"
mkdir -p "$APPDIR/usr/share/pixmaps"
mkdir -p "$APPDIR/usr/share/icons/hicolor/16x16/apps"
mkdir -p "$APPDIR/usr/share/icons/hicolor/32x32/apps"
mkdir -p "$APPDIR/usr/share/icons/hicolor/64x64/apps"
mkdir -p "$APPDIR/usr/share/icons/hicolor/128x128/apps"
mkdir -p "$APPDIR/usr/share/icons/hicolor/1024x1024/apps"
mkdir -p "$APPDIR/usr/share/icons/hicolor/scalable/apps"
mkdir -p "$DIST_DIR"

# Stage binary
cp target/release/gui "$APPDIR/usr/bin/rustid-gui"
chmod +x "$APPDIR/usr/bin/rustid-gui"

# Stage desktop file
cp build-config/rustid.desktop "$APPDIR/rustid.desktop"
cp build-config/rustid.desktop "$APPDIR/usr/share/applications/rustid.desktop"

# Stage metainfo
if [ -f build-config/rustid.metainfo.xml ]; then
    cp build-config/rustid.metainfo.xml "$APPDIR/usr/share/metainfo/net.timshomepage.rustid.metainfo.xml"
fi

# Stage root icons & .DirIcon for desktop environments / file managers
cp assets/rustid.png "$APPDIR/rustid.png"
cp assets/rustid.png "$APPDIR/.DirIcon"
cp assets/rustid.png "$APPDIR/usr/share/pixmaps/rustid.png"
cp assets/rustid.png "$APPDIR/usr/share/icons/hicolor/1024x1024/apps/rustid.png"

if [ -f assets/haiku/rustid_16.png ]; then
    cp assets/haiku/rustid_16.png "$APPDIR/usr/share/icons/hicolor/16x16/apps/rustid.png"
fi
if [ -f assets/haiku/rustid_32.png ]; then
    cp assets/haiku/rustid_32.png "$APPDIR/usr/share/icons/hicolor/32x32/apps/rustid.png"
fi
if [ -f assets/haiku/rustid_64.png ]; then
    cp assets/haiku/rustid_64.png "$APPDIR/usr/share/icons/hicolor/64x64/apps/rustid.png"
fi
if [ -f assets/haiku/rustid_128.png ]; then
    cp assets/haiku/rustid_128.png "$APPDIR/usr/share/icons/hicolor/128x128/apps/rustid.png"
fi
if [ -f assets/rustid.svg ]; then
    cp assets/rustid.svg "$APPDIR/rustid.svg"
    cp assets/rustid.svg "$APPDIR/usr/share/pixmaps/rustid.svg"
    cp assets/rustid.svg "$APPDIR/usr/share/icons/hicolor/scalable/apps/rustid.svg"
fi

# Stage AppRun launcher
cat << 'EOF' > "$APPDIR/AppRun"
#!/bin/sh
HERE="$(dirname "$(readlink -f "${0}")")"
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"
export XDG_DATA_DIRS="${HERE}/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
exec "${HERE}/usr/bin/rustid-gui" "$@"
EOF
chmod +x "$APPDIR/AppRun"

# Normalize architecture names for AppImage tooling
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64|amd64)
        TOOL_ARCH="x86_64"
        ;;
    aarch64|arm64)
        TOOL_ARCH="aarch64"
        ;;
    i386|i486|i586|i686)
        TOOL_ARCH="i686"
        ;;
    armv7*|armv6*|armhf)
        TOOL_ARCH="armhf"
        ;;
    *)
        TOOL_ARCH="$ARCH"
        ;;
esac

# Find or download appimagetool
APPIMAGETOOL=""
if command -v appimagetool >/dev/null 2>&1; then
    APPIMAGETOOL="$(command -v appimagetool)"
    echo "Found system appimagetool at: $APPIMAGETOOL"
else
    CACHE_DIR="${APPIMAGETOOL_CACHE_DIR:-$HOME/.cache/appimagetool}"
    mkdir -p "$CACHE_DIR"
    CACHED_TOOL="$CACHE_DIR/appimagetool-${TOOL_ARCH}.AppImage"

    # Download if not cached or if cached file is empty/corrupt
    if [ ! -s "$CACHED_TOOL" ]; then
        rm -f "$CACHED_TOOL"
        echo "Downloading appimagetool for ${TOOL_ARCH}..."
        TEMP_FILE="$CACHE_DIR/appimagetool-${TOOL_ARCH}.tmp.$$"

        URLS=(
            "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-${TOOL_ARCH}.AppImage"
            "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-${TOOL_ARCH}.AppImage"
        )

        DOWNLOAD_SUCCESS=0
        for url in "${URLS[@]}"; do
            echo "Attempting download from: $url..."
            if curl -fL --retry 3 --connect-timeout 15 "$url" -o "$TEMP_FILE"; then
                if [ -s "$TEMP_FILE" ]; then
                    chmod +x "$TEMP_FILE"
                    mv "$TEMP_FILE" "$CACHED_TOOL"
                    DOWNLOAD_SUCCESS=1
                    break
                fi
            fi
            rm -f "$TEMP_FILE"
        done

        if [ "$DOWNLOAD_SUCCESS" -ne 1 ]; then
            echo "Error: Failed to download appimagetool for ${TOOL_ARCH}." >&2
            echo "Please install appimagetool or place it at: $CACHED_TOOL" >&2
            exit 1
        fi
    fi
    APPIMAGETOOL="$CACHED_TOOL"
    echo "Using cached appimagetool at: $APPIMAGETOOL"
fi

echo "Generating AppImage: $OUTPUT_APPIMAGE..."
export ARCH="$TOOL_ARCH"
export APPIMAGE_EXTRACT_AND_RUN=1

# Remove previous artifact if exists
rm -f "$OUTPUT_APPIMAGE"

"$APPIMAGETOOL" "$APPDIR" "$OUTPUT_APPIMAGE"
chmod +x "$OUTPUT_APPIMAGE"

echo "Successfully built AppImage at: $OUTPUT_APPIMAGE"
