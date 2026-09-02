#!/usr/bin/env pwsh
# Build ARM64 Windows GUI binary using LLVM MinGW/GNU target (aarch64-pc-windows-gnullvm).
# Must be run from the repo root.

param(
    [string]$CrossCompilerDir = $env:CROSS_COMPILER_DIR
)

# Ensure the aarch64 target is installed
if (-not (rustup target list --installed | Select-String "aarch64-pc-windows-gnullvm")) {
    Write-Host "Installing aarch64-pc-windows-gnullvm target..."
    rustup target add aarch64-pc-windows-gnullvm
}

# Find or download the llvm-mingw toolchain
$searchBase = if ($CrossCompilerDir) { $CrossCompilerDir } else {
    Join-Path ([System.IO.Path]::GetTempPath()) "rust-cross-compiler"
}

if (-not (Test-Path $searchBase)) {
    New-Item -ItemType Directory -Path $searchBase | Out-Null
}

$destDir = Join-Path $searchBase "llvm-mingw"
$clangExe = Join-Path $destDir "bin\aarch64-w64-mingw32-clang.exe"

if (-not (Test-Path $clangExe)) {
    # Check if aarch64-w64-mingw32-clang is already available in PATH
    $inPath = Get-Command "aarch64-w64-mingw32-clang" -ErrorAction SilentlyContinue
    if (-not $inPath) {
        Write-Host "Downloading llvm-mingw toolchain for Windows ARM64..."
        $zipPath = Join-Path $searchBase "llvm-mingw.zip"
        Invoke-WebRequest -Uri "https://github.com/mstorsjo/llvm-mingw/releases/download/20241119/llvm-mingw-20241119-ucrt-x86_64.zip" -OutFile $zipPath
        Write-Host "Extracting llvm-mingw..."
        Expand-Archive -Path $zipPath -DestinationPath $searchBase
        $extracted = Get-ChildItem -Path $searchBase -Filter "llvm-mingw-*" -Directory | Where-Object { $_.Name -ne "llvm-mingw" } | Select-Object -First 1
        if ($extracted) {
            Rename-Item -Path $extracted.FullName -NewName "llvm-mingw"
        }
        Remove-Item -Path $zipPath -Force -ErrorAction SilentlyContinue
    }
}

if (Test-Path (Join-Path $destDir "bin")) {
    $binPath = Join-Path $destDir "bin"
    Write-Host "Using LLVM-MinGW at: $binPath"
    $env:PATH = "$binPath;$($env:PATH)"
}

cargo build `
    --target aarch64-pc-windows-gnullvm `
    --features gui `
    --bin rustid-gui `
    --release

if ($LASTEXITCODE -eq 0) {
    if (-not (Test-Path "target\dist")) {
        New-Item -ItemType Directory -Path "target\dist" | Out-Null
    }
    if (Test-Path "target\aarch64-pc-windows-gnullvm\release\rustid-gui.exe") {
        Copy-Item "target\aarch64-pc-windows-gnullvm\release\rustid-gui.exe" "target\dist\rustid_arm64.exe" -Force
    }
}

exit $LASTEXITCODE
