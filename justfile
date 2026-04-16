set unstable

# Lists the available actions
default:
	@echo "This is an {{arch()}} machine, running {{os()}} on {{num_cpus()}} cpus/cores/threads"
	@rustup default
	@just --list

base_run := if arch() == "powerpc" || "powerpc64" { "cargo +nightly run -Z build-std" } else { "cargo run" }
base_check := if arch() == "powerpc" || "powerpc64" { "cargo +nightly check -Z build-std --all-targets" } else { "cargo check --all-targets" }

_cargo_cross:
	@if ! command -v cargo-cross >/dev/null 2>&1; then cargo install cargo-cross; fi

# Check code validity and style
check:
	{{ base_check }}

# More in-depth code style checking
lint:
	cargo clippy --all-targets --all-features

# Fix linting erros
fix:
	cargo fix --all-targets --all-features

# Automatic code formatting
fmt:
	cargo fmt

# Run all the code quality stuff
quality: fmt check lint

# Build the app
build:
	cargo build

# Build debug app
build-debug:
	cargo build --features debug --bin debug

# Do an optimized, release build for the current platform
build-release:
	cargo build --release

_build-dos:
	# Fetch required tools (if they aren't already installed)
	@if ! command -v cargo-binutils >/dev/null 2>&1; then cargo install cargo-binutils; fi
	@if ! rustup component list --installed | grep -q llvm-tools-preview; then rustup component add llvm-tools-preview; fi
	@if ! rustup component list --installed --toolchain nightly-x86_64-unknown-linux-gnu | grep -q rust-src; then rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu; fi
	# Cleanup old binaries
	@rm -f *.com

_build-dos-debug:
	@cargo +nightly build -Zjson-target-spec --target i486-dos.json --features="debug dos-build" --bin debug --release
	@rust-objcopy -I elf32-i386 -O binary ./target/i486-dos/release/debug debug.com

_build-dos-dump:
	@cargo +nightly build -Zjson-target-spec --target i486-dos.json --features dos-build --bin dump --release
	@rust-objcopy -I elf32-i386 -O binary ./target/i486-dos/release/dump dump.com

# Build for DOS - subcommands are split into separate binaries
build-dos: _build-dos _build-dos-debug _build-dos-dump
	# Build initial binary and convert to COM
	@cargo +nightly build -Zjson-target-spec --target i486-dos.json --release --features dos-build
	@rust-objcopy -I elf32-i386 -O binary ./target/i486-dos/release/rustid rustid.com
	# Verify that the binary size is below the 64K limit
	@cargo test --test dos_binary_size_test --features dos-build

# Build for modern windows (cli),  requires visual studio to be installed
[windows]
build-windows:
	@if ! rustup target list --installed | grep -q x86_64-pc-windows-msvc; then rustup target add x86_64-pc-windows-msvc; fi
	cargo build --target x86_64-pc-windows-msvc --release

build-windows-arm:
	@if ! rustup target list --installed | grep -q aarch64-pc-windows-msvc; then rustup target add aarch64-pc-windows-msvc; fi
	cargo build --target aarch64-pc-windows-msvc --release

# Build for modern windows (cli), can be easier than msvc build
build-windows-gnu: _cargo_cross
	@if ! rustup target list --installed | grep -q x86_64-pc-windows-gnu; then rustup target add x86_64-pc-windows-gnu; fi
	cargo cross build --target x86_64-pc-windows-gnu --release

# Build for linux arm64
build-arm64: _cargo_cross
	@if ! rustup target list --installed | grep -q aarch64-unknown-linux-gnu; then rustup target add aarch64-unknown-linux-gnu; fi
	cargo cross build --target aarch64-unknown-linux-gnu

# Build for linux powerpc
build-ppc: _cargo_cross
	@if ! rustup target list --installed | grep -q powerpc-unknown-linux-gnu; then rustup target add powerpc-unknown-linux-gnu; fi
	cargo cross +nightly build --target powerpc-unknown-linux-gnu -Z build-std

# Build for x86 macs
build-mac: _cargo_cross
	@if ! rustup target list --installed | grep -q x86_64-apple-darwin; then rustup target add x86_64-apple-darwin; fi
	cargo cross build --target x86_64-apple-darwin --release

# Build for arm Macs
build-mac-arm: _cargo_cross
	@if ! rustup target list --installed | grep -q aarch64-apple-darwin; then rustup target add aarch64-apple-darwin; fi
	cargo cross build --target aarch64-apple-darwin --release

# Build for 32-bit Linux (should work on 486-class cpus)
build-486:
	@if ! rustup component list --installed --toolchain nightly | grep -q rust-src; then rustup component add rust-src --toolchain nightly; fi
	cargo +nightly build -Zjson-target-spec -Z build-std=std,core,alloc,panic_abort --target i486-linux.json --release

# Remove build files
clean:
	@cargo clean
	@rm -f drustid.com
	@rm -f debug.com
	@rm -f dump.com
	@rm -f rustid.com

# Build and run the app
run arg="":
	@{{base_run}} -- {{arg}}

# Run rustid, but pull cpu information from a cpuid dump
from-file arg="":
	@{{base_run}} file {{arg}}

# Build and run the debug app
run-debug arg="":
	@{{base_run}} --features debug --bin debug {{arg}}

# Run Windows arm64/x86_64 hybrid build - shows simulated x86 info
[windows]
run-x86-emu arg="":
	@if ! rustup target list --installed | grep -q arm64ec-pc-windows-msvc; then rustup target add arm64ec-pc-windows-msvc; fi
	cargo run --target arm64ec-pc-windows-msvc {{arg}}

# Run the dos build in DOSBox-X
[windows]
run-dos: build-dos
	"C:\DOSBox-X\dosbox-x.exe" .  /fastlaunch rustid.com

# Run the dos debug build in DOSBox-X
[linux, unix]
run-dos: build-dos
	dosbox-x . -fastlaunch rustid.com

# Run all the (native) tests
test:
	cargo test -- --test-threads=1

# Run tests and generate code coverage
coverage:
	cargo llvm-cov --open -- --test-threads=1

# Run 64 and 32 bit tests (on 64bit platform)
test-all: test test-x86 test-arm

# Run linux aarch64 tests
[linux, unix]
test-arm: _cargo_cross
	@if ! rustup target list --installed | grep -q aarch64-unknown-linux-musl; then rustup target add aarch64-unknown-linux-musl; fi
	cargo cross test --target aarch64-unknown-linux-musl -- --test-threads=1

# Run windwos arm tests
[windows]
test-arm: _cargo_cross
	@if ! rustup target list --installed | grep -q aarch64-pc-windows-msvc; then rustup target add aarch64-pc-windows-msvc; fi
	cargo cross test --target aarch64-pc-windows-gnu -- --test-threads=1


# Run tests for 32-bit x86 (musl target - no system dependencies)
[linux, unix]
test-x86: _cargo_cross
	@if ! rustup target list --installed | grep -q i686-unknown-linux-musl; then rustup target add i686-unknown-linux-musl; fi
	cargo cross test --target i686-unknown-linux-musl -- --test-threads=1

# Run tests for 32-bit x86
[windows]
test-x86:
	@if ! rustup target list --installed | grep -q i686-pc-windows-msvc; then rustup target add i686-pc-windows-msvc; fi
	cargo test --target i686-pc-windows-msvc -- --test-threads=1

