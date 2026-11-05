# Rustid

A CLI and GUI CPU identification tool for ARM, RISC-V, PowerPC, Motorola 68k, and x86/x86_64 CPUs.

### AI Disclaimer:

This application is developed using *some* AI, mostly related to:

* DOS Build
* Assembly code

## Features
- **Multi-Architecture Support:** Detects CPUs on x86/x86_64, ARM/AArch64, RISC-V, PowerPC, and Motorola 68k.
- **Vendor & Model Detection:** Identifies CPUs from Intel, AMD, Cyrix, VIA, Zhaoxin, Rise, Transmeta, Apple Silicon, Qualcomm, IBM, Motorola, and more.
- **Feature Flag Reporting (x86):** Detects support for FPU, MMX, SSE (up to 4.2), AVX, AVX-512, BMI, and others.
- **Cache & Topology Info:** Displays cache sizes, associativity, core/thread counts, and socket counts.
- **DOS Compatibility:** Compiles to a set of binaries that can be run on DOS environments (on real hardware 386-class or better, or with DOSBox/DOSBox-X).
- **UEFI Compatibility:** Compiles to a standalone UEFI application (32-bit and 64-bit x86)
- **Native GUI Support:** Lightweight Win32 GUI application for Windows (Windows 95 through 11), GTK3 GUI and standalone AppImage for Linux, and native Interface Kit application for Haiku OS.
- **Classic Macintosh Port:** Standalone native C port using the Macintosh Toolbox for System 6 through Mac OS 9.2.2 (68k and PowerPC) and Carbon/Aqua for Mac OS X 10.0–10.5 (PowerPC). See [Classic Mac Port](ports/classic_mac/README.md).

## Platform Support

### CLI — Tier 1

Primary CLI and standalone boot/real-mode platforms, with the majority of testing effort.

