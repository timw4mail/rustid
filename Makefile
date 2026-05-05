# Detect architecture and OS
ARCH := $(shell uname -m)
OS := $(shell uname -s)
NUM_CPUS := $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 1)

# Set base commands based on architecture
ifeq ($(ARCH),powerpc)
BASE_RUN := cargo +nightly run -Z build-std
BASE_CHECK := cargo +nightly check -Z build-std --all-targets
else ifeq ($(ARCH),powerpc64)
BASE_RUN := cargo +nightly run -Z build-std
BASE_CHECK := cargo +nightly check -Z build-std --all-targets
else
BASE_RUN := cargo run
BASE_CHECK := cargo check --all-targets
endif

.PHONY: default check lint fix fmt quality build build-debug build-release clean run from-file run-debug test coverage test-all build-dos _build-dos-tools _build-dos-debug _build-dos-dump

# Lists the available actions
default:
	@echo "This is an $(ARCH) machine, running $(OS) on $(NUM_CPUS) cpus/cores/threads"
	@rustup default
	@just --list 2>/dev/null || echo "Install 'just' to see available commands"

# Check code validity and style
check:
	$(BASE_CHECK)

# More in-depth code style checking
lint:
	cargo clippy --all-targets --all-features

# Fix linting errors
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

# Fetch cross compilation tool
_cargo_cross:
	@if ! command -v cargo-cross >/dev/null 2>&1; then cargo install cargo-cross; fi

# DOS build tools
_build-dos-tools:
	@if ! rustup component list --installed --toolchain nightly-x86_64-unknown-linux-gnu | grep -q rust-src; then rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu; fi

_build-dos-debug: _build-dos-tools
	@RUSTFLAGS="-C link-arg=-Tlink-exe.x" cargo +nightly build -Zjson-target-spec -Z build-std=core,alloc,panic_abort --target i486-dos.json --features="debug dos-build" --bin debug --release
	@cargo run --manifest-path tools/make_exe/Cargo.toml --quiet -- ./target/i486-dos/release/debug debug.exe

_build-dos-dump: _build-dos-tools
	@RUSTFLAGS="-C link-arg=-Tlink-exe.x" cargo +nightly build -Zjson-target-spec -Z build-std=core,alloc,panic_abort --target i486-dos.json --features dos-build --bin dump --release
	@cargo run --manifest-path tools/make_exe/Cargo.toml --quiet -- ./target/i486-dos/release/dump dump.exe

# Build for DOS (EXE format)
build-dos: _build-dos-tools _build-dos-debug _build-dos-dump
	@RUSTFLAGS="-C link-arg=-Tlink-exe.x" cargo +nightly build -Zjson-target-spec -Z build-std=core,alloc,panic_abort --target i486-dos.json --release --features dos-build
	@cargo run --manifest-path tools/make_exe/Cargo.toml --quiet -- ./target/i486-dos/release/rustid rustid.exe
	@cargo test --test dos_binary_size_test --features dos-build

# Build for modern windows (cli), requires visual studio to be installed
ifeq ($(OS),Windows_NT)
build-windows:
	@if ! rustup target list --installed | grep -q x86_64-pc-windows-msvc; then rustup target add x86_64-pc-windows-msvc; fi
	cargo build --target x86_64-pc-windows-msvc --release

build-windows-arm:
	@if ! rustup target list --installed | grep -q aarch64-pc-windows-msvc; then rustup target add aarch64-pc-windows-msvc; fi
	cargo build --target aarch64-pc-windows-msvc --release

# Run Windows arm64/x86_64 hybrid build - shows simulated x86 info
run-x86-emu:
	@if ! rustup target list --installed | grep -q arm64ec-pc-windows-msvc; then rustup target add arm64ec-pc-windows-msvc; fi
	cargo run --target arm64ec-pc-windows-msvc $(ARG)

# Run the dos build in DOSBox-X
run-dos: build-dos
	"C:\DOSBox-X\dosbox-x.exe" .  /fastlaunch rustid.exe

# Run windwos arm tests
test-arm: _cargo_cross
	@if ! rustup target list --installed | grep -q aarch64-pc-windows-msvc; then rustup target add aarch64-pc-windows-msvc; fi
	cargo cross test --target aarch64-pc-windows-gnu

# Run tests for 32-bit x86
test-x86:
	@if ! rustup target list --installed | grep -q i686-pc-windows-msvc; then rustup target add i686-pc-windows-msvc; fi
	cargo test --target i686-pc-windows-msvc
endif

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
	@rm -f *.com
	@rm -f *.exe
	@rm -f *.bin

# Build and run the app
run:
	@$(BASE_RUN) -- $(ARG)

# Run rustid, but pull cpu information from a cpuid dump
from-file:
	@$(BASE_RUN) file $(ARG)

# Build and run the debug app
run-debug:
	@$(BASE_RUN) --features debug --bin debug $(ARG)

# Run the dos build in DOSBox-X (Linux/Unix)
ifeq ($(OS),Linux)
run-dos: build-dos
	dosbox-x . -fastlaunch rustid.exe
endif
ifeq ($(OS),Darwin)
run-dos: build-dos
	dosbox-x . -fastlaunch rustid.exe
endif

# Run all the (native) tests
test:
	cargo test

# Run tests and generate code coverage
coverage:
	cargo llvm-cov --open

# Run 64 and 32 bit tests (on 64bit platform)
test-all: test test-x86 test-arm

# Run linux aarch64 tests
ifeq ($(OS),Linux)
test-arm: _cargo_cross
	@if ! rustup target list --installed | grep -q aarch64-unknown-linux-musl; then rustup target add aarch64-unknown-linux-musl; fi
	cargo cross test --target aarch64-unknown-linux-musl
endif
ifeq ($(OS),Darwin)
test-arm: _cargo_cross
	@if ! rustup target list --installed | grep -q aarch64-unknown-linux-musl; then rustup target add aarch64-unknown-linux-musl; fi
	cargo cross test --target aarch64-unknown-linux-musl
endif

# Run tests for 32-bit x86 (musl target - no system dependencies)
ifeq ($(OS),Linux)
test-x86: _cargo_cross
	@if ! rustup target list --installed | grep -q i686-unknown-linux-musl; then rustup target add i686-unknown-linux-musl; fi
	cargo cross test --target i686-unknown-linux-musl
endif
ifeq ($(OS),Darwin)
test-x86: _cargo_cross
	@if ! rustup target list --installed | grep -q i686-unknown-linux-musl; then rustup target add i686-unknown-linux-musl; fi
	cargo cross test --target i686-unknown-linux-musl
endif
