# Changelog

## [2.0.0] — Microarchitecture expansion (Intel & AMD), collision disambiguation, power-gated ARM core detection, multi-socket EFI fixes, Android & Windows support, and ARM overhaul

### Added
- Comprehensive Intel microarchitecture expansion covering Intel Family 6, Family 18 (Nova Lake), and Family 19 (Diamond Rapids) CPUID signatures across desktop, mobile, server, and embedded lineups: Meteor Lake, Arrow Lake, Lunar Lake, Panther Lake, Bartlett Lake, Twin Lake, Granite Rapids, Sierra Forest, Grand Ridge, Clearwater Forest, Sapphire Rapids, Emerald Rapids, Cooper Lake, Rocket Lake, Cannon Lake, Amber Lake, Whiskey Lake, Comet Lake, Knights Mill, Wildcat Lake, Nova Lake, and Diamond Rapids (`src/x86/vendor/intel.rs`, `src/x86/micro_arch.rs`)
- Stepping- and brand-string-aware CPUID signature collision disambiguation for overlapping Intel CPU models:
  - `06_55H`: Skylake-SP/X vs. Cascade Lake-SP/X vs. Cooper Lake
  - `06_8EH`: Amber Lake-Y vs. Kaby Lake-U/R vs. Coffee Lake-U vs. Whiskey Lake-U vs. Comet Lake-U
  - `06_9EH`: Kaby Lake-S/H/X vs. Coffee Lake-S/H vs. Coffee Lake-S/H Refresh
  - `06_0FH` & `06_17H`: Core 2 Duo / Quad / Mobile (Conroe, Kentsfield, Merom, Wolfdale, Yorkfield, Penryn) vs. Enterprise Xeon server equivalents (Woodcrest, Clovertown, Tigerton, Wolfdale-DP, Harpertown)
  - `06_2DH`, `06_3EH`, `06_3FH`, `06_4FH`: HEDT Extreme (`-E`) vs. Multi-Socket Enterprise Xeon (`-EP`/`-EN`/`-EX`) for Sandy Bridge, Ivy Bridge, Haswell, and Broadwell
  - `06_8FH`: Sapphire Rapids-SP vs. Sapphire Rapids-WS vs. Xeon Max (HBM)
  - `06_B7H` / `06_BFH`: Raptor Lake 13th Gen vs. 14th Gen Refresh vs. Core Series 1 / Series 2
  - `06_BEH`: Alder Lake-N vs. Twin Lake-N
- Comprehensive AMD microarchitecture expansion covering AMD Family 15h, 17h, 19h, and 1Ah generations across desktop, mobile, server (EPYC), and workstation (Threadripper) lineups: K5, K6, K7, K8, K10/K10.5, Bobcat, Jaguar, Puma, Bulldozer, Piledriver, Steamroller, Excavator, Zen 1, Zen+, Zen 2, Zen 3, Zen 3+, Zen 4, Zen 4c, Zen 5, and Zen 5c (`src/x86/vendor/amd.rs`, `src/x86/micro_arch.rs`)
- Stepping-, model-number-, and brand-string-aware CPUID signature collision disambiguation for overlapping AMD CPU models (`src/x86/vendor/amd.rs`):
  - Summit Ridge vs. Pinnacle Ridge vs. Whitehaven vs. Colfax vs. Naples vs. Raven Ridge vs. Picasso vs. Banded Kestrel vs. Dali vs. Pollock
  - Matisse vs. Rome vs. Castle Peak vs. Renoir vs. Lucienne vs. Van Gogh vs. Mendocino
  - Vermeer vs. Vermeer-X (3D V-Cache) vs. Milan vs. Milan-X vs. Chagall vs. Cezanne vs. Barcelo / Barcelo-R vs. Rembrandt / Rembrandt-R
  - Raphael vs. Raphael-X (3D V-Cache) vs. Dragon Range vs. Genoa vs. Genoa-X vs. Bergamo vs. Siena vs. Storm Peak vs. Phoenix vs. Phoenix 2 vs. Hawk Point
  - Granite Ridge vs. Turin vs. Turin Dense vs. Strix Point vs. Strix Halo vs. Krackan Point
  - Legacy AMD CPU model disambiguation across K5, K6, K7 Athlon/Duron, K8 Opteron/Athlon 64/X2, K10/K10.5 Phenom/Athlon II/Opteron, and Bulldozer/Piledriver/Steamroller/Excavator families