|               | DOS            | EFI | Windows        | macOS | Linux          | Haiku |
|--------------:|:--------------:|:---:|:--------------:|:-----:|:--------------:|:-----:|
| **x86_64**    | —              | 🟢  | ✅             | ✅    | 🟢             | ✅    |
| **x86_32**    | 🟢[¹](#note-1) | 🟢  | ✅[⁶](#note-6) | —     | 🟢[⁷](#note-7) | ✅    |
| **ARM 64**    | —              | —   | ✅[⁵](#note-5) | ✅    | 🟢             | ❌    |
| **ARM 32**    | —              | —   | —              | —     | ✅             | —     |
| **RISC-V 64** | —              | —   | —              | —     | ✅             | —     |
| **PowerPC**   | —              | —   | —              | —     | ✅             | —     |
| **PowerPC 64**| —              | —   | —              | —     | ⚠️[⁴](#note-4) | —     |

### CLI — Tier 2

Best-effort CLI platforms: they should work, but information may be more limited and/or less tested.

|               | FreeBSD        | NetBSD         | OpenBSD        | Android        |
|--------------:|:--------------:|:--------------:|:--------------:|:--------------:|
| **x86_64**    | ✅             | ✅             | ✅             | ⚠️[²](#note-2) |
| **x86_32**    | ✅             | ✅             | ✅             | ❌             |
| **ARM 64**    | ✅             | ✅             | ✅             | ✅[²](#note-2) |
| **ARM 32**    | ⚠️[³](#note-3) | ⚠️[³](#note-3) | ⚠️[³](#note-3) | ⚠️[²](#note-2) |
| **RISC-V 64** | ❌             | ❌             | ❌             | —              |
| **PowerPC**   | ❌             | ❌             | ❌             | —              |
| **PowerPC 64**| ❌             | ❌             | ❌             | —              |

### GUI

Native graphical applications built for each target platform (`rustid-gui` and the standalone Classic Mac port).

|                  | Windows (Win32) | Classic Mac OS / OS X PPC | Linux (GTK3)    | Haiku (Interface Kit) |
|-----------------:|:---------------:|:-------------------------:|:---------------:|:---------------------:|
| **x86_64**       | 🟢              | —                         | 🟢[⁹](#note-9)  | ✅                    |
| **x86_32**       | 🟢[⁶](#note-6)  | —                         | ✅              | ✅                    |
| **ARM 64**       | 🟢[⁵](#note-5)  | —                         | ✅              | ❌                    |
| **ARM 32**       | —               | —                         | ✅              | —                     |
| **RISC-V 64**    | —               | —                         | ✅              | —                     |
| **PowerPC**      | —               | ✅[⁸](#note-8)            | ✅              | —                     |
| **PowerPC 64**   | —               | ✅[⁸](#note-8)            | ⚠️[⁴](#note-4)  | —                     |
| **Motorola 68k** | —               | ✅[⁸](#note-8)            | —               | —                     |

**Legend:**
- 🟢 Supported and CI-tested / CI-built
- ✅ Supported
- ⚠️ Partial / untested
- ❌ Not supported

**Notes:**
- <a id="note-1"></a>¹ **DOS**: Includes both a 32-bit protected-mode DOS/32A executable (`rustid.exe`) and a 16-bit real-mode fallback executable (`rust86.exe`). Requires a 386 or newer CPU. See [DOS.md](DOS.md).
- <a id="note-2"></a>² **Android**: Runs via Termux. Untested except on ARM 64 (compile-checked in CI).
- <a id="note-3"></a>³ **ARM 32 BSD**: Less tested than ARM 64.
- <a id="note-4"></a>⁴ **PowerPC 64 Linux**: Untested.
- <a id="note-5"></a>⁵ **Windows ARM 64**: Limited hardware data; also supports `arm64ec`.
- <a id="note-6"></a>⁶ **Windows x86_32**: Built with an original Pentium (`i586`, no SSE/SSE2) baseline and dynamic Win32 API shims; compatible with Windows 95/98/ME and NT 4.0/2000/XP through Windows 11.
- <a id="note-7"></a>⁷ **Linux x86_32**: Includes a static `i486-unknown-linux-musl` build target (`just build-486`) supporting 486-class CPUs.
- <a id="note-8"></a>⁸ **Classic Mac OS & Mac OS X PowerPC**: Provided by the standalone C port in [`ports/classic_mac`](ports/classic_mac/README.md). Ships as a Universal Fat Binary (`rustid.bin` / `rustid.dsk`) for System 6.0.8–Mac OS 9.2.2 (Motorola 68000–68060 and PowerPC 601–G4) and a native Aqua Carbon bundle (`Rustid.app`) for Mac OS X 10.0–10.5 (PowerPC G3/G4/G5).
- <a id="note-9"></a>⁹ **Linux (GTK3)**: Available as a standalone `x86_64` AppImage in releases, or built from source with `--features gui --bin gui` on any supported Linux architecture.

## Getting Started

### Installing (DOS)
Extract the binaries from the release into the same folder on DOS.

### Installing (EFI / UEFI)
Copy the EFI binaries from the Github release (`BOOTX64.EFI` for 64-bit, `BOOTIA32.EFI` for 32-bit) to the `EFI/BOOT` directory of the EFI System Partition. They can also be run from a USB drive.

### Installing (MacOS, Linux, Windows, etc.)
- Rust (`cargo` needs to be installed)
- For most environments, `cargo install rustid` will add `rustid` to your path
- Rust on Windows can be slightly more complicated, see [this guide](https://rust-lang.github.io/rustup/installation/windows-msvc.html) for steps to install the MSVC compiler and libraries.

### Installing (Classic Mac OS)
See the [Classic Mac Port documentation](ports/classic_mac/README.md) for details on running the Universal Fat binary or standalone 68k / PowerPC binaries on vintage hardware or emulators.

## Usage
- For binaries, just run `rustid`, for more commands run `rustid --help`.

Output varies by architecture. Here is an example for x86_64:

```text
--------------- Rustid 2.0.0 (x86_64-linux) ---------------
        System: N7 B650E

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

      Features: Base: FPU TSC CX8 CX16 CMOV MMX MMX+ 3DNow!-Prefetch HT APIC AMD64
                SSE: SSE SSE2 SSE3 SSE4A SSE4.1 SSE4.2 SSSE3
                AVX: AVX AVX2 AVX-VNNI VPCLMULQDQ
                AVX512: F DQ IFMA CD BW VL BITALG VPOPCNTDQ VP2INTERSECT
                Security: NX RDSEED RDRAND AES VAES SHA AMD-V
                Math: FMA BMI1 BMI2 F16C
                Other: POPCNT
```

See the [examples](./examples) folder for other examples

## Development
See [DEVELOPMENT.md](./DEVELOPMENT.md)

## Information References

- [sandpile.org](https://sandpile.org/x86/cpuid.htm) - One of the best known x86 references
- [cpufetch](https://github.com/Dr-Noob/cpufetch) (a similar tool that might work better for you)
- [x86-cpuid-db](https://gitlab.com/x86-cpuid.org/x86-cpuid-db) - good reference of various cpuid information leaves
- [cpuid visualizer](https://cpuid.apps.poly.nomial.co.uk/) - helpful for mapping cpu signatures from other sources
- [CPU-World](https://www.cpu-world.com/index.html)
- [My own hardware collection](https://timshome.page/collection/cpu)

