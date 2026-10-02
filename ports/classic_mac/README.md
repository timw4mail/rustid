# Classic Macintosh Port (`ports/classic_mac`)

A native, standalone Classic Macintosh port of **rustid** targeting Apple Macintosh systems running **System 6.0.8 through Mac OS 9.2.2** on Motorola 680x0 and PowerPC processors.

---

## Overview

Unlike modern platforms where `rustid` is compiled directly from Rust, the Classic Macintosh operating system has no LLVM or native `rustc` backend targeting the classic Macintosh Toolbox, 68k `CODE` resources, or PowerPC Code Fragment Manager (CFM) PEF binaries.

To bring `rustid`'s CPU and hardware identification capabilities to vintage Macintosh hardware, this port is implemented as a clean, standalone C application located in [`ports/classic_mac/`](file:///home/tim/code/rustid/ports/classic_mac/). It provides:

- **100% Native Macintosh Toolbox GUI**: Window Manager, Menu Manager, TextEdit with multi-color runs, Dialog Manager, and Scrap Manager (clipboard) support.
- **Dual-Architecture Support**: Built for both **Motorola 680x0** (68000 through 68060) and **PowerPC** (601 through G4).
- **Universal Fat Binary**: A single executable binary file that runs natively at full speed on both 68k and PowerPC Macs.
- **Safe Hardware Probing**: Completely safe user-space Gestalt queries and hardware tables designed to run without illegal instruction crashes across all operating system versions.

---

## System Compatibility

| Hardware Architecture | Minimum OS | Maximum OS | Notes |
|:----------------------|:-----------|:-----------|:------|
| **Motorola 68000 / 68020 / 68030** | System 6.0.8 | System 7.5.5 | Color QuickDraw supported if available; falls back cleanly to monochrome. |
| **Motorola 68040 / 68LC040** | System 7.0 | Mac OS 8.1 | Full FPU/MMU discrimination. |
| **PowerPC 601 / 603 / 604 / G3 / G4** | System 7.1.2 | Mac OS 9.2.2 | Runs as native CFM PEF code via the `cfrg` resource. |

---

## Directory Structure

```text
ports/classic_mac/
├── classic_mac_engine.c    # Core CPU detection, Gestalt probing, specs table, and main()
├── classic_mac_engine.h    # Engine data structures and function prototypes
├── mac_bridge.c            # Macintosh Toolbox UI, event loop, TextEdit formatting, menus
├── mac_bridge.h            # GUI bridge API and command ID constants
└── README.md               # This documentation

build-config/classic_mac/
├── Makefile                # Cross-compilation Makefile using Retro68
├── rustid.r                # Toolbox resources (MBAR, MENU, WIND, ALRT, DITL, SIZE, BNDL, cfrg)
└── rustid_icons.r          # Multi-resolution icon suite (ICN#, ics#, icl8, ics8, icl4, ics4, ICON)
```

### Key Components

