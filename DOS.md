# DOS Version

This document covers building, using, and understanding the DOS version of rustid.

## Overview

The DOS port of rustid compiles to a 16-bit real-mode MZ executable that runs on 386-class (or better) DOS systems. It uses Rust's `no_std` environment with a custom target spec, a bump allocator, and DOS software interrupts for console I/O.

Three binaries are produced:

| Binary | Cargo bin | Purpose |
|--------|-----------|---------|
| `rustid.exe` | `dos_rustid` | Main CPU identification (formatted table output) |
| `dump.exe` | `dump` | Raw CPUID register dump per logical core |
| `debug.exe` | `debug` | Rust `Debug`-formatted output + quirk diagnostics |

## Prerequisites

- **Nightly Rust toolchain** — DOS build requires `-Z build-std` and `-Zjson-target-spec`, both unstable
- **`rust-src` component** — needed for `build-std`:
  ```
  rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
  ```
- **`rust-lld`** — the custom target uses `linker-flavor = "ld.lld"` (bundled with Rust)
- **`tools/make_exe`** — a bundled ELF-to-MZ-EXE converter, built automatically as part of the DOS build
- **DOSBox-X** (optional) — for running/testing the DOS binaries without real hardware

## Building

### Build all DOS binaries

```bash
just build-dos
# or: make build-dos
```

This will:
1. Build `dos_rustid`, `dump`, and `debug` using the `i486-dos.json` target spec
2. Convert each ELF binary to MZ EXE format via `tools/make_exe`
3. Run the binary size test

The resulting `rustid.exe`, `dump.exe`, and `debug.exe` appear in the project root.

## Running

### In DOSBox-X

```bash
just run-dos
# or: make run-dos
```

Launches `rustid.exe` in DOSBox-X with the config at `tools/dosbox-x.conf`.

### Automated test in DOSBox-X

```bash
just test-dos
# or: make test-dos
```

Runs with a 2-second time limit and logs output.

### On real hardware

Copy the `.exe` files to a DOS system and run:

```
C:\> rustid.exe
C:\> dump.exe
C:\> debug.exe
```

## How It Works

### Target Specification (`i486-dos.json`)

The custom target `i486-unknown-none-code16` targets a 386-class CPU in 16-bit real mode:

- `arch = "x86"`, `cpu = "i386"` (broadest compatibility)
- `llvm-target = "i486-unknown-none-code16"`
- `os = "none"` (no OS, bare metal)
- `relocation-model = "static"`
- `panic-strategy = "abort"`
- `max-atomic-width = 32`

The `cfg(dos)` conditional is activated via `build.rs`:
```rust
cfg_aliases! {
    dos: { all(target_os = "none", target_arch = "x86") },
}
```

### Entry Point

All three binaries use the same entry sequence in `.code16` real-mode assembly:

```asm
mov ax, cs
mov ds, ax
mov es, ax
mov ss, ax
.byte 0x66, 0x0F, 0xB7, 0xE4    ; movzx esp, sp (32-bit prefix)
.byte 0xE9
.word rust_main - 1f             ; manual 16-bit near jump
```

The linker script (`link-exe.x`) places this in the `.startup` section at offset `0x10`, right after a 6-byte metadata header (data seg offset, stack seg offset, stack size).

### EXE Conversion (`tools/make_exe`)

The Rust tool at `tools/make_exe/src/main.rs` converts the ELF binary produced by the linker into a DOS MZ executable:

1. Parses ELF32/ELF64 headers manually (no external ELF crate)
2. Extracts `PT_LOAD` segments into a flat binary
3. Reads 6 bytes of metadata: `[data_seg, stack_seg, stack_size]`
4. Writes an MZ header with `IP = 0x0010`, `CS = 0x0000`, `SS`/`SP` from metadata, and `min_alloc = 0x1000` (64KB heap)

### Allocator

