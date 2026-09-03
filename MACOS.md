# macOS Build Support

This document describes how to build `rustid` for macOS, both the classic CLI
and the native AppKit GUI (`rustid-gui`), and explains why the older PowerPC
(PPC/PPC64) Macs cannot be targeted with the current Rust toolchain.

## Target coverage

The macOS GUI and CLI are built for the **two architectures Apple currently
supports**:

| Slice   | Architecture | Notes                              |
| ------- | ------------ | ----------------------------------- |
| Apple   | `arm64`      | Apple Silicon M1/M2/M3/M4           |
| Intel   | `x86_64`     | Intel Macs (and Rosetta 2)          |

The GUI application (`Rustid.app`) is distributed as a **universal ("fat")
binary** containing both slices so a single bundle runs on any modern Mac.

## Why PPC / PPC64 macOS is not supported

Early Intel transition-era Macs still on PowerPC (G3/G4/G5) **cannot** run the
Rust binaries produced today. This is a toolchain limitation, not a choice:

1. **LLVM removed the `powerpc{,64}-apple-darwin` backends in 2020.**
   `rustc` produces machine code through LLVM, and since LLVM 11 the PowerPC
   Darwin targets were deleted. Every current Rust compiler therefore has no
   way to emit PPC Mach-O object code. There are no `aarch64`-like PPC
   alternatives — `powerpc-apple-darwin` was a full architecture backend that
   is simply gone.

2. **rustup never shipped the PPC Darwin targets.** No target triple matching
   `powerpc*-apple-darwin` is published by the Rust project, so even the
   standard `rustup target add ...` fails.

3. **`ld64.lld`'s Mach-O port has no PPC.** Even if object code existed, the
   linker that assembles Mach-O universal binaries does not understand the
   PPC instruction set.

4. **A Linux ELF cannot be turned into a runnable PPC Mach-O.**
   `objcopy`/`objdump` cannot retarget executables across architectures and
   endianness — that requires re-linking from object files with a

   Darwin-PPC-aware linker against a PowerPC macOS SDK (10.4/10.5).

### Roadmap: a hybrid GCC-PPC build (research only)

Building a *genuine* PPC molecule would require abandoning Rust for the PPC
slice:

- Provide a **PowerPC macOS SDK** (10.4/10.5) — Apple no longer distributes
  these; you must obtain a legal copy of the SDK headers/libraries yourself.
- Compile the CPU-ID / info-gathering C sources for Darwin-PPC with an
  appropriate **GCC cross toolchain** (e.g. `gcc-4.x` retargeted to
  `powerpc-apple-darwin`, or newer GCC with a PPC target and a Darwin Mach-O
  linker such as `ld64`).
- Combine the resulting PPC Mach-O with the `arm64`/`x86_64` Rust slices using
  a **Mach-O fat/universal tool** (`lipo` equivalent, e.g. the `fat-macho`
  crate used below).

This is a large amount of C tooling work with no Rust involvement, so it is a
documented futher-research item rather than a shipped feature.

## Prerequisites

There are two supported ways to build for macOS:

**Natively on a Mac.** No extra toolchain is needed — Apple's `clang`/`lipo`
already handle both `x86_64` and `arm64`. The `build-mac-gui` recipe and
`build-config/build-macos.sh` detect that we are on a Mac and point Cargo at
the system `clang` automatically (see below).

**Cross-compiling from Linux.** Because macOS is the only target with a
proprietary SDK requirement, the standard `cargo-xcross` Docker images are
**not published** for Darwin. You must build a local cross environment once:

- **`osxcross`** — installs a macOS SDK and an `llvm-dsymutil`/`ld64` chain
  usable from Linux. See <https://github.com/tpoechtrager/osxcross>.
  - Provide a legal macOS SDK tarball when prompted (Xcode `MacOSX*.sdk`).
- Set up `osxcross` so that `cargo` can find the linker, e.g.:
  ```bash
  export PATH="$HOME/osxcross/target/bin:$PATH"
  export CC_x86_64_apple_darwin=o64-clang
  export CC_aarch64_apple_darwin=o64-clang
  ```
  or add the equivalent `target.<triple>.linker` to `.cargo/config.toml`.

