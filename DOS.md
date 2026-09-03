# DOS Version

This document covers building, using, and understanding the DOS version of rustid.

## Overview

The DOS port of rustid supports two execution environments:
1. **32-bit Protected Mode (DOS32A Extender)** — Primary target for 386+ CPUs supporting CPUID and DOS extender operation. Uses the DOS32A DOS extender (`tools/dos32a/dos32a.exe`) bound into Linear Executable (LE) format via `tools/elf2le`.
2. **16-bit Real Mode Fallback** — 16-bit real-mode MZ executable for pre-CPUID CPUs (386/486) or environments requiring real-mode CPU reset identification.

Binaries produced:

| Binary | Cargo bin | Mode | Purpose |
|--------|-----------|------|---------|
| `rustid.exe` | `dos` | 32-bit DOS32A | Main CPU identification (formatted table output) |
| `rust86.exe` | `rust86` | 16-bit Real Mode | Real-mode fallback CPU identification & debug diagnostics (`/D`) |

**Note**: You will only need to directly run `rustid.exe`, as `rust86.exe` will be executed automatically if running on a pre-CPUID CPU.

## Prerequisites

- **Nightly Rust toolchain** — DOS build requires `-Z build-std` and `-Zjson-target-spec`, both unstable
- **`rust-src` component** — needed for `build-std`:
  ```
  rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
  ```
- **`tools/elf2le`** — bundled ELF-to-LE converter for DOS32A binaries
- **`tools/make_exe`** — bundled ELF-to-MZ-EXE converter for 16-bit real-mode binaries
- **DOSBox-X** — required for building `rustid.exe` (runs DOS32A patching/binding scripts in DOS environment) and running/testing DOS binaries

## Building

### Build all DOS binaries

```bash
just build-dos
# or: make build-dos
```

This will:
1. Build 32-bit protected mode binary, converting via `tools/elf2le` and invoking `dos32a.exe` inside DOSBox-X to generate `rustid.exe`
2. Build 16-bit real-mode binary and convert via `tools/make_exe` into `rust86.exe`
3. Run the binary size test

The resulting executables (`rustid.exe`, `rust86.exe`) appear in the project root.

## Running

- **In DOSBox-X**: `just run-dos` or `make run-dos` (launches `rustid.exe` using `tools/dosbox-x.conf`)
- **Automated test run**: `just test-dos` or `make test-dos` (runs in DOSBox-X with console logging)
- **On real hardware**: Copy `.exe` files to a DOS system and execute `rustid.exe`.

## How It Works

### EXE Conversion (`tools/make_exe` & `tools/elf2le`)

- **`tools/make_exe`**: Converts ELF binaries produced by `rust-lld` into 16-bit DOS MZ executables.
- **`tools/elf2le`**: Converts 32-bit ELF binaries into LE format and binds them with DOS32A (`tools/dos32a/dos32a.exe`) inside DOSBox-X to generate 32-bit extender binaries (`rustid.exe`).

### Allocator