- New `MicroArch` enum variants: `MicroArch::Zen3Plus`, `MicroArch::Zen4C`, and `MicroArch::Zen5C` (`src/x86/micro_arch.rs`)
- Hybrid core type and server core resolution for Raptor Lake, Meteor Lake, Arrow Lake, Lunar Lake, Panther Lake, Sapphire Rapids, Emerald Rapids, Granite Rapids, Sierra Forest, Grand Ridge, and Clearwater Forest (`src/x86/vendor/intel.rs`)
- Manufacturing process node constants for Intel and AMD fabrication nodes (`INTEL_7`, `INTEL_4`, `INTEL_3`, `INTEL_20A`, `INTEL_18A`, `N10SF`, `TSMC_3`, `TSMC_4`, `TSMC_5`, `TSMC_6`, `TSMC_7`, `GF_12`, `GF_14`, `GF_28_SHP`, `TSMC_28_SHP`, `GF_32_SOI`, `GF_45_SOI`, `TSMC_40`, `TSMC_65`, `IBM_65_SOI`, `IBM_90_SOI`, `IBM_130_SOI`) (`src/common/constants.rs`, `src/x86/vendor/amd.rs`, `src/x86/vendor/intel.rs`)
- Technical citations and source documentation referencing Intel SDM Vol 4 Table 2-1, Intel specification updates, Linux `intel-family.h`, `libcpuid`, and instlatx64 dumps (`src/x86/vendor/intel.rs`)
- Power-gated ARM core discovery via sysfs topology (`/sys/devices/system/cpu/possible`, `/sys/devices/system/cpu/present`, `/sys/devices/system/cpu/cpu*/topology`) on Linux and Android, allowing accurate detection of offline/gated big and little cores (`src/arm/os/mod.rs`, `src/arm/os/linux.rs`, `src/arm/os/android.rs`)
- Android system information and ARM core detection via Android system properties (`__system_property_get` / `getprop`) and sysfs (`src/common/os/android.rs`, `src/arm/os/android.rs`)
- Windows topology (sockets, cores, threads) and multi-level cache detection (L1, L2, L3) via `GetLogicalProcessorInformationEx` (`src/common/os/windows.rs`)
- Windows ARM SoC and model detection for Qualcomm Snapdragon processors (Snapdragon X Elite, Snapdragon 8cx Gen 3) (`src/arm/os/windows.rs`)
- Expanded ARM implementer and microarchitecture database covering Apple Silicon, Qualcomm Oryon/Kryo, Samsung Exynos, Fujitsu A64FX, ARM Neoverse/Cortex, Phytium, Ampere, Nvidia, and SiPearl (`src/arm/brand.rs`, `src/arm/micro_arch.rs`)
- SMBIOS 2.x/3.x table parser for EFI, enabling system name and CPU speed detection on UEFI firmware (`src/x86/efi/smbios.rs`)
- OS-level cache detection with share-count fallback merging for ARM, PPC, RISC-V, and x86 (`src/common/cache.rs`)
- Asymmetric 3D V-Cache display support for dual-CCD AMD Ryzen processors with single-CCD 3D V-Cache (e.g. Ryzen 7950X3D) (`src/x86/cache.rs`, `src/x86/display.rs`)
- Additional Mac model mappings across ARM, x86_64, and PowerPC (`src/common/display.rs`)
- Example output fixture for Apple MacBook Neo (`examples/macbook-neo.txt`)
- Automated changelog extraction script and crates.io publishing support in release workflow (`.github/scripts/extract_changelog.py`, `.github/workflows/release.yml`)
- Additional x86 integration tests and CPUID dump test fixtures (Intel Core i7-12700H, Intel Celeron Eee PC, VIA EdenX2) (`tests/cpuid/dump/edenx2.txt`, `tests/cpuid_dump_test.rs`)
- `check-win-arm` target check command in `justfile`