A simple bump allocator is used (`src/cpuid/dos/allocator.rs`):
- Initialized to ~64KB (rest of the current 64KB segment after the binary)
- Non-atomic operations for 386 compatibility (no `CMPXCHG`)
- No deallocation (sufficient for rustid's few long-lived allocations)
- `DosAllocator` marked `unsafe impl Sync` — DOS is single-threaded

### Console I/O & Exit

All output goes through DOS software interrupts:

- **`printc()`** — `INT 21h, AH = 02h` with character in `DL`
- **`exit()`** — `INT 21h, AH = 4Ch` with exit code in `AL`
- The `print!` / `println!` macros wrap these, with an optimization for literal string arguments

### CPU Detection

- Uses the real `CPUID` instruction directly (`src/cpuid/fns.rs`) via inline asm
- For pre-CPUID CPUs (386/486): falls back to a **CPU reset signature** technique — sets the CMOS shutdown byte to `0x0A` (jump to `40:67` after reset), writes the warm boot vector, triggers reset via port `0x92` (with keyboard controller fallback), and captures `EDX` (which contains the CPU signature on 386/486 after reset)
- Cyrix-specific detection uses I/O ports `0x22`/`0x23` to read Configuration Control Registers (`src/cpuid/vendor/cyrix.rs`)

### Frequency Measurement

CPU frequency is measured using `RDTSC` + PIT Channel 0 + BIOS timer tick (`0040:006C`) for about 110ms. For pre-TSC CPUs (386/486), a calibrated instruction loop runs over 8 BIOS ticks (~440ms) with different cycle counts per loop iteration depending on the CPU type (386: 29, 486: 10, Cyrix: 14, UMC: 10, RapidCAD: 20). Frequency is derived from the ratio of TSC delta or instruction count to elapsed PIT pulses.

### MP Table Scanning

For multi-socket detection (`src/cpuid/mp.rs`), the DOS version scans BIOS memory for the Intel MP Specification `_MP_` floating pointer structure, using `peek_u8`/`peek_u16` for safe segmented memory access. Falls back to reading the EBDA segment via `INT 15h, AX = C100h`.

## Quirks & Limitations

### Binary Size
- The DOS EXE must stay under ~62KB (64KB segment minus header). A test verifies this.
- Currently the binaries are well under this limit.

### Segment Model
- Everything (code, data, stack, heap) lives in a single 64KB segment
- The EXE loader maps CS = DS = ES = SS

### Pre-CPUID CPUs (386/486)
- Detection relies on performing an actual CPU reset via CMOS/port 0x92. This is:
  - **Extremely disruptive** — the CPU actually resets
  - **Only works in real mode** — will crash in protected/V8086 mode
  - **Only works on certain chipsets** — port 0x92 and CMOS shutdown must be supported
  - Used only as a last resort when CPUID is not available
- Without the reset method, pre-CPUID chips show limited info

### Cyrix CPUs
- Some Cyrix 6x86 processors have CPUID support **disabled by default** in the chip
- If detected, rustid prints a message directing users to a third-party utility to enable it
- This must be done before running rustid

### Simplified Feature Display
- Under DOS, features are displayed as a single "Base" category (flat list) rather than grouped by SSE/AVX/AVX512/Math/Security
- The feature list is also limited to save binary space

### No Hypervisor Detection
- Hypervisor vendor detection is suppressed under DOS (unlikely to be running under a hypervisor)

### No OS-Specific Features
- No `/proc/cpuinfo` parsing
- No system calls or sysctl
- No thread/core pinning (DOS is single-tasking)
- Socket count uses MP Table scanning only (limited to Intel/Vortex/Centaur per spec)

### Frequency Accuracy
- Uses PIT + BIOS timer ticks (~54.9ms each) — less precise than OS-level methods
- Pre-TSC measurement uses a calibrated busy loop over ~440ms

### Allocator
- Bump allocator only; no deallocation (`free` is a no-op)
- Fine for rustid's usage pattern (few long-lived allocations)

### Requires Nightly Rust
- `-Z build-std` and `-Zjson-target-spec` are unstable features
- May break if the nightly toolchain changes these interfaces

### Supported CPUs
- 386-class (or better) with DOS-compatible BIOS
- Tested on real hardware: 386, 486, Pentium, Pentium II/III/4, AMD K6/Athlon, various embedded SoCs

## Testing

### Binary size test
```bash
cargo test --test dos_binary_size_test --features dos-build
```
Verifies each `.exe` is under 62KB.

### DOSBox-X integration test
```bash
just test-dos
```
Builds and runs in DOSBox-X with a 2-second time limit and console logging.

### Manual testing
```bash
just run-dos
```
Builds and runs `rustid.exe` interactively.

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

### `dump.exe`

Dumps raw CPUID register values (EAX, EBX, ECX, EDX) for each CPUID leaf and subleaf, per logical core.

### `debug.exe`

Outputs Rust `Debug`-formatted representation of the `Cpu` struct and Cyrix vendor info (if applicable), plus quirk diagnostic information.
