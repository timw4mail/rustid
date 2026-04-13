# Changelog

## [0.11.3]

### Added
- AMD Elan processor mapping
- Intel brand ID lookup table (DOS build excluded due to space constraints)
- License file
- Multiple core type support for ARM processors
- DOS binary size test to verify 64K limit

### Changed
- Use key names instead of blind indexes for lscpu cache information
- Move Speed struct to common module
- Use fewer String objects in PPC module
- Relabel "Cores" display to "Topology"
- Streamline MP table lookup for DOS
- Improved information display for PowerPC

### Fixed
- Fix cache detection for PPC
- Fix detection of cache share-count using correct bit mask
- Fix entry for Geode LX
- Fix PPC display code
- Show L3 cache count for multiple sockets

## [0.10.1]

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

## [0.9.5]

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

## [0.8.6]

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

## [0.7.6]

### Added
- Examples folder with output from real systems
- Another CPU example
- AMD cache display fix (K5/K6)
- Socket count detection for Linux
- Socket count display in DOS (when > 1)

### Changed
- Refactored mp module to split implementations by OS
- Re-wrapped __cpuid function in unsafe block for compatibility with older Rust versions

## [0.7.0]

### Added
- Core/thread count display for DOS
- Extended topology iteration code
- Cache multiplier display based on CPID cache share count

### Changed
- Renamed AMD64 to EM64T for Intel CPUs

### Fixed
- Intel core/thread count detection

## [0.6.2]

### Added
- Cyrix-specific matching for fallback cache lookup
- Associativity to cache output
- Cores/threads for AMD CPUs
- Old-style cache lookup for Intel CPUs

### Fixed
- Logic for determining if Intel cache fallback works

## [0.5.1]

### Added
- Architecture line to output (i386/i686/x86_64_v1/etc)
- Cache information display
- More CPU models

### Changed
- Reformatted Cyrix-specific block


## [0.4.0]

### Added
- Experimental ARM CPU support
- Experimental PowerPC (PPC) functionality
- CPU clock speed display
- Core 2 Quad detection
- Topology/cache/speed information lookup
- AMD extended CPU signature detection (brand_id, pkg_type)
- Intel overdrive processor detection
- UMC 486 mappings
- More CPU mappings and easter eggs

### Changed
- Removed ufmt dependency
- Improved formatting of output

## [0.3.9]

### Added
- Initial release
- x86/x64 CPU detection
- Brand and microarchitecture mapping
- DOS support