A simple bump allocator is used (`src/x86/dos/allocator.rs`):
- Initialized to memory segment after the binary
- Non-atomic operations for 386 compatibility (no `CMPXCHG`)
- No deallocation (sufficient for rustid's few allocations)
- `DosAllocator` marked `unsafe impl Sync` — DOS is single-threaded

### Console I/O & Exit

All output goes through DOS software interrupts:

- **`printc()`** — `INT 21h, AH = 02h` with character in `DL`
- **`exit()`** — `INT 21h, AH = 4Ch` with exit code in `AL`
- The `print!` / `println!` macros wrap these, with an optimization for literal string arguments

### CPU Detection

- Uses the real `CPUID` instruction directly (`src/x86/fns.rs`) via inline asm
- For pre-CPUID CPUs (386/486): falls back to a **CPU reset signature** technique in 16-bit real-mode — sets the CMOS shutdown byte to `0x0A` (jump to `40:67` after reset), writes the warm boot vector, triggers reset via port `0x92` (with keyboard controller fallback), and captures `EDX` (which contains the CPU signature on 386/486 after reset)
- Cyrix-specific detection uses I/O ports `0x22`/`0x23` to read Configuration Control Registers (`src/x86/vendor/cyrix.rs`)

### Frequency Measurement

CPU frequency is measured using `RDTSC` + PIT Channel 0 + BIOS timer tick (`0040:006C`) for about 110ms. For pre-TSC CPUs (386/486), a calibrated instruction loop runs over 8 BIOS ticks (~440ms) with different cycle counts per loop iteration depending on the CPU type. Frequency is derived from the ratio of TSC delta or instruction count to elapsed PIT pulses.

### MP Table Scanning

For multi-socket detection (`src/x86/mp.rs`), the DOS version scans BIOS memory for the Intel MP Specification `_MP_` floating pointer structure, using `peek_u8`/`peek_u16` for safe segmented memory access. Falls back to reading the EBDA segment via `INT 15h, AX = C100h`.

## Quirks & Limitations

### Binary Size
- The real-mode dos binary (`rust86.exe`) must stay under ~62KB (64KB segment minus header). A test verifies this.
- The new rustid.exe overcomes this limit with the dos32a extender

### Pre-CPUID CPUs (386/486)
- Detection relies on performing an actual CPU reset via CMOS/port 0x92. This is:
  - **Extremely disruptive** — the CPU actually resets
  - **Only works in real mode** — will crash in protected/V8086 mode
  - Used only as a last resort when CPUID is not available
- Without the reset method, pre-CPUID chips show limited info

### Frequency Accuracy
- Uses PIT + BIOS timer ticks (~54.9ms each) — less precise than OS-level methods
- Pre-TSC measurement uses a calibrated busy loop over ~440ms

### Requires Nightly Rust
- `-Z build-std` and `-Zjson-target-spec` are unstable features
- May break if the nightly toolchain changes these interfaces

### Supported CPUs
- 386-class (or better) with DOS-compatible BIOS
- Tested on real hardware: 386, 486, Pentium, Pentium II/III/4, AMD K6/Athlon, various embedded SoCs

## Testing

- **Binary size test**: `cargo test --test dos_binary_size_test --features dos-build` (Verifies real-mode binaries stay under 62KB limit)
- **DOSBox-X integration test**: `just test-dos` (Builds and runs `rustid.exe` in DOSBox-X with console logging)
- **Interactive test**: `just run-dos` (Launches `rustid.exe` in DOSBox-X)

## Example Output

### `rustid.exe` (AMD K6-2)

```
--------------- Rustid 1.0.0 (x86-DOS) ---------------
  Architecture: i586
        Vendor: AuthenticAMD (AMD)
         Model: AMD-K6(tm) 3D processor
     MicroArch: K6
      Codename: Chompers/CXT
  Process Node: 250nm
    Easter Egg: NexGenerationAMD
         Cache: L1d: 32 KB, 2-way
                L1i: 32 KB, 2-way
     Frequency: 500.00 MHz
     Signature: Family 5h, Model 8h, Stepping Ch
                (0, 5, 0, 8, 12)
      Features: FPU TSC CMPXCHG8B MMX 3DNow!
```

### `rustid.exe` (Vortex86DX3 — showing multi-socket topology)

```
--------------- Rustid 1.0.0 (x86-DOS) ---------------
  Architecture: i686-SSE
        Vendor: Vortex86 SoC (DM&P)
         Model: Vortex86DX3
     MicroArch: Vortex86DX3
  Process Node: 40nm
      Topology: 2 sockets, 2 cores, 2 threads
         Cache: L1d: 2x 16 KB, 4-way
                L1i: 2x 16 KB, 4-way
                L2:  2x 256 KB, 4-way
     Frequency: 1.00 GHz
     Signature: Family 6h, Model 1h, Stepping 1h
                (0, 6, 0, 1, 1)
      Features: FPU TSC CMPXCHG8B CMOV MMX SSE
```
