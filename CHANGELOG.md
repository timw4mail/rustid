# Changelog

For changes prior to version 2.0.0, see [CHANGELOG-1.0.md](./CHANGELOG-1.0.md).

## [2.3.0] — Classic Macintosh GUI port (68k & PowerPC), AMD K8 L2 cache improvements, and expanded CPUID test fixtures

### Added
- **Classic Macintosh Port (68k & PowerPC)**: Added standalone C-based Classic Macintosh GUI application (`ports/classic_mac/*`, `build-config/classic_mac/*`) supporting System 6.0.8 through Mac OS 9.2.2 (as well as Mac OS X Carbon/Aqua) on Motorola 680x0 (68000–68060) and PowerPC (601–G5), featuring Gestalt hardware probing, resizable and scrollable windows, styled text rendering, clipboard copy, and universal fat binary packaging
- **PowerPC Model & Microarchitecture Detection**: Added PowerPC 7410 (Nitro) PVR `0x800C` recognition and "Vger" codename designation for PowerPC 7450 in the Rust library (`src/ppc/micro_arch.rs`), alongside comprehensive vintage Mac PowerPC CPU and machine model tables in the Classic Mac engine (`ports/classic_mac/classic_mac_engine.c`)
- **AMD Athlon 64 X2 6400+ CPUID Dump & Test Suite**: Added CPUID dump fixture and integration test suite covering signature, topology, per-core L1/L2 cache sizing & associativity, x86_64-v1 feature class, 3DNow!, and K8 Windsor core detection (`tests/cpuid/dump/Amd_Athlon(tm)_64_X2_6400+.txt`, `tests/cpuid_dump_test.rs`)
- **Intel Core i7-2675QM CPUID Dump**: Added 2011 MacBook Pro Sandy Bridge CPUID dump test fixture (`tests/cpuid/dump/Intel_Core_i7_2675QM`)
- **Classic Macintosh Application Icons**: Added multi-depth icon suite (1-bit, 4-bit, and 8-bit large and small icons, dialog icons, and modern `.icns`) for Classic Mac OS and Carbon OS X targets (`build-config/classic_mac/rustid_icons.r`, `build-config/classic_mac/generate_icns.py`, `build-config/classic_mac/setup_bundle_icon.py`)

### Changed
- **AMD L2 Cache Share-Count Fallback**: Adjusted default L2 cache share count for AMD processors that omit explicit CPUID cache sharing information, ensuring dual-core K8 processors report dedicated per-core L2 caches (`src/common/cache.rs`)
- **Ports Directory Organization**: Reorganized non-Rust platform builds into dedicated `ports/` directory (`ports/classic_mac/*`, `build-config/classic_mac/Makefile`)
- **Feature Flag HTT Display**: Standardized Hyper-Threading / multi-threading feature flag label to `"HTT"` in feature listings (`src/x86/features.rs`)

### Fixed
- **AppImage Build Script Paths**: Fixed AppImage packaging in `build-config/build-appimage.sh` by ensuring directory references resolve to absolute paths rather than relative paths
- **Classic Mac Window Resizing & Scrolling**: Fixed window redraw, scrolling offset bounds, and control synchronization on classic Macintosh systems (`ports/classic_mac/mac_bridge.c`)

## [2.2.0] — Linux GTK & Haiku GUI applications, Intel Wildcat Lake support, and cross-platform application icons

### Added
- **Linux GTK GUI & AppImage**: Added native Linux GTK3 graphical interface and standalone AppImage packaging (`build-config/build-appimage.sh`, `Makefile`, `justfile`)
- **Native Haiku GUI**: Added native Haiku graphical application with styled output and system theme synchronization (`src/gui/haiku/*`)
- **Intel Wildcat Lake Support**: Added hybrid CPU detection for Intel Wildcat Lake (e.g., Core 3 304) resolving Cougar Cove and Darkmont cores (`src/x86/vendor/intel.rs`)
- **Cross-Platform Application Icons**: Added application icons for Windows (with architecture badges for x86/x64/arm64), macOS, Linux, and Haiku (`assets/*`)

