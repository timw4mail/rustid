#!/usr/bin/env pwsh
# Build 32-bit Windows GUI targeting original Pentium (no SSE/SSE2).
# Must be run from the repo root. Requires nightly toolchain.

param(
    [string]$Action = "build",
    [string]$CrossCompilerDir = $env:CROSS_COMPILER_DIR
)

# Ensure the i686 target is installed (rustc needs it for target spec)
if (-not (rustup target list --installed | Select-String "i686-pc-windows-gnu")) {
    Write-Host "Installing i686-pc-windows-gnu target..."
    rustup target add i686-pc-windows-gnu
}

# Ensure nightly rust-src is installed for -Z build-std
if (-not (rustup component list --installed --toolchain nightly | Select-String "rust-src")) {
    Write-Host "Installing rust-src for nightly..."
    rustup component add rust-src --toolchain nightly
}

# Find the MinGW-w64 toolchain
$searchBase = if ($CrossCompilerDir) { $CrossCompilerDir } else {
    Join-Path ([System.IO.Path]::GetTempPath()) "rust-cross-compiler"
}

if (-not (Test-Path $searchBase)) {
    New-Item -ItemType Directory -Path $searchBase | Out-Null
}

$mingwDir = Get-ChildItem -Path $searchBase -Filter "i686-w64-mingw32*" -Directory -ErrorAction SilentlyContinue |
    Where-Object { Test-Path (Join-Path $_.FullName "bin\dlltool.exe") -or Test-Path (Join-Path $_.FullName "bin\i686-w64-mingw32-gcc.exe") } |
    Select-Object -First 1

if (-not $mingwDir) {
    # Check if i686-w64-mingw32-gcc is already available in PATH
    $inPath = Get-Command "i686-w64-mingw32-gcc" -ErrorAction SilentlyContinue
    if ($inPath) {
        $binDir = Split-Path -Parent $inPath.Source
        $rootDir = Split-Path -Parent $binDir
        $mingwDir = Get-Item $rootDir
    }
}

if (-not $mingwDir) {
    Write-Error "Could not find MinGW-w64 toolchain in $searchBase. Set CROSS_COMPILER_DIR or ensure i686-w64-mingw32-gcc is in PATH."
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
$env:WINDRES                 = "i686-w64-mingw32-windres"
$env:RUSTFLAGS = "-L $mingwLib -L $gccLib"

cargo +nightly $Action `
    -Z json-target-spec `
    -Z build-std=std,panic_abort,core,alloc `
    --target build-config/i586-pc-windows-gnu.json `
    --features gui `
    --bin gui `
    --release

if ($LASTEXITCODE -eq 0 -and $Action -eq "build") {
    if (-not (Test-Path "target\dist\windows")) {
        New-Item -ItemType Directory -Path "target\dist\windows" -Force | Out-Null
    }
    if (Test-Path "target\i586-pc-windows-gnu\release\gui.exe") {
        Copy-Item "target\i586-pc-windows-gnu\release\gui.exe" "target\dist\windows\rustid_x86_32.exe" -Force
    }
}

exit $LASTEXITCODE
