# Changelog

For changes prior to version 2.0.0, see [CHANGELOG-1.0.md](./CHANGELOG-1.0.md).

## [2.3.1] — SteamOS / Linux GTK GUI fixes, CPUID brand string NUL-termination, and Linux system name improvements

### Added
- **Steam Deck Test Fixture**: Added AMD Custom APU 0405 (Van Gogh) CPUID dump test fixture and assertions (`tests/cpuid/dump/Amd_Custom_APU_0405.txt`, `tests/cpuid_dump_test.rs`)
- **Linux GTK Dark Theme Detection**: Added automatic system dark theme detection on startup and explicit monospace font fallbacks for the Linux GTK GUI (`src/gui/linux/window.rs`)

### Changed
- **Common Core & OS Deduplication**:
  - Unified cache level hierarchy insertion via `Cache::insert_level`, removing duplicated multi-level cache match blocks across Linux sysfs and Windows detection (`src/common/cache.rs`, `src/common/os/linux_sysfs.rs`, `src/common/os/windows.rs`)
  - Consolidated system and SoC detail line formatting in `CpuDisplay` via shared `display_system_info`, streamlined cache entry string construction, and made `is_duplicate` string comparison unconditionally available across all architectures (`src/common/display.rs`, `src/arm/display.rs`)
  - Replaced multiple manual whitespace-collapsing loops with a shared `collapse_whitespace` utility (`src/common/util.rs`, `src/common/os/common.rs`, `src/common/os/windows.rs`, `src/x86/cpu.rs`)
  - Streamlined CPU list count and expansion logic using a shared `parse_cpu_ranges` iterator and unified trailing `/proc/cpuinfo` hardware/model lookups (`src/common/os/common.rs`, `src/common/os/android.rs`)
- **Shared SMBIOS Parser & Windows/UEFI OS Abstractions**:
  - Extracted cross-platform SMBIOS 2.x and 3.x table parsing and system information resolution into a shared `smbios` module (`src/common/os/smbios.rs`), deduplicating >750 lines across UEFI (`src/x86/efi/smbios.rs`) and Windows (`src/common/os/windows.rs`)
  - Added Windows registry helper functions `read_reg_dword` and `read_reg_qword` in `src/common/os/windows.rs`, eliminating repetitive registry querying boilerplate across ARM CPU feature detection (`src/arm/os/windows.rs`) and GUI dark mode detection (`src/gui/windows/theme.rs`)
  - Consolidated repetitive buffer allocation and pointer traversal loops for `GetLogicalProcessorInformationEx` via `for_each_processor_info_ex` across socket counting, topology detection, and cache enumeration (`src/common/os/windows.rs`)
  - Removed duplicate SMBIOS and Apple hardware model unit tests in Windows and UEFI modules, consolidating test coverage into shared common modules tested on all targets (`src/common/os/smbios.rs`, `src/common/os/common.rs`, `src/common/os/windows.rs`, `src/x86/efi/smbios.rs`)
- **x86 Core, Topology & CPUID Dump Deduplication**:
  - Unified hybrid core probing, clustering (`push_or_merge_core`), and thread/cache finalization (`finalize_hybrid_cores`) across CPUID dump detection, live OS thread affinity, and UEFI MP services (`src/x86/cpu.rs`, `src/x86/os.rs`, `src/x86/efi/mod.rs`)
  - Consolidated multi-socket core, thread, and cache scaling into `Topology::scale_sockets`, replacing duplicate scaling blocks across topology detection, OS enrichment, UEFI MP services, and DOS MP Table (`src/x86/topology.rs`, `src/x86/os.rs`, `src/x86/efi/mod.rs`, `src/x86/dos/mod.rs`)
  - Extracted `CpuDump::parse_line` and streamlined multi-CPU context detection in `CpuDump::parse_str`, removing duplicated file-wide line parsing loops (`src/x86/provider.rs`)