### Changed
- Refined EFI socket count and thread count calculation in `src/x86/count.rs` to compute physical package count accurately against logical processors from MP Services without overcounting individual cores in legacy SMBIOS tables or undercounting multi-socket servers
- Overhauled ARM output formatting and structure to mirror x86 output styling, eliminate redundant printing of shared vendor/SoC metadata across core types, group clusters cleanly, and label sub-cores with "Name" (`src/arm/display.rs`)
- Hoisted shared ARM CPU codenames to the top-level CPU header section when all core clusters belong to the same SoC/chip, avoiding repetitive per-core-type codename lines (`src/arm/display.rs`)
- Moved raw Mac model identifier display (e.g. `[MacBookAir10,1]`) to verbose mode (`-v`/`--verbose`), keeping default output clean with friendly marketing names (`src/common/display.rs`)
- Enabled verbose mode by default for EFI binaries (`src/efi_rustid.rs`)
- Real-mode DOS binary (`rust86.exe`) refactored and merged with real-mode debug binary, eliminating the separate debug executable while significantly reducing binary footprint and adding simple CLI argument handling (`src/rust86.rs`, `src/x86/dos/args.rs`)
- Overhauled integration test suite with `make_tests!` macro, eliminating repetitive test boilerplate across CPUID fixtures (`tests/cpuid_dump_test.rs`)
- Topology display now consistently shows sockets, cores, and threads across EFI, DOS, and standard OS builds, and restored core/thread count formatting for homogeneous x86 CPUs (`src/x86/display.rs`)
- EFI QEMU runner forces text mode so output is visible in `run-efi-32` and `run-efi-64` (`src/x86/efi/display.rs`)
- Improved x86 cache count detection with additional fallback paths (`src/common/cache.rs`, `src/x86/cache.rs`)
- Improved robustness of CPU topology counts on EFI via MP Services and SMBIOS data (`src/x86/efi/mp.rs`, `src/x86/efi/smbios.rs`)
- Simplified Centaur / VIA feature extraction and CPU identification logic (`src/x86/vendor/centaur.rs`)
- Restored fallback CPU speed measurement for DOS environments lacking TSC support (`src/x86/dos/speed.rs`, `src/x86/topology.rs`)

### Fixed
- Fixed Alder Lake E-core identification in `Intel::core_micro_arch` (previously reported as `Goldmont`, now correctly identified as `Gracemont`) (`src/x86/vendor/intel.rs`)
- Fixed socket count detection on multi-socket Apple EFI / UEFI hardware (such as Xserve1,1 and MacPro1,1) where socket count was erroneously clamped to 1 or inflated by per-core SMBIOS entries (`src/x86/count.rs`, `src/x86/efi/mp.rs`)
- Fixed legacy SMBIOS 2.4 processor table parsing where CPU status `0x01` (Enabled without bit 6 set) was treated as unpopulated (`src/x86/efi/smbios.rs`)
- Corrected L2 cache count detection for VIA Eden / Nano X2 dual-core processors (`src/x86/cache.rs`)
- Improved behavior and error handling for Cyrix CPUs running in DOS (`src/x86/vendor/cyrix.rs`)
- Fixed compile gating breaking real-mode DOS builds and resolved DOS compilation warnings (`src/common/cache.rs`, `src/x86/display.rs`)
- Fixed compiler warnings on macOS ARM target builds (`src/arm/os/macos.rs`, `src/arm/os/mod.rs`)
- Fixed socket count calculation on Haiku OS where total logical CPU count from `sysinfo` was erroneously treated as physical sockets, causing inflated core/thread counts on multi-core processors like VIA Nano X2 (`src/common/os/haiku.rs`, `src/x86/display.rs`)
- Fixed unified L1 cache size formatting in `src/common/display.rs` where raw byte sizes (e.g. 16384 bytes) were displayed as KB without unit conversion
- Fixed missing compile guards for Android target compilation (`src/arm/features.rs`, `src/arm/mod.rs`, `src/arm/os/mod.rs`)
- Fixed PowerPC build compile error (`src/common/cache.rs`)