### Changed
- **CPUID Dump & Example Fixture Naming**: Standardized CPUID test dumps and output example filenames using consistent underscore-separated naming (e.g., `Amd_Ryzen_7_2700U.txt`, `Intel_Core_I7_12700H.txt`)
- **Changelog Split**: Split historical changelog entries prior to version 2.0.0 into [`CHANGELOG-1.0.md`](./CHANGELOG-1.0.md)
- **Changelog Streamline**: Cut down verbosity of changelog to user-visible changes to improve readability and relevance

## [2.1.1] — Binary restructuring, DOS topology and compact-mode fixes, and Windows system name detection

### Added
- **DOS Compact Mode**: Added `/C` / `/COMPACT` flag support to 32-bit DOS extender binary (`rustid.exe`) (`src/bin/dos.rs`, `src/common/display.rs`)
- **Bay Trail Test Fixture**: Added Intel Bay Trail N3540 CPUID dump test fixture and assertions (`tests/cpuid/dump/Intel_Pentium_N3540.txt`, `tests/cpuid_dump_test.rs`)

### Changed
- **Binary Source Restructuring**: Reorganized cargo binary sources under `src/bin/` (`gui`, `dos`, `efi`) (`src/bin/*`, `Cargo.toml`)
- **Topology Detection Robustness**: Improved SMT width and package core/thread calculations from CPUID Leaf B/19/1F APIC topology domains (`src/x86/topology.rs`, `src/x86/display.rs`)

### Fixed
- **DOS Console Output**: Fixed console output corruption and redirection in 32-bit DOS, adding clean fallback text output when `/M` / `/MONO` is used or stdout is redirected (`src/x86/dos/mod.rs`, `src/bin/dos.rs`)
- **System and Model Name Detection**: Aligned Windows SMBIOS table and Registry parsing with Linux sysfs detection, improving Apple Mac hardware identification under Windows and consistent vendor prefixing across platforms (`src/common/os/windows.rs`, `src/common/os/common.rs`)
- **Bay Trail Core and Thread Counts**: Fixed spurious core/thread counts on Intel Bay Trail CPUs in DOS (`src/x86/dos/mod.rs`)
- **Sequential Dump Loading**: Fixed issue where loading multi-core dumps affected subsequent dump parsing (`src/x86/provider.rs`)
- **Cross-Platform Build Fixes**: Resolved compile errors across ARM, PowerPC, and macOS targets (`src/arm/os/mod.rs`, `src/ppc/cpu.rs`, `src/common/os/macos.rs`)

## [2.1.0] — Native Windows GUI with Windows 9x backwards compatibility, unified topology model, and multi-socket display

### Added
- **Native Windows GUI**: Added standalone Win32 GUI application (`gui`) supporting Windows 95 through Windows 11 with dynamic ANSI/Unicode and RichEdit fallbacks, status bar, and dump file loading (`src/gui/windows/*`)
- **Windows Cross-Compilation**: Added build scripts and toolchain configurations for 32-bit x86, x86_64, and ARM64 Windows GUI targets (`build-config/*`, `Makefile`, `justfile`)
- **DOS Color Output**: Added text-mode color rendering for 32-bit DOS (`rustid.exe`) with `/M` / `/MONO` monochrome flag support (`src/bin/dos.rs`, `src/x86/dos/mod.rs`)
- **Multi-Socket & Topology Display**: Added multi-socket system detection and formatting across all architectures, including sysfs package detection on Linux/Android and single-core SMT thread formatting (`src/common/topology.rs`, `src/common/display.rs`, `src/common/os/linux_sysfs.rs`)
- **NetBurst Northwood vs. Gallatin Detection**: Disambiguated Pentium 4 from Pentium 4 Extreme Edition / Xeon MP using L3 cache presence (`src/x86/vendor/intel.rs`, `src/common/cache.rs`)
- **Test Fixtures**: Added dual-socket Intel Xeon E5-2470 and Pentium 4 Northwood test fixtures (`examples/Intel_Xeon_E5_2470_V2.txt`, `tests/cpuid/dump/Intel_Pentium_4_Northwood.txt`)

### Changed
- **Consolidated Linux & Android OS Detection**: Unified sysfs traversal, cache discovery, and hardware enrichment across Linux and Android into shared modules (`src/common/os/*`)
- **Architecture Module Alignment**: Structured ARM, RISC-V, and PowerPC modules around shared generic CPU types and unified display patterns (`src/common/cpu.rs`, `src/arm/*`, `src/riscv/*`, `src/ppc/*`)

