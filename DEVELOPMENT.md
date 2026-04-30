# Development Setup

## Prerequisites (development)
- Rust (`rustup` and `cargo` need to be installed)
- `just` - Required to run build scripts. Can be installed with `cargo install just`.
- DOSBox-X (optional) - Helpful for development and testing of the DOS version

## Building

**Standard Build:**
```bash
just build-release
```

**Build for DOS:**
```bash
just build-dos
```
This produces a `rustid.exe` binary compatible with DOS environments (like DOSBox-X).

**Cross-Compilation:**
For other architectures, see the `justfile` for available targets (`just build-arm64`, `just build-ppc`, etc). Cross-compilation should be considered experimental.

## Usage

Simply run the compiled binary to see your CPU details:

```bash
just run
```
or
```bash
cargo run
```