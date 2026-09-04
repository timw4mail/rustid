# Development Setup

Note: Currently targets Rust 1.94.1, as that is the latest version on Haiku

## Prerequisites (development)
- **Rust Toolchain**: `rustup` and `cargo`. Nightly toolchain with `rust-src` component is required for DOS builds (`-Z build-std`).
- **`just` or `make`**: Task runner to execute build scripts. Install with `cargo install just` or use `make`.
- **DOSBox-X**: **Required** for building `rustid.exe` and running/testing DOS binaries.
- **QEMU & OVMF** (optional): Used for running and testing UEFI binaries (`just run-efi-64` / `just run-efi-32`).

## Platform-specific notes

### DOS
* DOS builds require `dosbox-x`
* Compilation target only

### Haiku
* Haiku is not supported by rustup, so there is no native cross-compile support
* Haiku does have `cargo`
* Use `make` instead of `just`
* Rust has to be installed from `HaikuDepot`, or via the Terminal:
```bash
pkgman install rust_bin
```

## Building

**Standard Release Build:**
```bash
just build-release
# or: make build-release
```

**Build for EFI / UEFI (x86 & x86_64):**
```bash
just build-efi
# or: make build-efi
```
This produces EFI binaries:
- `target/efi-disk/EFI/BOOT/BOOTX64.EFI` (64-bit EFI)
- `target/efi-disk/EFI/BOOT/BOOTIA32.EFI` (32-bit EFI)

Individual EFI Builds:
- `just build-efi-64` — 64-bit x86_64 EFI binary
- `just build-efi-32` — 32-bit x86 EFI binary

**Build for DOS:**
```bash
just build-dos
# or: make build-dos
```
This produces two binaries in the project root:
- `rustid.exe` — 32-bit protected-mode DOS32A binary (`dos` cargo binary)
- `rust86.exe` — 16-bit real-mode fallback binary with debug support (`rust86` cargo binary)

**Individual DOS Builds:**
- `just build-dos32a` — Build 32-bit DOS32A extender binary (`rustid.exe`)
- `just build-dos-real` — Build 16-bit real-mode binary (`rust86.exe`)

**Cross-Compilation & Platform Builds:**
- `just build-arm64` — Linux AArch64
- `just build-ppc` — Linux PowerPC
- `just build-486` — 32-bit x86 Linux (486-compatible target spec)
- `just build-mac` / `just build-mac-arm` — macOS (x86_64 / AArch64)
- `just build-windows` / `just build-windows-arm` / `just build-windows-gnu` — Windows CLI binaries
- `just build-windows-gui` — Windows native GUI application (MSVC)
- `just build-windows-gui-gnu` — Windows GUI application cross-compilable from Linux (MinGW/GNU)

## Testing & Quality

**Run Native Unit Tests:**
```bash
just test
# or: cargo test
```

**Run All Cross/Architecture Tests:**
```bash
just test-all
# tests native, x86 (i686-musl), and arm (aarch64-musl)
```

**Code Formatting & Linting:**
```bash
just quality
# runs fmt, check, and clippy
```

## Running

Run the native binary interactively:
```bash
just run
# or: cargo run
```

Run with CPUID dump input file:
```bash
just from-file -- path/to/dump.txt
```
or
```bash
cargo run -- -f path/to/dump.txt
```

Launch 64-bit EFI binary in QEMU:
```bash
just run-efi-64
```

Launch 32-bit EFI binary in QEMU:
```bash
just run-efi-32
```

Launch DOS build in DOSBox-X:
```bash
just run-dos
```