### Fixed
- **Legacy Windows SMBIOS & Multi-Socket Detection**: Added fallback SMBIOS table parsing and multi-socket detection for Windows 95 through XP (`src/common/os/windows.rs`)
- **PowerPC Clock Speed Detection**: Fixed CPU clock speed reporting on Linux PowerPC systems (`src/ppc/cpu.rs`)
- **DOS Multi-Socket Detection**: Corrected MP Table topology parsing to prevent false multi-socket reports on multi-core processors (`src/x86/dos/mod.rs`)
- **Cross-Platform Build Fixes**: Resolved build and import issues on EFI, Android, and Haiku targets (`src/x86/display.rs`, `src/common/os/haiku.rs`, `src/common/os/android.rs`)

## [2.0.0] — Comprehensive Intel/AMD microarchitecture expansion, ARM SoC detection, UEFI SMBIOS support, and output redesign

### Added
- **Intel Microarchitecture Expansion**: Added comprehensive detection and signature disambiguation for modern and legacy Intel processors (Meteor Lake, Arrow Lake, Lunar Lake, Raptor Lake, Alder Lake, Sapphire Rapids, Emerald Rapids, Granite Rapids, Sierra Forest, Core 2, Xeon, and Pentium families) (`src/x86/vendor/intel.rs`, `src/x86/micro_arch.rs`)
- **AMD Microarchitecture Expansion**: Added comprehensive detection for Zen 1 through Zen 5/5c (Ryzen, Threadripper, EPYC), legacy K5 through K10, Bulldozer families, and asymmetric 3D V-Cache display (`src/x86/vendor/amd.rs`, `src/x86/micro_arch.rs`, `src/x86/cache.rs`)
- **ARM SoC and Core Detection**: Added detection for Apple Silicon, Qualcomm Snapdragon (Oryon/Kryo), Samsung Exynos, and ARM Cortex/Neoverse cores across macOS, Linux, Android, and Windows ARM64 (`src/arm/*`)
- **UEFI SMBIOS & System Detection**: Added SMBIOS 2.x/3.x table parsing to EFI target for system model and clock speed detection (`src/x86/efi/smbios.rs`)
- **OS-Level Cache & Topology Detection**: Added cross-platform OS cache detection and multi-level topology resolution across x86, ARM, PowerPC, and RISC-V (`src/common/cache.rs`, `src/common/os/*`)
- **Test Suite Modernization**: Added macro-driven CPUID dump test suite and test fixtures (`tests/cpuid_dump_test.rs`, `tests/cpuid/dump/*`)

### Changed
- **Output Formatting Alignment**: Aligned ARM and PowerPC display formatting with x86 layout, grouped multi-cluster cores, and formatted friendly Apple Mac marketing names (`src/arm/display.rs`, `src/common/display.rs`)
- **Real-Mode DOS Binary Optimization**: Streamlined real-mode DOS binary (`rust86.exe`) footprint and added CLI argument support (`src/bin/dos.rs`, `src/x86/dos/*`)
- **EFI Display**: Defaulted EFI builds to verbose display mode with forced text-mode console rendering (`src/bin/efi.rs`, `src/x86/efi/display.rs`)

### Fixed
- **Intel Hybrid E-Core Detection**: Corrected Alder Lake E-core identification from Goldmont to Gracemont (`src/x86/vendor/intel.rs`)
- **Socket and Topology Calculation**: Fixed multi-socket counting bugs on Apple EFI hardware, Haiku OS, and DOS MP Table systems (`src/x86/count.rs`, `src/common/os/haiku.rs`, `src/x86/dos/mp.rs`)
- **Centaur / VIA Feature Detection**: Fixed false-positive PadLock security feature detection on IDT WinChip and corrected VIA Eden/Nano cache detection (`src/x86/vendor/centaur.rs`, `src/x86/cache.rs`)
- **Cross-Platform Cache Formatting & Gating**: Fixed unified L1 cache size formatting and resolved compilation warnings on macOS ARM, Android, DOS, and PowerPC (`src/common/display.rs`, `src/arm/*`, `src/ppc/*`)
