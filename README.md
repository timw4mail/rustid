# Rustid

A lightweight CPU identification tool for Windows, Linux, and DOS. `rustid` queries processor information using the `CPUID` instruction and maps it to specific microarchitectures and feature sets.

**AI Disclaimer**: This application is developed using some AI, mostly to help with the build environment for DOS, and the original code commit.

## Features

- **Vendor & Model Detection:** Identifies CPUs from Intel, AMD, Cyrix, VIA, Zhaoxin, Rise, Transmeta, and more.
- **Feature Flag Reporting:** Detects support for FPU, MMX, SSE (up to 4.2), AVX, AVX-512, BMI, and others.
- **DOS Compatibility:** Compiles to a single binary that can be run on DOS environments (on real hardware 386-class or better, or with DOSBox/DOSBox-X).

## Getting Started

### Prerequisites

- Rust (`rustup` and `cargo` need to be installed)
- `just` - Required to run build scripts. Can be installed with `cargo install just`.
- DOSBox-X (optional) - Helpful for development and testing of the DOS version

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
---------------------
Rustid version 0.5.1
---------------------

  Architecture: x86_64_v4

        Vendor: AuthenticAMD (AMD)

         Model: AMD Ryzen 9 7950X3D 16-Core Processor

     MicroArch: Zen 4

      Codename: Raphael

          Node: 5nm

         Speed: 4.19 GHz

     Signature: Family 19h, Model 61h, Stepping 2h
                (10, 15, 6, 1, 2)

      Features: FPU TSC CMPXCHG8B CMPXCHG16B CMOV MMX HT SSE SSE2 SSE3 SSE4.1 SSE4.2 SSSE3 AVX AVX2 AVX512F FMA BMI1 BMI2 RDRAND POPCNT F16C
```

## Information References

- [sandpile.org](https://sandpile.org/x86/cpuid.htm) - One of the best known x86 references
- [cpufetch](https://github.com/Dr-Noob/cpufetch) (a similar tool that might work better for you)
- [Paradice CPUID Guide](https://www.paradicesoftware.com/specs/cpuid/) - helpful for Cyrix workarounds
- [x86-cpuid-db](https://gitlab.com/x86-cpuid.org/x86-cpuid-db) - good reference of various cpuid information leaves
- [cpuid visualizer](https://cpuid.apps.poly.nomial.co.uk/) - helpful for mapping cpu signatures from other sources
- [CPU-World](https://www.cpu-world.com/index.html)
- [My own hardware collection](https://timshome.page/collection/cpu)

