# Development Setup

## Prerequisites (development)
- **Rust Toolchain**: `rustup` and `cargo`. Nightly toolchain with `rust-src` component is required for DOS builds (`-Z build-std`).
- **`just` or `make`**: Task runner to execute build scripts. Install with `cargo install just` or use `make`.
- **DOSBox-X**: **Required** for building `rustid.exe` (invokes `dos32a.exe` inside DOSBox-X to bind the Linear Executable `.le` payload) and running/testing DOS binaries (`just run-dos` / `just test-dos`).
- **`cargo-cross`** (optional): Used for cross-compiling target architectures (`just build-arm64`, `just build-ppc`, `just test-arm`, etc.).

## Building

**Standard Release Build:**
```bash
just build-release
# or: make build-release
```

**Build for DOS:**
```bash
just build-dos
# or: make build-dos
```
This produces three binaries in the project root:
- `rustid.exe` — 32-bit protected-mode DOS32A binary (`dos_rustid` cargo binary)
- `rust86.exe` — 16-bit real-mode fallback binary (`rust86` cargo binary)
- `debug86.exe` — 16-bit real-mode debug binary (`debug86` cargo binary)

**Individual DOS Builds:**
- `just build-dos32a` — Build 32-bit DOS32A extender binary (`rustid.exe`)
- `just build-dos-real` — Build 16-bit real-mode binaries (`rust86.exe`, `debug86.exe`)

**Cross-Compilation & Platform Builds:**
- `just build-arm64` — Linux AArch64
- `just build-ppc` — Linux PowerPC
- `just build-486` — 32-bit x86 Linux (486-compatible target spec)
- `just build-mac` / `just build-mac-arm` — macOS (x86_64 / AArch64)
- `just build-windows` / `just build-windows-arm` / `just build-windows-gnu` — Windows targets

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

Launch DOS build in DOSBox-X:
```bash
just run-dos
```