- **DOS & UEFI Target Deduplication**:
  - Extracted shared ANSI CSI SGR parser (`parse_ansi_csi_sgr`) in `src/common/display.rs` with unit tests, replacing duplicate escape sequence parsers across DOS direct VRAM output (`src/x86/dos/mod.rs`) and UEFI screen console output (`src/x86/efi/display.rs`)
  - Unified conventional memory segment:offset peeking (`peek_u8_so`, `peek_u16_so`) in `src/x86/dos/mod.rs` and removed duplicate memory pointer implementations from MP Table parsing (`src/x86/dos/mp.rs`)
  - Consolidated real-mode and protected-mode CPU speed measurement and calibration in `src/x86/dos/speed.rs` into unified helpers (`calculate_freq_mhz`, `wait_for_ticks_tsc`, `run_calibrated_loop`)
  - Compile-gated post-486 microarchitecture lookup tables, vendor modules (`centaur`, `transmeta`), CPUID dump generation (`dump`), hypervisor detection, 128-bit easter egg validation, cache share-count resolution, ANSI color formatting, and Apple Mac model tables from the 16-bit real-mode DOS binary (`rust86.exe`), reducing its size by ~23% (from 43,036 bytes to 33,128 bytes) (`src/x86/*`, `src/common/cache.rs`, `src/common/display.rs`)
- **ARM & RISC-V Architecture Deduplication**:
  - Extracted shared `display_arm_core_details` helper to unify hybrid and homogeneous cluster formatting across implementer, microarchitecture, codename, frequency, and core cache lines (`src/arm/display.rs`)
  - Added `MicroArch::as_str` and `From<MicroArch> for &'static str` in `src/arm/micro_arch.rs`, deriving `String` conversion directly from static strings
  - Used named `IMPL_*` vendor constants in `Vendor::from(usize)` instead of raw hex values (`src/arm/brand.rs`)
  - Unified RISC-V feature category mapping via `FEATURE_CATEGORIES`, eliminating the duplicate 50-line raw extension name array in `populate_detected_features` (`src/riscv/features.rs`)
  - Replaced repetitive `CpuArch` struct initialization in RISC-V device tree matching with a declarative `COMPAT_TABLE` lookup (`src/riscv/micro_arch.rs`)
- **Cross-Platform GUI Deduplication**:
  - Extracted shared semantic report line tokenizer (`ReportLineToken` and `tokenize_report_line`) in `src/gui/styled_text.rs`, deduplicating report styling logic across Linux/Haiku GTK styled runs and Windows RichEdit RTF generation (`src/gui/styled_text.rs`, `src/gui/windows/rtf.rs`)
  - Consolidated Windows menu item creation across Unicode (`AppendMenuW`) and ANSI (`AppendMenuA`) modes via a unified `append_menu_item` helper, removing ~200 lines of duplicated menu definitions (`src/gui/windows/menu.rs`)
  - Unified Win32 message box display and default window procedure delegation via `show_message_box` and `def_window_proc` (`src/gui/windows/window.rs`)
  - Routed keyboard shortcuts and menu clicks through a single `execute_command` handler (`src/gui/windows/window.rs`)
- **Linux DMI System & SoC Name Filtering**: Prioritized `product_name` over `product_family` when resolving Linux DMI system names and filtered generic placeholder values (including `"arm-platform"`) from system and SoC names (`src/common/os/linux.rs`, `src/common/os/common.rs`, `src/arm/display.rs`)
- **Build Artifact Layout & DOS/32A Binding**: Standardized build output paths under `target/dist/<platform>/`, added direct DOS/32A stub binding to `elf2le`, and removed redundant build configuration files (`Makefile`, `justfile`, `build-config/*`, `tools/elf2le/src/main.rs`)

