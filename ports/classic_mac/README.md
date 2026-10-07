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
| **PowerPC 601 / 603 / 604 / G3 / G4** (Classic) | System 7.1.2 | Mac OS 9.2.2 | Runs as native CFM PEF code linked to `InterfaceLib`. Backward-compatible with vintage System 7 ROMs. |
| **PowerPC G3 / G4 / G5** (Carbon CFM) | Mac OS 8.6 | Mac OS 9.2.2 | Standalone Carbon binary (`rustid_carbon.bin`) linked to `CarbonLib`. |
| **PowerPC G3 / G4 / G5** (Mac OS X) | Mac OS X 10.0 | Mac OS X 10.5 | Native Aqua CFM application bundle (`Rustid.app`). Runs natively without launching Classic Environment. |

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
| `rustid_ppc.bin` | MacBinary II | PowerPC | Standalone PowerPC PEF binary linked to `InterfaceLib` for System 7.1.2 – Mac OS 9.2.2. |
| `rustid_ppc.dsk` | 800 KB HFS Disk | PowerPC | Raw floppy disk image containing the PowerPC standalone binary. |
| `rustid_carbon.bin` | MacBinary II | PowerPC | Carbon CFM binary linked to `CarbonLib` with `'carb'` resource. Runs on Mac OS 8.6 – 9.2.2 with CarbonLib. |
| `rustid_carbon.dsk` | 800 KB HFS Disk | PowerPC | Raw floppy disk image containing the Carbon standalone binary. |
| `rustid.icns` | Apple ICNS | Universal | Universal icon containing Classic QuickDraw (`ICN#`, `icl8`, `ics#`, `ics8`), OS X 10.0–10.4 32-bit (`it32`/`t8mk`, etc.), and modern PNG chunks (`ic07`–`ic14`). |
| `Rustid.app` | Hybrid Bundle | PowerPC (G3/G4/G5) | **Hybrid Application Bundle**. Double-clickable on Mac OS X Aqua (launches `Contents/MacOS/rustid` with Aqua icon). On Mac OS 9, displays custom app icon via `kHasCustomIcon` and contains double-clickable `Rustid` alongside `Contents/MacOSClassic/rustid`. |
| `rustid_hybrid.dmg` | 14 MB Apple Disk Image | PowerPC & 68k | **Native Mac OS X Apple Disk Image (`.dmg`)**. Mounts natively on OS X (10.0 Cheetah through 10.5 Leopard) via `DiskImageMounter.app` on double-click. Contains `Rustid.app` (with custom folder icon), `Rustid (Carbon)`, `Rustid (Classic PPC)`, and `Rustid (Universal Fat)` with intact HFS resource forks. |
| `rustid_hybrid.dsk` | 14 MB HFS Disk | PowerPC & 68k | **Classic Emulator Multi-OS Disk Image**. Mounts directly in SheepShaver, Basilisk II, QEMU, Floppy Emu, and SCSI devices. Identical content to `rustid_hybrid.dmg`. |
| `Rustid-osx-ppc.tar.gz` | Gzip Tar Archive | PowerPC (G3/G4/G5) | Compressed distribution archive of `Rustid.app` for Mac OS X PowerPC systems. |

### File Formats & Dual-OS Compatibility Explained

