# Rustid

A lightweight CPU identification tool for Windows, Linux, DOS, and UEFI/EFI. `rustid` queries processor information using the `CPUID` instruction and maps it to specific microarchitectures and feature sets. There is also support for ARM, RISC V, and PowerPC cpu detection.

### AI Disclaimer:

This application is developed using *some* AI, mostly related to:

* DOS Build
* Assembly code

## Features
- **Multi-Architecture Support:** Detects CPUs on x86/x86_64, ARM/AArch64, Risc V and PowerPC.
- **Vendor & Model Detection:** Identifies CPUs from Intel, AMD, Cyrix, VIA, Zhaoxin, Rise, Transmeta, Apple Silicon, Qualcomm, and more.
- **Feature Flag Reporting (x86):** Detects support for FPU, MMX, SSE (up to 4.2), AVX, AVX-512, BMI, and others.
- **Cache & Topology Info:** Displays cache sizes, associativity, core/thread counts, and socket counts.
- **DOS Compatibility:** Compiles to a single binary that can be run on DOS environments (on real hardware 386-class or better, or with DOSBox/DOSBox-X).
- **UEFI Compatibility:** Compiles to a standalone UEFI application (32-bit and 64-bit x86) with zero external dependencies.

## Platform Support

### Tier 1

Primary platforms, with the majority of testing effort.

|               | DOS    | EFI    | Windows | macOS  | Linux  | Haiku  |
|--------------:|:------:|:------:|:-------:|:------:|:------:|:------:|
| **x86_64**    | —      | ✅     | ✅ | ✅ | 🟢 | ✅     |
| **x86_32**    | ✅[¹](#note-1) | ✅ | ✅ | — | 🟢 | ✅ |
| **ARM 64**    | —      | —      | ⚠️[⁵](#note-5) | ✅ | 🟢 | ❌ |
| **ARM 32**    | —      | —      | —       | —      | ✅     | —      |
| **RISC-V 64** | —      | —      | —       | —      | ✅     | —      |
| **PowerPC**   | —      | —      | —       | —      | ✅     | —      |
| **PowerPC 64**| —      | —      | —       | —      | ⚠️[⁴](#note-4) | — |

### Tier 2

These are best-effort platforms: they should work, but information may be more limited and/or less correct.

|               | FreeBSD | NetBSD | OpenBSD | Android |
|--------------:|:-------:|:------:|:-------:|:-------:|
| **x86_64**    | ✅      | ✅     | ✅      | ⚠️[²](#note-2) |
| **x86_32**    | ✅      | ✅     | ✅      | ⚠️[²](#note-2) |
| **ARM 64**    | ✅      | ✅     | ✅      | ⚠️[²](#note-2) |
| **ARM 32**    | ⚠️[³](#note-3) | ⚠️[³](#note-3) | ⚠️[³](#note-3) | ⚠️[²](#note-2) |
| **RISC-V 64** | ❌      | ❌     | ❌      | —       |
| **PowerPC**   | ❌      | ❌     | ❌      | —       |
| **PowerPC 64**| ❌      | ❌     | ❌      | —       |

**Legend:**
- 🟢 CI-tested (`just test-all` on `ubuntu-latest`)
- ✅ Supported
- ⚠️ Partial
- ❌ Not supported

**Notes:**
- <a id="note-1"></a>¹ DOS: requires 386 or newer CPU
- <a id="note-2"></a>² Android: untested, uses Linux platform logic
- <a id="note-3"></a>³ ARM 32 BSD: panics if MIDR cannot be read from sysctl
- <a id="note-4"></a>⁴ PowerPC 64 Linux: untested
- <a id="note-5"></a>⁵ Windows ARM 64: limited data

## Getting Started

### Installing (DOS)
For DOS, there are binaries on Github for each release.

### Installing (EFI / UEFI)
Copy the EFI binaries (`BOOTX64.EFI` for 64-bit, `BOOTIA32.EFI` for 32-bit) to the `EFI/BOOT` directory of the EFI System Partition. They can also be run from a USB drive.

### Installing (MacOS, Linux, Windows, etc.)
- Rust (`cargo` needs to be installed)
- For most environments, `cargo install rustid` will add `rustid` to your path

## Usage
- For binaries, just run `rustid`, for more commands run `rustid --help`.
- For DOS, the main binary is `rustid.exe`, with debug and cpuid dump functionality in `debug.exe` and `dump.exe` respectively.

## Development
See [DEVELOPMENT.md](./DEVELOPMENT.md)

Output varies by architecture. Here is an example for x86_64:

```text
--------------- Rustid 1.2.0 (x86_64-windows) ---------------
  Architecture: x86_64-v4

        Vendor: AuthenticAMD (AMD)

    Hypervisor: Microsoft Hv (Microsoft HyperV)

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

      Features: Base: FPU TSC CX8 CX16 CMOV MMX MMX+ 3DNow!-Prefetch HT APIC AMD64
                SSE: SSE SSE2 SSE3 SSE4A SSE4.1 SSE4.2 SSSE3
                AVX: AVX AVX2 AVX-VNNI VPCLMULQDQ
                AVX512: F DQ IFMA CD BW VL BITALG VPOPCNTDQ VP2INTERSECT
                Security: NX RDSEED RDRAND AES VAES SHA
                Math: FMA BMI1 BMI2 F16C
                Other: POPCNT

```

For ARM, Risc V, and PowerPC, the output includes different fields (e.g., brand/implementor, codename, cache per core type).

## Information References

- [sandpile.org](https://sandpile.org/x86/cpuid.htm) - One of the best known x86 references
- [cpufetch](https://github.com/Dr-Noob/cpufetch) (a similar tool that might work better for you)
- [x86-cpuid-db](https://gitlab.com/x86-cpuid.org/x86-cpuid-db) - good reference of various cpuid information leaves
- [cpuid visualizer](https://cpuid.apps.poly.nomial.co.uk/) - helpful for mapping cpu signatures from other sources
- [CPU-World](https://www.cpu-world.com/index.html)
- [My own hardware collection](https://timshome.page/collection/cpu)

