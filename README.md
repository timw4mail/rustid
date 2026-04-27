# Rustid

A lightweight CPU identification tool for Windows, Linux, and DOS. `rustid` queries processor information using the `CPUID` instruction and maps it to specific microarchitectures and feature sets. There is also some support for ARM and PowerPC cpu detection.

### AI Disclaimer:

This application is developed using *some* AI, mostly related to:

* DOS Build
* Assembly code

## Features

- **Multi-Architecture Support:** Detects CPUs on x86/x86_64, ARM/AArch64, and PowerPC.
- **Vendor & Model Detection:** Identifies CPUs from Intel, AMD, Cyrix, VIA, Zhaoxin, Rise, Transmeta, Apple Silicon, Qualcomm, and more.
- **Feature Flag Reporting (x86):** Detects support for FPU, MMX, SSE (up to 4.2), AVX, AVX-512, BMI, and others.
- **Cache & Topology Info:** Displays cache sizes, associativity, core/thread counts, and socket counts.
- **DOS Compatibility:** Compiles to a single binary that can be run on DOS environments (on real hardware 386-class or better, or with DOSBox/DOSBox-X).

## Getting Started

### Prerequisites

- Rust (`rustup` and `cargo` need to be installed)
- `just` - Required to run build scripts. Can be installed with `cargo install just`.
- DOSBox-X (optional) - Helpful for development and testing of the DOS version

### Building

**Standard Build:**
```bash
just build-release
```

**Build for DOS:**
```bash
just build-dos
```
This produces a `rustid.com` binary compatible with DOS environments (like DOSBox-X).

**Cross-Compilation:**
For other architectures, see the `justfile` for available targets (`just build-arm64`, `just build-ppc`, etc). Cross-compilation should be considered experimental.

## Usage

Simply run the compiled binary to see your CPU details:

```bash
just run
```
or
```bash
cargo run
```

Output varies by architecture. Here is an example for x86_64:

```text
--------------- Rustid 1.0.0 (x86_64-windows) ---------------
  Architecture: x86_64-v4

        Vendor: AuthenticAMD (AMD)

         Model: AMD Ryzen 9 9950X3D2 16-Core Processor

     MicroArch: Zen 5

      Codename: Granite Ridge

  Process Node: 4nm

      Topology: 16 cores (32 threads)

         Cache: L1d: 16x 48 KB, 12-way
                L1i: 16x 32 KB, 8-way
                L2:  16x 1 MB, 16-way
                L3:  2x 96 MB, 16-way

     Frequency: 4.29 GHz

     Signature: Family 1Ah, Model 44h, Stepping 0h
                (11, 15, 4, 4, 0)

      Features: FPU TSC CMPXCHG8B CMPXCHG16B CMOV MMX HT AMD64 SSE SSE2 SSE3 SSE4A SSE4.1 SSE4.2 SSSE3 AES VAES AVX AVX2 AVX512F FMA BMI1 BMI2 RDRAND POPCNT F16C SHA
```

For ARM and PowerPC, the output includes different fields (e.g., brand/implementor, codename, cache per core type).

## Information References

- [sandpile.org](https://sandpile.org/x86/cpuid.htm) - One of the best known x86 references
- [cpufetch](https://github.com/Dr-Noob/cpufetch) (a similar tool that might work better for you)
- [x86-cpuid-db](https://gitlab.com/x86-cpuid.org/x86-cpuid-db) - good reference of various cpuid information leaves
- [cpuid visualizer](https://cpuid.apps.poly.nomial.co.uk/) - helpful for mapping cpu signatures from other sources
- [CPU-World](https://www.cpu-world.com/index.html)
- [My own hardware collection](https://timshome.page/collection/cpu)