## [1.9.0] — EFI/UEFI support, compact display, and VIA features

### Added
- EFI/UEFI application support for 32-bit and 64-bit x86 firmware (`just build-efi`, `just run-efi-64`, `just run-efi-32`)
- EFI MP Services for core/thread/socket topology detection on UEFI systems
- Graphical display mode with custom font rendering for EFI
- Colored EFI output via ANSI escape sequences
- Compact display mode that removes extra newlines between sections (`c` / `compact` flag)
- Centaur/VIA feature list now shown in DOS32A extended builds
- i486 Linux musl build target (`just build-486-musl`)

### Changed
- Separated DOS argument parsing and speed detection into dedicated submodules (`src/x86/dos/args.rs`, `src/x86/dos/speed.rs`)
- Moved MP Table detection under DOS sub-module (`src/x86/dos/mp.rs`)
- EFI binaries placed in their own target directory (`target/efi-disk/`)
- Cleaned up legacy CPU socket count detection code
- Updated README platform support tables with EFI column

## [1.8.0] — DOS32A protected-mode support and cross-architecture cleanup

### Added
- DOS32A DOS Extender support for 32-bit protected-mode DOS binaries (`rustid.exe` built with DOS32A) - this allows showing more information, adding program arguments, and removes the 64K size limitation of the original real-mode dos binaries
- Custom `elf2le` tool to convert ELF binaries to Linear Executable (LE) format for DOS32A
- Automatic fallback to real-mode DOS binary for pre-CPUID CPUs requiring CPU reset identification
- Platform support details and tables added to README

### Changed
- Renamed internal `cpuid` module to `x86` for cross-architecture consistency
- Extracted display/formatting logic from CPU detection in ARM and PowerPC modules
- Reorganized binary targets, link scripts, and target specs into `build-config/` directory
- Aligned DOS extender formatting, panic handling, and trace display with real-mode DOS version

### Fixed
- Patched DOS32A binary to suppress startup banner and warnings
- An issue where Intel/Zhaoxin CPUs with multiple cores may show the core count as socket count, and multiply the core count by the spurious socket count

## [1.7.0] — RISC-V support and improved system name detection

### Added
 - Initial Risc V (64bit) support
 - IvyBridge-EN (EP) CPU mapping
 - Xeon E5-2407 CPUID dump for testing

### Changed
 - Improved Linux system name detection (more sources, whitespace cleanup, empty string handling)
 - Expanded filtering of generic firmware strings (e.g. "System Product Name", "Default string") when detecting the Linux system name
 - Linux system name folds in the hypervisor vendor for VMs (e.g. "QEMU Standard PC ...") and falls back to board identity when the product string is a placeholder

 ### Fixed
 - System name no longer returns empty strings
 - System name lookup skips placeholder strings instead of misreporting them

## [1.6.0] — ARM Mac model detection and cross-platform Mac model lookup

### Added
- System/device model detection from Linux devicetree `compatible` string
- Mac model lookup table and mappings (ARM and x86_64)
- PowerPC Mac model mappings
- Expanded PowerPC model coverage and improved mappings
- PVR value display in hex for PowerPC debug output
- More unit tests
- Android and OpenBSD build support

### Changed
- Centralized OS-specific information gathering for ARM into shared module
- Refactored System/SoC properties onto the main CPU object
- Moved raw MIDR values into `Midr` sub-struct with updated debug display
- Extracted Mac model table for cross-architecture reuse
- Improved formatting of ARM SoC data

### Fixed
- Crash from missing values in `lscpu` output
- System formatting and string filtering for Mac model detection
- PowerPC system display (correct property reference, code errors, missing borrow)

## [1.5.0] — ARM core types, BSD support, and feature trait unification

