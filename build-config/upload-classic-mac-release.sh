#!/usr/bin/env bash
#
# upload-classic-mac-release.sh
# Uploads Classic Macintosh (Fat Binary and 800K Disk Image) and Mac OS X
# PowerPC Application Bundle assets to a GitHub release using the GitHub CLI (`gh`).
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUT_DIR="$REPO_ROOT/target/classic_mac"

TAG=""
BUILD_FIRST=false
DRY_RUN=false
REPO_OVERRIDE=""

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS] [TAG]

Uploads Classic Macintosh and Mac OS X PowerPC binaries to a GitHub release.

Arguments:
  TAG                       Release tag (e.g. v2.3.0 or 2.3.0). Defaults to the current
                            git tag or the version defined in Cargo.toml.

Options:
  -b, --build               Force rebuilding Classic Mac binaries before uploading.
  -n, --dry-run             Show what would be uploaded without making any changes.
  -r, --repo OWNER/REPO     Target GitHub repository (default: auto-detected or timw4mail/rustid).
  -h, --help                Show this help message.

Artifacts uploaded:
  - target/classic_mac/rustid.bin
      Classic Macintosh Fat Binary (System 6.0.8 - Mac OS 9.2.2)
  - target/classic_mac/Rustid-osx-ppc.zip
      Mac OS X PowerPC Application Bundle (Rustid.app)
  - target/classic_mac/rustid.dsk
      Classic Macintosh 800K Floppy Disk Image (HFS)
EOF
    exit 0
}

# Parse options
while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            usage
            ;;
        -b|--build)
            BUILD_FIRST=true
            shift
            ;;
        -n|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -r|--repo)
            if [[ -z "${2:-}" ]]; then
                echo "Error: --repo requires an argument (OWNER/REPO)" >&2
                exit 1
            fi
            REPO_OVERRIDE="$2"
            shift 2
            ;;
        -*)
            echo "Error: Unknown option: $1" >&2
            echo "Run '$(basename "$0") --help' for usage." >&2
            exit 1
            ;;
        *)
            if [[ -z "$TAG" ]]; then
                TAG="$1"
                shift
            else
                echo "Error: Unexpected argument: $1" >&2
                exit 1
            fi
            ;;
    esac
done

# 1. Verify GitHub CLI is installed and authenticated
if ! command -v gh >/dev/null 2>&1; then
    echo "Error: GitHub CLI ('gh') is not installed or not in PATH." >&2
    echo "Please install it from https://cli.github.com/ and authenticate with 'gh auth login'." >&2
    exit 1
fi

if ! gh auth status >/dev/null 2>&1; then
    echo "Error: GitHub CLI ('gh') is not authenticated." >&2
    echo "Please run 'gh auth login' to authenticate with GitHub." >&2
    exit 1
fi

# 2. Determine target GitHub repository
REPO=""
if [[ -n "$REPO_OVERRIDE" ]]; then
    REPO="$REPO_OVERRIDE"
elif [[ -n "${GH_REPO:-}" ]]; then
    REPO="$GH_REPO"
else
    # Try resolving from git remotes
    REMOTE_URL=$(git -C "$REPO_ROOT" remote get-url github 2>/dev/null || git -C "$REPO_ROOT" remote get-url origin 2>/dev/null || true)
    if [[ "$REMOTE_URL" =~ github\.com[:/]([^/]+)/([^/.]+)(\.git)?$ ]]; then
        REPO="${BASH_REMATCH[1]}/${BASH_REMATCH[2]}"
    else
        REPO="timw4mail/rustid"
    fi
fi

# 3. Determine target release tag
if [[ -z "$TAG" ]]; then
    # Try exact git tag on HEAD first
    TAG=$(git -C "$REPO_ROOT" describe --tags --exact-match 2>/dev/null || true)
    if [[ -z "$TAG" ]]; then
        # Default to Cargo.toml version
        VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' "$REPO_ROOT/Cargo.toml" | head -n 1)
        if [[ -n "$VERSION" ]]; then
            TAG="v$VERSION"
        else
            echo "Error: Could not determine release tag from git or Cargo.toml. Please specify a TAG." >&2
            exit 1
        fi
    fi
fi

# Normalize tag: ensure leading 'v' if numeric version like "2.3.0"
if [[ "$TAG" =~ ^[0-9]+\.[0-9]+ ]]; then
    TAG="v$TAG"
fi

echo "==> Target repository: $REPO"
echo "==> Target release tag: $TAG"

# 4. Check if the release exists on GitHub
if ! gh release view "$TAG" -R "$REPO" >/dev/null 2>&1; then
    echo "Error: GitHub release '$TAG' does not exist in repository '$REPO'." >&2
    echo "Please create the release first or wait for the GitHub Actions release workflow to finish." >&2
    exit 1
fi

# 5. Check artifacts and optionally build
FAT_BIN="$OUT_DIR/rustid.bin"
OSX_ZIP="$OUT_DIR/Rustid-osx-ppc.zip"
DSK_IMG="$OUT_DIR/rustid.dsk"

if [[ "$BUILD_FIRST" = true ]] || [[ ! -f "$FAT_BIN" ]] || [[ ! -f "$OSX_ZIP" ]] || [[ ! -f "$DSK_IMG" ]]; then
    echo "==> Building Classic Mac and Mac OS X binaries..."
    export PATH="${RETRO68_PREFIX:-$HOME/code/Retro68-build/toolchain}/bin:$PATH"
    make -C "$REPO_ROOT/build-config/classic_mac"
fi

# Verify artifacts exist after build check
if [[ ! -f "$FAT_BIN" ]]; then
    echo "Error: Missing expected artifact: $FAT_BIN" >&2
    exit 1
fi

if [[ ! -f "$OSX_ZIP" ]]; then
    echo "Error: Missing expected artifact: $OSX_ZIP" >&2
    exit 1
fi

if [[ ! -f "$DSK_IMG" ]]; then
    echo "Error: Missing expected artifact: $DSK_IMG" >&2
    exit 1
fi

echo "==> Artifacts ready:"
echo "    - $(ls -lh "$FAT_BIN" | awk '{print $9, "(" $5 ")"}')"
echo "    - $(ls -lh "$OSX_ZIP" | awk '{print $9, "(" $5 ")"}')"
echo "    - $(ls -lh "$DSK_IMG" | awk '{print $9, "(" $5 ")"}')"

# 6. Upload assets
ASSET_FAT="$FAT_BIN#Classic Macintosh Fat Binary (System 6 - Mac OS 9)"
ASSET_OSX="$OSX_ZIP#Mac OS X PowerPC Application Bundle (Rustid.app)"
ASSET_DSK="$DSK_IMG#Classic Macintosh 800K Floppy Disk Image (rustid.dsk)"

if [[ "$DRY_RUN" = true ]]; then
    echo "==> [DRY RUN] Would execute:"
    echo "    gh release upload \"$TAG\" \\"
    echo "      \"$ASSET_FAT\" \\"
    echo "      \"$ASSET_OSX\" \\"
    echo "      \"$ASSET_DSK\" \\"
    echo "      --clobber \\"
    echo "      -R \"$REPO\""
    echo "==> [DRY RUN] Complete. No changes made."
    exit 0
fi

echo "==> Uploading assets to release $TAG in $REPO..."
gh release upload "$TAG" \
    "$ASSET_FAT" \
    "$ASSET_OSX" \
    "$ASSET_DSK" \
    --clobber \
    -R "$REPO"

echo "==> Success! Release updated:"
echo "    https://github.com/$REPO/releases/tag/$TAG"