- **Historical Hybrid Application Bundles (`.app`) on Mac OS 9 vs Mac OS X**:
  - During the OS X transition era (used in commercial titles like *Super Collapse! II* and documented in Apple's *Inside Mac OS X: System Overview*), applications targeting both Mac OS 9 and Mac OS X were packaged in `.app` bundles:
    - **Mac OS X Aqua**: The Finder recognizes the bundle directory, launches `Contents/MacOS/rustid`, and renders `Contents/Resources/rustid.icns`.
    - **Mac OS 9 Finder**: Classic Mac OS has no native directory execution handler; it opens `.app` as a folder. Following the Apple/Super Collapse 2 convention:
      1. An exact-named executable (`Rustid`) is placed directly at the bundle root alongside `Contents/`, equipped with full resource forks (`cfrg`, `carb`, `SIZE`, `BNDL`, `ICN#`) for immediate double-click launching.
      2. The Classic CFM slice is also housed in `Contents/MacOSClassic/rustid` for `LaunchCFMApp` / Carbon compatibility.
      3. The bundle folder itself receives custom application iconography in Mac OS 9 via an invisible `Icon\r` file (resource ID `-16455`) and the Finder's `kHasCustomIcon` (`0x0004`) catalog bit.
- **Apple Disk Image (`.dmg`) vs Legacy Emulator Images (`.dsk`)**:
  - **Mac OS X**: `.dmg` (Apple Disk Image) is the native disk image format registered to `/System/Library/CoreServices/DiskImageMounter.app` and `hdiutil`. Double-clicking `rustid_hybrid.dmg` mounts the volume directly onto the desktop. Mac OS X does not associate `.dsk` files with any application by default.
  - **Classic Emulators**: Software emulators (SheepShaver, Basilisk II, Mini vMac) and hardware emulators (Floppy Emu, BlueSCSI) typically expect `.dsk` or `.img` files. Both `rustid_hybrid.dmg` and `rustid_hybrid.dsk` are provided with identical sector layouts.
- **Resource Fork Preservation**:
  - Classic Mac OS CFM binaries require an intact resource fork containing `'cfrg'` (Code Fragment Configuration) and `'carb'` (Carbon) resources.
  - If files are transferred over non-HFS filesystems (FAT32, ext4, standard tar archives, or emulator shared folders), the resource fork is detached, preventing Mac OS 9 from executing the binary.
  - To guarantee 100% working execution on Mac OS 9:
    1. Mount **`rustid_hybrid.dmg`** / **`rustid_hybrid.dsk`** or **`rustid_carbon.dsk`** directly in your emulator or disk mounter (all resource forks are natively stored in HFS).
    2. Alternatively, expand **`rustid_carbon.bin`** or **`Rustid.bin`** using **StuffIt Expander** on the Mac.
- **CarbonLib vs InterfaceLib**:
  - `Rustid (Carbon)` and `rustid_carbon.bin` link to `CarbonLib`. They require `CarbonLib` (v1.0.4 through v1.6) to be installed in `System Folder:Extensions` on Mac OS 8.6–9.2.2.
  - `Rustid (Classic PPC)` and `rustid_ppc.bin` link to `InterfaceLib`. They run on any PowerPC Macintosh running System 7.1.2 through Mac OS 9.2.2 with **zero extensions required**.
  - `Rustid (Universal Fat)` runs natively on both 68k and PowerPC without CarbonLib.

---

## Running and Testing

### 1. On Mac OS X (PowerPC 10.0 Cheetah – 10.5 Leopard)

- **Native Apple Disk Image (`.dmg`)**:
  - Double-click **`target/classic_mac/rustid_hybrid.dmg`** in Finder. Mac OS X's native `DiskImageMounter.app` mounts the volume `Rustid Hybrid` to your desktop.
  - Drag `Rustid.app` into `/Applications` or run it directly. It executes with native Aqua styling, Carbon window controls, and Aqua application icon.
- **Tarball Archive**:
  - Alternatively, extract **`target/classic_mac/Rustid-osx-ppc.tar.gz`** using Archive Utility or `tar -xzf`.

### 2. In Classic Emulators

- **SheepShaver** (PowerPC emulation):
  - Add `target/classic_mac/rustid_hybrid.dmg` (or `rustid_hybrid.dsk` / `rustid.dsk`) to your SheepShaver volumes list.
  - Boot into System 7.5.3 – Mac OS 9.0.4.
- **Basilisk II** (68k emulation):
  - Add `target/classic_mac/rustid.dsk` or `target/classic_mac/rustid_68k.dsk` to the volumes list in the Basilisk II GUI / `~/.basilisk_ii_prefs`.
  - Boot into System 7 or Mac OS 8. The `rustid` disk will appear on the desktop.
- **Infinite Mac** (Web-based emulation):
  - Drag and drop `rustid.bin` or `rustid.dsk` directly into an [Infinite Mac](https://infinitemac.org) browser window.

### 3. On Real Vintage Hardware

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

### Open Firmware Device-Tree & Model Identifier Probing

On PCI and NewWorld PowerPC Macs (introduced starting with the Bondi Blue iMac in 1998 through the G4 and G5 era), Apple changed how hardware is reported:
- The Gestalt machine type (`gestaltMachineType` / `'mach'`) returns a generic ID of `406` (`gestaltPowerMacNewWorld`) across nearly all NewWorld hardware.
- Gestalt alone cannot distinguish an iMac G3 from a Power Mac G4 Cube, a Titanium PowerBook G4, or a dual-processor Mirrored Drive Doors G4.

To solve this, `rustid` dynamically probes the Open Firmware device tree for the hardware model identifier:
1. **Dynamic Name Registry Binding**: Queries the Code Fragment Manager (CFM) via `GetSharedLibrary("NameRegistryLib")` and resolves `RegistryCStrEntryLookup` and `RegistryPropertyGet`.
   - *Zero NuBus or 68k Breakage*: By loading dynamically rather than hard-linking against `libNameRegistryLib.a`, `rustid` avoids PEF load errors (`cfragNoLibraryErr`) on early NuBus PowerPC Macs (such as the Power Macintosh 6100, 7100, and 8100) and 680x0 machines where the Name Registry is absent.
2. **Device Tree Node Traversal**: Looks up the root `"Devices:device-tree"` node and inspects the `"compatible"` property (a sequence of null-terminated C strings) followed by `"model"`.
3. **Model Identifier Resolution**: Matches model identifier strings (e.g. `PowerMac1,1`, `PowerMac3,6`, `PowerBook3,2`, `iMac,1`, `RackMac1,1`, or OldWorld PCI entries like `AAPL,PowerMac G3` and `AAPL,7500`) against a comprehensive internal hardware catalog.
4. **Hardware Specification Tuning**: Provides factory clock speeds, bus frequencies (such as 66 MHz, 100 MHz, 133 MHz, and 167 MHz system buses), and specific G4 CPU revisions (distinguishing MPC7400, MPC7410, MPC7450, and MPC7455) for identified models.
5. **Detection Hierarchy**:
   - Device Tree Model Identifier (via Name Registry)
   - Localized Gestalt Machine Name (`gestaltUserVisibleMachineName` / `'mnam'`)
   - Classic Gestalt Machine ID (`gestaltMachineType` / `'mach'`)

### Model Specification Fallback Table

On systems where Open Firmware device-tree identifiers or modern Gestalt selectors (`'pclk'`, `'bclk'`, `'mnam'`) are not available, the engine uses a built-in hardware lookup table. This table maps both numeric `gestaltMachineType` codes and Open Firmware identifier strings to their factory CPU family, core clock speed, bus speed, and standard coprocessor configuration.

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

### Resizable Window & Scrolling

- **Clean Full-Window Layout**: The bottom status bar has been eliminated to maximize report viewing area. The TextEdit view extends the full height of the window.
- **Native Vertical Scroll Bar (`scrollBarProc`)**:
  - A standard 16-pixel Macintosh scroll bar control docks along the right margin of the window.
  - **Thumb Dragging (`inThumb`)**: Smoothly drag the scroll box to scrub through the report.
  - **Scroll Arrows (`inUpButton` / `inDownButton`)**: Click or hold to scroll line-by-line via continuous `TrackControl` tracking.
  - **Page Scrolling (`inPageUp` / `inPageDown`)**: Click in the scroll track to jump one page up or down.
  - **Automatic Hilite State**: The scroll bar automatically ghost-deactivates (`HiliteControl(255)`) when the entire report fits within the visible window and activates (`HiliteControl(0)`) with calculated bounds when content exceeds the view.
- **Window Sizing & Zooming**: The main window uses `zoomDocProc`, featuring a native Macintosh size box (grow icon) in the bottom-right corner and a standard title-bar zoom box.
  - **Grow Box (`inGrow`)**: Click and drag the bottom-right size box to freely resize the window (minimum bounds: 380x200). The vertical scroll bar automatically resizes to leave room for the 15x15 grow icon.
  - **Zoom Box (`inZoomIn` / `inZoomOut`)**: Click the zoom box in the title bar to toggle between user size and full-screen maximization.
  - Dynamic recalculation updates the TextEdit display (`viewRect` and `destRect`), reflows lines via `TECalText`, and recalculates scroll bar maximums.
- **Keyboard Navigation**:
  - `Up Arrow` / `Down Arrow`: Scroll vertically line-by-line, updating the scroll bar thumb in real time.
  - `Page Up` / `Page Down`: Scroll by full pages, updating the scroll bar thumb in real time.

### Memory & MultiFinder Specification

The application includes a `SIZE` resource configured with:
- **Minimum Partition Size**: 1,024 KB
- **Preferred Partition Size**: 2,048 KB
- Flags set: `is32BitCompatible`, `multiFinderAware`, `acceptSuspendResumeEvents`, and `backgroundAndForeground`.
