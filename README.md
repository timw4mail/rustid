# Rustid

A lightweight CPU identification tool for Windows, Linux, and DOS. `rustid` queries processor information using the `CPUID` instruction and maps it to specific microarchitectures and feature sets.

**AI Disclaimer**: This application is developed using some AI, mostly to help with the build environment for dos, and the original code commit.

## Features

- **Vendor & Model Detection:** Identifies CPUs from Intel, AMD, Cyrix, VIA, Zhaoxin, and more.
- **Feature Flag Reporting:** Detects support for FPU, MMX, SSE (up to 4.2), AVX, AVX-512, BMI, and others.
- **DOS Compatibility:** Compiles to a single binary that can be run on DOS environments (like DOSBox-X).

## Project Structure

- `src/cpuid/`: Core logic for CPUID instruction wrappers and data parsing.
- `src/dos.rs`: DOS-specific I/O and entry points.
- `src/main.rs`: CLI application entry point.

## Getting Started

### Prerequisites

- Rust (`rustup` and `cargo` need to be installed)
- `just` - Required to run build scripts. Can be installed with `cargo install just`.
- Dosbox-X (optional) - Helpful for development and testing of the DOS version

### Building

**Standard Build:**
```bash
cargo build --release
```

**Build for DOS:**
```bash
just build-dos
```
This produces a `rustid.com` binary compatible with DOS environments (like DOSBox-X).

## Usage

Simply run the compiled binary to see your CPU details:

```bash
cargo run
```

Example Output:
```text
CPU Vendor:    AuthenticAMD (AMD)
CPU Name:      AMD Ryzen 9 5950X 16-Core Processor
CPU Codename:  Vermeer
CPU Signature: Family 25, Model 33, Stepping 2
               (10, 15, 2, 1, 2)
Logical Cores: 32
Features:
  FPU:      true
  ...
```

## Information References

- [sandpile.org](https://sandpile.org/x86/cpuid.htm) - One of the best known x86 references
- [cpufetch](https://github.com/Dr-Noob/cpufetch) (a similar tool that might work better for you)
- [Paradice CPUID Guide](https://www.paradicesoftware.com/specs/cpuid/) - helpful for Cyrix workarounds
- [CPU-World](https://www.cpu-world.com/index.html)
- [My own hardware collection](https://timshome.page/collection/cpu)

