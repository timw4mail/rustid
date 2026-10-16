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

All build recipes copy their final binaries into platform-specific subdirectories under `target/dist/`:
- `target/dist/linux/` — Linux CLI, GUI, and AppImage binaries
- `target/dist/windows/` — Windows GUI / CLI executables
- `target/dist/mac/` — macOS CLI binaries
- `target/dist/classic_mac/` — Classic Mac OS & Mac OS X PowerPC binaries/bundles
- `target/dist/dos/` — DOS 32-bit extender and 16-bit real-mode binaries
- `target/dist/efi/` — UEFI 32-bit and 64-bit boot binaries
- `target/dist/haiku/` — Haiku GUI / CLI binaries

**Standard Release Build:**
```bash
just build-release
# or: make build-release
```
Produces `target/dist/<os>/rustid` (or `rustid.exe` on Windows).

**Build for EFI / UEFI (x86 & x86_64):**
```bash
just build-efi
# or: make build-efi
```
This produces EFI binaries:
- `target/dist/efi/EFI/BOOT/BOOTX64.EFI` (64-bit EFI)
- `target/dist/efi/EFI/BOOT/BOOTIA32.EFI` (32-bit EFI)

Individual EFI Builds:
- `just build-efi-64` — 64-bit x86_64 EFI binary
- `just build-efi-32` — 32-bit x86 EFI binary

**Build for DOS:**
```bash
just build-dos
# or: make build-dos
```
This produces two binaries in `target/dist/dos/`:
- `target/dist/dos/rustid.exe` — 32-bit protected-mode DOS32A binary
- `target/dist/dos/rust86.exe` — 16-bit real-mode fallback binary with debug support

**Individual DOS Builds:**
- `just build-dos32a` — Build 32-bit DOS32A extender binary (`target/dist/dos/rustid.exe`)
- `just build-dos-real` — Build 16-bit real-mode binary (`target/dist/dos/rust86.exe`)

**Cross-Compilation & Platform Builds:**
- `just build-arm64` — Linux AArch64 (`target/dist/linux/rustid_arm64`)
- `just build-ppc` — Linux PowerPC (`target/dist/linux/rustid_ppc`)
- `just build-486` — Static 32-bit x86 Linux musl for 486-class CPUs (`target/dist/linux/rustid_486`)
- `just build-mac` / `just build-mac-arm` — macOS (`target/dist/mac/rustid_x86_64` / `rustid_arm64`)
- `just build-classic-mac` — Classic Mac OS 68k/PPC & Mac OS X PPC (`target/dist/classic_mac/`)
- `just build-windows-gui` — All Windows GUI applications (`target/dist/windows/rustid_x86_32.exe`, `rustid_x86_64.exe`, `rustid_arm64.exe`)
- `just build-linux-gui` / `just build-appimage` — Linux GTK3 GUI & standalone AppImage (`target/dist/linux/rustid-gui`, `rustid-<arch>.AppImage`)
- `just build-haiku-gui` — Haiku native GUI application (`target/dist/haiku/rustid_haiku`)

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

## Code Style & Architectural Guidelines

To maintain readability, modularity, and consistency across platforms, adhere to the following conventions:

### 1. Platform Constants over Runtime Boolean Flags
- Prefer compile-time `#[cfg(...)]` constants (e.g., `NEWLINE`) over plumbing runtime boolean flags (e.g., `crlf: bool`) through function signatures when behavior is determined solely by the target platform.

### 2. Pass Cohesive Structs over Individual Flags
- Pass structured configuration types (e.g., `flags: CliFlags`) instead of separate boolean flags (`verbose: bool`, `compact: bool`, `color: bool`).

### 3. Use Domain Enums Instead of Booleans
- Use explicit enums with descriptive variants (e.g., `GuiTheme::Light` / `GuiTheme::Dark`, `ReportSource::LiveHardware` / `ReportSource::DumpFile`) for function parameters instead of `bool` flags.

### 4. Direct Common Calls over Forwarding Wrappers
- Invoke shared functions from common modules (such as `crate::gui::common::*`) directly at call sites rather than creating redundant 1-line forwarding wrappers in platform-specific files.

### 5. Concise Iterations & Utility Reuse
- Unify repetitive multi-register processing with array loops (e.g., iterating `[res.eax, res.ebx, res.ecx, res.edx]`).
- Use iterator chains (`.filter_map()`, `.collect()`) for feature aggregation.
- Reuse existing string/formatting helpers in `crate::common::util` (such as `ucfirst`) across architecture modules.