## Building

The only thing required beyond the toolchain is Cargo's own declared macOS
dependencies (`objc2`, `objc2-foundation`, `objc2-app-kit`), which are
activated automatically on macOS targets.

**Recommended: one-shot bundle build.** The `build-mac-gui` recipe (see
`justfile`) drives `build-config/build-macos.sh`, which builds both slices,
merges them with `lipo`, assembles `Rustid.app`, and zips it as
`target/dist/rustid-macos.zip`. On a Mac it automatically sets
`CARGO_TARGET_*_APPLE_DARWIN_LINKER=clang` so the `.cargo/config.toml`
osxcross wrappers are not required; on Linux it relies on the osxcross
wrappers in `PATH`.

Building the slices manually works too (a `.cargo/config.toml` sets the
osxcross wrapper linkers, so override them with `clang` when building natively
on a Mac):

**CLI (both slices):**
```bash
rustup target add x86_64-apple-darwin aarch64-apple-darwin
cargo build --target x86_64-apple-darwin --release
cargo build --target aarch64-apple-darwin --release
```

**GUI (`rustid-gui`, both slices):**
```bash
cargo build --target x86_64-apple-darwin --features gui --bin rustid-gui --release
cargo build --target aarch64-apple-darwin --features gui --bin rustid-gui --release
```

Type-checking (no SDK/linker required) works with plain `cargo check`:
```bash
cargo check --target aarch64-apple-darwin --features gui --bin rustid-gui
cargo check --target x86_64-apple-darwin --features gui --bin rustid-gui
```

### Combine into a universal binary

The two slices are merged into a single fat Mach-O using the `fat-macho`
helper in `tools/make_fat`:

```bash
cargo run --manifest-path tools/make_fat/Cargo.toml -- \
    target/x86_64-apple-darwin/release/rustid-gui \
    target/aarch64-apple-darwin/release/rustid-gui \
    target/dist/rustid-gui-macos
```

### Build the application bundle (`Rustid.app`)

A standard macOS `.app` is just a directory laid out as:

```
Rustid.app/
  Contents/
    Info.plist
    MacOS/
      rustid-gui        # the universal "fat" binary
    Resources/
      AppIcon.icns      # optional app icon
```

The `build-mac-gui` recipe (see `justfile`) assembles this layout and zips it
as `rustid-macos.zip`, mirroring the Windows `rustid-windows.zip` artifact.

### Verification

The generated binary should report all three architectures:
```bash
file target/dist/rustid-gui-macos
# Mach-O universal binary with 2 architectures: [x86_64, arm64]
```

## The GUI implementation

The macOS GUI lives in `src/gui/macos/` and is selected in
`src/rustid_gui.rs` via the `macos_os` cfg alias (set in `build.rs`), exactly
parallel to the Win32 GUI in `src/gui/windows/`.

- `mod.rs` — AppKit `NSApplication` + `NSWindow` + scrollable `NSTextView`
  report viewer and application menus. Layout:
  - **File** — Copy Report (⌘C), Save Report… (⌘S), Export CPUID Dump…
    (x86 only), Open CPUID Dump… (⌘O, x86 only), and Refresh Hardware (⌘R,
    reloads live hardware / clears the dump).
  - **View** — Standard / Debug / Everything / CPUID Dump (x86 only) views.
  - **Options** — Verbose Output, Compact Mode, and Dark Mode checkboxes.
  - Everything appends the debug dump to the report; Copy/Save always export
    whatever is currently displayed. Dark mode is honoured through the
    `AppleInterfaceStyle` user default unless overridden via the Options menu.
    Loading a CPUID dump file points the CPUID provider at the dump, mirroring
    the CLI `-f`/Windows GUI behaviour.
- `render.rs` — reuses the shared report-generation code in `rustid` and
  renders the same line-coloring heuristics (labels, headers, values,
  highlights, dividers) as the Windows RTF generator, but emitted as an
  `NSAttributedString`.