### Added
- SoC/device model name shown in ARM output when available from `/proc/cpuinfo`
- `CpuArch::brand_arch()` factory method to deduplicate x86 vendor micro-arch lookup closures
- Shared `get_proc_cpuinfo_data()` helper that parses `/proc/cpuinfo` into structured key-value maps
- ARM core type groups separated by a blank line in output for readability
- BSD support for ARM (NetBSD, FreeBSD, OpenBSD) via new `src/arm/os/bsd.rs` module with MIDR detection through sysctl and inline asm fallback
- System/device model field (`CpuArch::system`) displayed as "System" line, sourced from `hw.model` (NetBSD), `hw.fdt.model` (FreeBSD), or `/proc/cpuinfo Model` (Linux), separate from the SoC model line
- `TArmFeatures` trait providing a uniform interface for OS-specific ARM feature detection, implemented across Linux, macOS, Windows, and BSD modules
- Shared `populate_detected_features()` helper eliminating duplicated feature-map construction in each OS module
- ARM1176JZF-S (Raspberry Pi 1) microarchitecture variant and `MicroArch::Arm1176` variant
- Raspberry Pi codename annotations in ARM core entries (Pi 2/3/4/5)
- FreeBSD SoC detection via `hw.fdt.compatible` sysctl
- `specres` feature flag to the ARM miscellaneous feature list
- Named MIDR bit-field offset constants (`IMPLEMENTER_OFFSET`, `PART_OFFSET`, `VARIANT_OFFSET`, `ARCHITECTURE_OFFSET`, `REVISION_MASK`)

