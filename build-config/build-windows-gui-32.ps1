#!/usr/bin/env pwsh
# Build 32-bit Windows GUI targeting original Pentium (no SSE/SSE2).
# Must be run from the repo root. Requires cargo cross and nightly toolchain.

param(
    [string]$CrossCompilerDir = $env:CROSS_COMPILER_DIR
)

# Ensure the i686 target is installed (cargo cross needs it even for JSON target builds)
if (-not (rustup target list --installed | Select-String "i686-pc-windows-gnu")) {
    Write-Host "Installing i686-pc-windows-gnu target..."
    rustup target add i686-pc-windows-gnu
}

# Ensure nightly rust-src is installed for -Z build-std
if (-not (rustup component list --installed --toolchain nightly | Select-String "rust-src")) {
    Write-Host "Installing rust-src for nightly..."
    rustup component add rust-src --toolchain nightly
}

# Find the MinGW-w64 toolchain (downloaded by cargo cross)
$searchBase = if ($CrossCompilerDir) { $CrossCompilerDir } else {
    Join-Path ([System.IO.Path]::GetTempPath()) "rust-cross-compiler"
}

if (-not (Test-Path $searchBase)) {
    New-Item -ItemType Directory -Path $searchBase | Out-Null
}

$mingwDir = Get-ChildItem -Path $searchBase -Filter "i686-w64-mingw32*" -Directory -ErrorAction SilentlyContinue |
    Where-Object { Test-Path (Join-Path $_.FullName "bin\dlltool.exe") } |
    Select-Object -First 1

if (-not $mingwDir) {
    Write-Host "MinGW-w64 toolchain not found. Downloading via cargo cross..."
    cargo cross build --target i686-pc-windows-gnu --features gui --bin rustid-gui --release 2>&1 | Out-Null
    $mingwDir = Get-ChildItem -Path $searchBase -Filter "i686-w64-mingw32*" -Directory -ErrorAction SilentlyContinue |
        Where-Object { Test-Path (Join-Path $_.FullName "bin\dlltool.exe") } |
        Select-Object -First 1
}

if (-not $mingwDir) {
    Write-Error "Could not find MinGW-w64 toolchain in $searchBase. Set CROSS_COMPILER_DIR or run 'cargo cross build --target i686-pc-windows-gnu' first."
    exit 1
}

$mingwBin = Join-Path $mingwDir.FullName "bin"
$mingwLib = Join-Path $mingwDir.FullName "i686-w64-mingw32\lib"
$gccLib   = Join-Path $mingwDir.FullName "lib\gcc\i686-w64-mingw32" |
    Get-ChildItem -Directory | Select-Object -First 1 | ForEach-Object { $_.FullName }

Write-Host "Using MinGW at: $mingwBin"

$env:PATH = "$mingwBin;$($env:PATH)"
$env:CARGO_TARGET_I586_PC_WINDOWS_GNU_LINKER = "i686-w64-mingw32-gcc"
$env:CC_i586_pc_windows_gnu  = "i686-w64-mingw32-gcc"
$env:AR_i586_pc_windows_gnu  = "i686-w64-mingw32-ar"
$env:RUSTFLAGS = "-L $mingwLib -L $gccLib"

cargo +nightly build `
    -Z json-target-spec `
    -Z build-std=std,panic_abort,core,alloc `
    --target build-config/i586-pc-windows-gnu.json `
    --features gui `
    --bin rustid-gui `
    --release

if ($LASTEXITCODE -eq 0) {
    if (-not (Test-Path "target\dist")) {
        New-Item -ItemType Directory -Path "target\dist" | Out-Null
    }
    if (Test-Path "target\i586-pc-windows-gnu\release\rustid-gui.exe") {
        Copy-Item "target\i586-pc-windows-gnu\release\rustid-gui.exe" "target\dist\rustid_x86_32.exe" -Force
    }
}

exit $LASTEXITCODE