- [`ports/classic_mac/classic_mac_engine.c`](file:///home/tim/code/rustid/ports/classic_mac/classic_mac_engine.c): Implements CPU identification, bus/clock frequency detection, Gestalt querying, model name lookup, memory sizing, and report generation matching the modern `rustid` output formats.
- [`ports/classic_mac/mac_bridge.c`](file:///home/tim/code/rustid/ports/classic_mac/mac_bridge.c): Handles Macintosh Toolbox initialization, the main `WaitNextEvent` loop, window updates, TextEdit view rendering with styled color runs, clipboard export via Scrap Manager, and dialog handling.
- [`build-config/classic_mac/rustid.r`](file:///home/tim/code/rustid/build-config/classic_mac/rustid.r): Defines Macintosh resources compiled with `Rez`. Includes application signature `'RsId'`, bundle `'BNDL'`, file reference `'FREF'`, MultiFinder memory configuration `'SIZE'`, version stamps `'vers'`, and Code Fragment resource `'cfrg'`.
- [`build-config/classic_mac/rustid_icons.r`](file:///home/tim/code/rustid/build-config/classic_mac/rustid_icons.r): Contains complete multi-depth icon families:
  - 32x32: 1-bit (`'ICN#'`), 4-bit 16-color (`'icl4'`), 8-bit 256-color (`'icl8'`), and dialog icon (`'ICON'`).
  - 16x16: 1-bit small (`'ics#'`), 4-bit small (`'ics4'`), and 8-bit small (`'ics8'`).

---

## Prerequisites & Toolchain Setup

Building the Classic Mac OS port requires **Retro68**, a modern GCC-based cross-compilation suite targeting 68k and PowerPC Classic Mac OS.

### 1. Retro68 Toolchain

- **Repository**: [https://github.com/autc04/Retro68](https://github.com/autc04/Retro68)
- Install or build Retro68 on your Linux, macOS, or WSL host.
- The build expects the toolchain to be located at `$HOME/code/Retro68-build/toolchain` by default, or pointed to by the `RETRO68_PREFIX` environment variable.

### 2. Required Tools

Ensure the following Retro68 binaries are available in your `PATH` or located under `$RETRO68_PREFIX/bin`:

- `m68k-apple-macos-gcc` — Motorola 680x0 C compiler
- `powerpc-apple-macos-gcc` — PowerPC C compiler
- `MakePEF` — Preferred Executable Format (PEF) packager
- `Rez` — Macintosh Resource compiler

---

## Building

You can build the Classic Mac binaries using either `just` or `make`:

### Using `just`

```bash
just build-classic-mac
```

### Using `make`

```bash
make build-classic-mac
```

### Direct Makefile Invocation

```bash
export PATH="$HOME/code/Retro68-build/toolchain/bin:$PATH"
make -C build-config/classic_mac
```

To clean previous build artifacts:

```bash
make -C build-config/classic_mac clean
```

---

## Build Output & Artifacts

All built binaries are output to [`target/classic_mac/`](file:///home/tim/code/rustid/target/classic_mac/):

| File | Type | Target Hardware | Description |
|:-----|:-----|:----------------|:------------|
| `rustid.bin` | MacBinary II | 68k & PowerPC | **Universal Fat Binary**. Contains both 68k `CODE` and PowerPC PEF executable code. Runs natively on any Mac from System 6 to Mac OS 9. |
| `rustid.dsk` | 800 KB HFS Disk | 68k & PowerPC | Raw floppy disk image containing `rustid` Universal Fat Binary. Ready to mount in emulators. |
| `rustid_68k.bin` | MacBinary II | Motorola 680x0 | Standalone 68k binary optimized for 68000–68060 systems running System 6.0.8 – Mac OS 8.1. |
| `rustid_68k.dsk` | 800 KB HFS Disk | Motorola 680x0 | Raw floppy disk image containing the 68k standalone binary. |
| `rustid_ppc.bin` | MacBinary II | PowerPC | Standalone PowerPC PEF binary for System 7.1.2 – Mac OS 9.2.2. |
| `rustid_ppc.dsk` | 800 KB HFS Disk | PowerPC | Raw floppy disk image containing the PowerPC standalone binary. |

### File Formats Explained

- **MacBinary II (`.bin`)**: Classic Mac OS files consist of two forks: a *Data Fork* and a *Resource Fork*, along with Finder metadata (File Type `APPL` and Creator `RsId`). Modern operating systems and filesystems (ext4, NTFS, FAT32) do not support resource forks. MacBinary packages both forks and metadata into a single byte stream. Transferring `.bin` files ensures the application icon and code resources are not stripped.
- **Raw HFS Disk Image (`.dsk`)**: Standard 800 KB Macintosh HFS disk image. This can be mounted directly in emulators or written block-for-block to physical 3.5" DD floppy disks.

---

## Running and Testing

### 1. In Emulators

- **Basilisk II** (68k emulation):
  - Add `target/classic_mac/rustid.dsk` or `target/classic_mac/rustid_68k.dsk` to the volumes list in the Basilisk II GUI / `~/.basilisk_ii_prefs`.
  - Boot into System 7 or Mac OS 8. The `rustid` disk will appear on the desktop.
- **SheepShaver** (PowerPC emulation):
  - Add `target/classic_mac/rustid.dsk` or `target/classic_mac/rustid_ppc.dsk` to your SheepShaver volumes list.
  - Boot into System 7.5.3 – Mac OS 9.0.4.
- **Infinite Mac** (Web-based emulation):
  - Drag and drop `rustid.bin` or `rustid.dsk` directly into an [Infinite Mac](https://infinitemac.org) browser window.

### 2. On Real Vintage Hardware

To copy `rustid` to physical Macintosh hardware:

- **Floppy Disks**:
  Write the `.dsk` image to an 800 KB double-density disk using a USB floppy drive or Floppy Emu:
  ```bash
  dd if=target/classic_mac/rustid.dsk of=/dev/sdX bs=84k status=progress
  ```
- **SCSI Solid-State Drives (BlueSCSI, SCSI2SD, RaSCSI / PiSCSI)**:
  Mount the SD card on your host and copy `rustid.bin` onto the drive, or mount `rustid.dsk` as an auxiliary SCSI LUN/drive image.
- **Local Network / FTP / Web**:
  Download `rustid.bin` directly onto the classic Mac using Netscape Navigator, Internet Explorer, or an FTP client (like Fetch). Expand it using **StuffIt Expander** to restore the resource fork.
- **USB Storage (New World Macs)**:
  On iMacs, G3s, and G4s running Mac OS 8.6 or Mac OS 9, copy `rustid.bin` onto a FAT32/HFS USB flash drive, then unpack it on the Mac with StuffIt Expander.

---

## Technical Architecture & Hardware Detection

### Safe Gestalt Inquiries

Hardware detection relies on the Macintosh Toolbox `Gestalt` Manager. To maintain compatibility with older systems (such as early System 6 without Gestalt loaded, or System 7 systems with non-standard ROMs):

1. **68k Safe Traps**: On 68k, trap call `0xA1AD` is executed via inline assembly. If Gestalt is unavailable, the trap returns error code `0xA1AD` (`unimpErr`) rather than crashing.
2. **PPC Problem State Safety**: PowerPC Mac OS runs all third-party applications in user/problem mode ($MSR[PR] = 1$). Privileged supervisor-level instructions—such as `mfspr rD, 287` (PVR - Processor Version Register)—cause an immediate illegal instruction trap on Mac OS 8/9. This port probes the native CPU type strictly through `gestaltNativeCPUtype` (`'cput'`) and `gestaltSysArchitecture` (`'sysa'`).

### Gestalt Selectors Used

- `gestaltMachineType` (`'mach'`): Identifies the hardware model ID (e.g., Macintosh IIci, Quadra 700, PowerBook 180, Power Mac 8500).
- `gestaltNativeCPUtype` (`'cput'`): Differentiates native PowerPC 601, 603, 603e, 604, 604e, 750 (G3), and 7400/7450 (G4) CPUs.
- `gestaltProcessorType` (`'proc'`): Differentiates Motorola 68000, 68010, 68020, 68030, 68040, and 68060.
- `gestaltFPUType` (`'fpu '`): Detects presence of 68881, 68882, or integrated 68040 floating-point hardware.
- `gestaltMMUType` (`'mmu '`): Detects Apple AMU, Motorola 68851 PMMU, or integrated 68030/68040 MMU.
- `gestaltProcClkSpeed` (`'pclk'`) & `gestaltBusClkSpeed` (`'bclk'`): Accurately reports CPU core frequency and bus speed.
- `gestaltPhysicalRAMSize` (`'ram '`): Queries total installed physical RAM.
- `gestaltSystemVersion` (`'sysv'`): Decodes system software version (e.g., System 7.1, System 7.5.5, Mac OS 8.6, Mac OS 9.2.2).
- `gestaltUserVisibleMachineName` (`'mnam'`): Reads the localized Pascal string for the official Apple marketing name.

### Model Specification Fallback Table

On early System 6 and 7 versions where `'pclk'`, `'bclk'`, or `'mnam'` are not populated by the system ROM, the engine uses a built-in hardware lookup table keyed by `gestaltMachineType`. This table maps over 50 classic Macintosh models to their factory CPU family, clock speed, bus speed, and standard coprocessor configuration.

---

## User Interface & Controls

### Menu Bar & Shortcuts

| Menu | Command | Shortcut | Function |
|:-----|:--------|:---------|:---------|
| **Apple** | About Rustid... | — | Displays the About box with the application icon and version. |
| **File** | Refresh | `Cmd + R` | Re-runs hardware detection and updates the report. |
| **File** | Quit | `Cmd + Q` | Exits the application and returns to the Finder. |
| **Edit** | Copy | `Cmd + C` | Copies the entire hardware report text to the Mac clipboard (Scrap). |
| **View** | Standard | `Cmd + 1` | Displays standard summary report (System, Architecture, Model, MicroArch, Clocks, RAM). |
| **View** | Debug | `Cmd + 2` | Displays detailed technical report with Raw Gestalt selector values. |
| **View** | Everything | `Cmd + 3` | Displays full combined diagnostic dump. |
| **View** | Colors | — | Toggles color text syntax highlighting on Color QuickDraw systems. |

### TextEdit Color Runs & Typography

- Output text is displayed in 9-point **Monaco** inside a Macintosh TextEdit (`TEHandle`) control.
- Text lines use standard Classic Mac carriage returns (`\r` / `0x0D`). Line feeds (`\n`) are automatically converted on ingestion to preserve TextEdit line wrap and style offsets.
- When colors are enabled, label titles are rendered in forest green, subcategories in steel blue, and alert values in crimson.

### Memory & MultiFinder Specification

The application includes a `SIZE` resource configured with:
- **Minimum Partition Size**: 1,024 KB
- **Preferred Partition Size**: 2,048 KB
- Flags set: `is32BitCompatible`, `multiFinderAware`, `acceptSuspendResumeEvents`, and `backgroundAndForeground`.