### Changed
- Restructured ARM module into OS-specific submodules (`src/arm/os/{apple,linux,windows}.rs`) with shared core detection in `os/mod.rs`
- Renamed `src/arm/os/apple.rs` → `macos.rs` for consistency
- Rewrote ARM Linux feature detection to parse `/proc/cpuinfo` instead of `libc` system calls; removed `libc` dependency
- Simplified ARM CPU model/part lookup tables from verbose `match` arms to concise tuple arrays
- Replaced raw line-by-line `/proc/cpuinfo` parsing with shared structured parser across ARM, PPC, and x86 topology detection
- Replaced hardcoded cache lookup tables for ARM and PPC micro-architectures with runtime OS cache detection
- Simplified x86 vendor micro-arch closures (AMD, Intel, Centaur, Cyrix) via shared `CpuArch::brand_arch()`
- Intel CPU vendor module now implements the `TMicroArch` trait, matching other vendors
- PPC clock speed parsing uses shared `get_proc_cpuinfo_data()` instead of raw string parsing
- ARM 32-bit Linux MIDR detection reads from sysfs instead of inline `mrc p15` assembly to avoid SIGILL on older CPUs
- Migrated all OS-specific ARM feature functions from standalone `has_*()` to `TArmFeatures` trait implementations
- Sysctl parser accepts `=` as a delimiter alongside `:` for NetBSD compatibility
- Output labels refined: "Brand" → "Implementer"; split "SoC/System" into distinct "System" and "SoC" lines
- Updated G4 PowerPC codenames (Apollo 6/7, Max, V'ger) for accuracy

### Fixed
- Corrected hex literal formatting in Apple CPU part matching (e.g., `0x32` → `0x032`) to ensure correct M3/M4 detection
- Fixed PPC clock speed parsing from `cpu MHz` lines
- Corrected ARM Cortex-A72 mapping (previously misidentified as Cortex-A65)
- Corrected PPC G4 codename labels for 7447, 7455, 7457 variants
- Eliminated double blank line in ARM output when a core type has no cache information

## [1.4.0] — Hybrid x86 core type details

### Added
- Support for showing core type details for hybrid x86 cpus
- More detailed documentation for DOS version (DOS.md)
- Data source properties to debug output

### Changed
- Updated cpuid dump feature to dump information from each thread, rather than the same output for each thread
- Optimized allocations and memory usage for DOS version

### Fixed
- Corrected ARM cpu mapping for Cortex-A76 (found in Raspberry Pi 5)

## [1.3.0] — Hypervisor detection and cache improvements

### Added
- Hypervisor vendor string in debug output

### Changed
- Added ability to get cache types for different core types on linux arm

### Fixed
- Fixed detection of KVM hypervisor
- Fixed crash when `lscpu -C` produces no output
- Fix crash for Cyrix cpus in dos due to excessive memory allocations
- Fix other memory allocation crash for dos

## [1.2.0] — Verbose mode and x86 display refactoring

### Added
- Verbose flag added to cli options
- Extended signature line to verbose mode
- Re-added "Features" label to x86 CPU output
- Show APIC and MMX+ extensions in x86 feature list
- Show 3dnow prefetch in x86 feature list
- test-dos command to Makefile and Justfile

### Changed
- Separated dos binaries from non-dos binary
- Refactored x86 display table to use common display module, reducing code duplication
- Further deduplicated output formatting code across architectures
- Refactored ARM display of different core types
- Removed unused DOS module

### Fixed
- Updated Centaur feature flag detection (IDT, Via, Zhaoxin) based on CPU datasheets
- Read hypervisor vendor string in the correct byte order
- Improved core and thread count detection for AMD cpus

## [1.1.0] — Hypervisor info, categorized features, and Makefile

### Added
- Makefile for users who prefer it over Just or in environments that don't support just
- Detection of hypervisor information (when current OS is virtualized)
- Checks for NX-bit and Virtualization features
- Expanded AX512 feature list
- CPU feature list categorized by type
- Direct conversion of raw ELF binary to DOS MZ EXE binary (instead of using rust-objcopy)
- Feature section for Centaur cpu instructions

### Changed
- Display of CPU feature list for ARM
- Updated non-x86 formatting to match x86 style
- Restored functionality of CPU reset signature detection (for dos)
- Updated README to focus on binary usage
- Updated README to reflect binaries and cargo install
- Improved output for cpu dumps when displaying dumps on x86_64 for x86 cpus
- Allow color output on Windows

## [1.0.0] — Zen5 support, color output, and reorganized test data

### Added
- Zen5 CPU support
- SiS model string support
- Mac arm64 example
- Process node values for Vortex86 mappings
- Color output for PowerPC and ARM
- Helper to identify source CPU ID data
- Ability to combine CLI flags (debug command can get info from CPUID dump)
- Haiku socket detection and MpTable implementation
- Reorganized test data files

### Changed
- Removed custom string format macro
- Removed type wrappers, using native Rust types with DOS allocator
- Enabled alloc types for DOS (String, Vec)
- Updated release build config to reference .exe files
- Restored Intel Brand Table lookup for DOS (replaced Unicode registered trademark with (R))
- Refactored ARM and PPC formatting into common module
- Adjusted CLI parsing for all architectures
- Improved speed measurements for CPUs without TSC instruction
- Updated most of the examples
- Updated M1 Apple chip cache mapping
- Minor code cleanups

### Fixed
- 386 compatibility for DOS build
- Display of SiS CPU easter egg
- Fixed Haiku socket detection
- Removed wildly inaccurate speed measurement for some Cyrix CPUs
- Tweaked display of cleaned-up model strings

## [0.11.4] — Brand ID lookup, multi-core ARM, and CPUID dump rendering

### Added
- OS and CPU Architecture in version string
- AMD Elan processor mapping
- Intel brand ID lookup table (DOS build excluded due to space constraints)
- License file
- Multiple core type support for ARM processors
- DOS binary size test to verify 64K limit
- Option to render output from raw CPUID dump files

### Changed
- Use key names instead of blind indexes for lscpu cache information
- Move Speed struct to common module
- Use fewer String objects in PPC module
- Relabel "Cores" display to "Topology"
- Streamline MP table lookup for DOS
- Improved information display for PowerPC
- Optimize DOS binary size further
- Favor AMD-style cache lookup for Centaur CPUs with fallback
- Improve accuracy of CPUID dump display

### Fixed
- Fix cache detection for PPC
- Fix detection of cache share-count using correct bit mask
- Fix entry for Geode LX
- Fix PPC display code
- Show L3 cache count for multiple sockets
- Fix string truncation bug in DOS, increase fixed string sizes for multi-byte characters

## [0.10.1] — AMD 5x86 synthetic model and improved Cyrix detection

### Added
- Synthetic model name for AMD 5x86

### Changed
- Made DOS speed measurements more accurate
- Made Cyrix brand list more specific depending on cpu model
- Improved Cyrix and K6 detection
- Improved Cyrix detection without CPUID
- Updated 486 Linux build configuration
- Excluded core_affinity crate from x86 targets
- Improved robustness of cache info detection from extended leaves 5 and 6
- De-duplicated ARM formatting logic

### Fixed
- Don't show enable cpuid message for 5x86 chips that don't support it

## [0.9.5] — Apple Silicon codenames, AES/SHA flags, and cache associativity

### Added
- Intel N100 CPU mapping
- Intel Haswell-EP CPU mapping
- AMD K10 Dual-Core Athlon mapping
- AMD FX-9590 example
- RapidCAD example
- AES, VAES, and SHA flag checks
- Additional feature classes for 686 class processors with SSE and SSE2
- Qualcomm CPU mappings
- Codenames for more Apple Silicon CPUs
- More integration tests with cache, core, and thread count validation
- Additional output examples (2PPRO, Crusoe, U5S)

### Changed
- Improved AMD cache associativity detection
- Improved detection of 386 and 486 socket Cyrix CPUs
- Improved detection of 486 CPUs
- Improved formatting of Apple Silicon CPUs on macOS
- Various output formatting tweaks
- Added rough speed detection for DOS
- Refactored string handling with new String wrapper type
- Reduced code duplication in topology detection
- Updated Cyrix MII example

### Fixed
- Fix Windsor CPU mapping
- Fix core count for AMD CPUs before Bulldozer
- Fix mapping of Brisbane
- Differentiate between 3 and 4 core Phenom 1 chips
- Fix M1 CPU mapping

### Removed
- Removed Windows code for getting MP tables (packages won't run on old CPUs)
- Removed arm-only dependency from x86/x86_64 Windows builds

## [0.8.6] — Apple Silicon detection and Transmeta support

### Added
- Apple Silicon detection with core codenames and cache info
- Qualcomm CPU mappings
- PowerPC speed/cache information
- More ARM core mappings
- Transmeta CPU support
- Integration tests using raw CPUID dumps
- More Zhaoxin CPU support
- Raw CPUID dump folder for testing/debugging

### Changed
- Refactored ARM detection to prepare for multiple core types
- Refactored vendor-specific micro-arch mapping into vendor sub-modules
- Simplified Linux multi-socket detection via /proc/cpuinfo

## [0.7.6] — Socket count detection and example output

### Added
- Examples folder with output from real systems
- Another CPU example
- AMD cache display fix (K5/K6)
- Socket count detection for Linux
- Socket count display in DOS (when > 1)

### Changed
- Refactored mp module to split implementations by OS
- Re-wrapped __cpuid function in unsafe block for compatibility with older Rust versions

## [0.7.0] — Core/thread count display and cache multiplier

### Added
- Core/thread count display for DOS
- Extended topology iteration code
- Cache multiplier display based on CPID cache share count

### Fixed
- Intel core/thread count detection

## [0.6.2] — Cache associativity and AMD core detection

### Added
- Cyrix-specific matching for fallback cache lookup
- Associativity to cache output
- Cores/threads for AMD CPUs
- Old-style cache lookup for Intel CPUs

### Fixed
- Logic for determining if Intel cache fallback works

## [0.5.1] — Architecture line and cache information display

### Added
- Architecture line to output (i386/i686/x86_64_v1/etc)
- Cache information display
- More CPU models

### Changed
- Reformatted Cyrix-specific block


## [0.4.0] — ARM, PowerPC, and clock speed support

### Added
- Experimental ARM CPU support
- Experimental PowerPC (PPC) functionality
- CPU clock speed display
- Core 2 Quad detection
- Topology/cache/speed information lookup
- Intel overdrive processor detection
- UMC 486 mappings
- More CPU mappings and easter eggs

### Changed
- Removed ufmt dependency
- Improved formatting of output

## [0.3.9] — Initial release

### Added
- Initial release
- x86/x64 CPU detection
- Brand and microarchitecture mapping
- DOS support