### Fixed
- **CPUID Brand String NUL-Termination & SteamOS GUI Text Rendering**: Fixed `read_multi_leaf_str` reading past the NUL terminator into trailing firmware buffer memory (such as on the Steam Deck's `AMD Custom APU 0405`), which embedded a `\0` byte in the model string and caused `GtkTextBuffer` UTF-8 validation to reject the report text (`src/x86/fns.rs`, `src/gui/common.rs`, `src/gui/linux/window.rs`)
- **Centaur/Zhaoxin SM2 Enabled Check**: Fixed `sm2_enabled` checking bit 0 (support) instead of bit 1 (enabled status) in `src/x86/vendor/centaur.rs`
- **DOS CLI Compact Flag**: Fixed inverted `"COMPACT"` CLI argument handling in `src/bin/dos.rs`
- **DOS Real-Mode & UEFI Build Fixes**: CFG-gated hybrid topology helpers to non-DOS platforms in `src/x86/cpu.rs` to fix `just build-dos`, and resolved missing `is_intel` symbol import in `src/x86/efi/mod.rs`
- **RISC-V Crypto & Extended Feature Display**: Fixed category mismatch between `"ScalarCrypto"` in `build_feature_map` and `"Crypto"` in `display_features`, and added display keys for `"Zc"`, `"AmoMaxMin"`, and `"MemModel"` extensions (`src/riscv/features.rs`, `src/riscv/display.rs`)
- **Windows GUI Export Filename Sanitization**: Fixed `Ctrl+S` export shortcut in Windows GUI missing filename sanitization for `/` and `\` characters and missing completion alert dialog (`src/gui/windows/window.rs`)
- **Hybrid Core Affinity Persistence Across View Mode Switches & CPUID Dumps**: Cached initial logical core IDs in `get_logical_core_ids` and restored primary core affinity in `for_each_logical_core` and `dump_all_cpus`, fixing hybrid core output disappearing on subsequent GUI view renders (such as switching to "Everything" mode) and fixing multi-core CPUID dumps repeating CPU 0 (`src/common/os/common.rs`, `src/x86/os.rs`, `src/x86/dump.rs`)

## [2.3.0] — Classic Macintosh GUI port (68k & PowerPC), PowerPC detection, and AMD Athlon improvements

### Added
- **Classic Macintosh Port (68k & PowerPC)**: Added standalone Classic Macintosh GUI application supporting System 6.0.8 through Mac OS 9.2.2 as well as Mac OS X (Aqua/Carbon) across Motorola 68000–68060 and PowerPC (601–G5), featuring hardware probing (CPU, FPU, MMU, bus/clock speeds, Mac model), resizable and scrollable windows, styled text rendering, clipboard copy, and universal fat binary packaging (`ports/classic_mac/*`)
- **Mac OS X Application Bundle**: Added native PowerPC Mac OS X application bundle (`Rustid.app`) featuring native application icons (`.icns`), Aqua menu integration, Command-Q shortcut, and Dock quit handling (`build-config/classic_mac/*`)
- **PowerPC Model & Microarchitecture Detection**: Added PowerPC 7410 (Nitro) PVR `0x800C` recognition and "Vger" codename designation for PowerPC 7450, alongside comprehensive vintage Mac machine model identification (`src/ppc/micro_arch.rs`, `ports/classic_mac/models/models.c`)

### Changed
- **AMD L2 Cache Share-Count Fallback**: Adjusted default L2 cache share count for AMD processors omitting explicit CPUID cache sharing data, ensuring dual-core K8 processors report dedicated per-core L2 caches (`src/common/cache.rs`)
- **Feature Flag HTT Display**: Standardized Hyper-Threading / multi-threading feature flag label to `"HTT"` in feature listings (`src/x86/features.rs`)

### Fixed
- **AMD Mobile Athlon vs. Geode NX Disambiguation**: Fixed false-positive detection where mobile Athlon XP-M processors sharing signature Family 6, Model 8, Stepping 1 with the AMD Geode NX were misidentified as Geode NX (`src/x86/cpu.rs`, `src/x86/vendor/amd.rs`)
- **Windows 32 bit build**: Fixed errors with CI and local build caused by mismatch of SSE2 between the prebuilt standard library and the binary build config

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
