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

.PHONY: default check check-riscv lint fix fmt quality build build-debug build-release _cargo_cross _build-dos-tools _build-dos-debug build-dos-real _build-dos32a-tools _build-dos32a-rustid build-dos32a build-dos build-windows build-windows-arm build-windows-gnu build-arm64 build-ppc build-mac build-mac-arm build-486 build-486-musl clean clean-files run from-file run-x86-emu run-dos test-dos test coverage test-all test-arm test-x86

# Lists the available actions
default:
	@echo "This is an $(ARCH) machine, running $(OS) on $(NUM_CPUS) cpus/cores/threads"
	@rustup default
	@just --list 2>/dev/null || echo "Install 'just' to see available commands"

# Fetch cross compilation tool
_cargo_cross:
	@if ! command -v cargo-cross >/dev/null 2>&1; then cargo install cargo-cross; fi

# Check code validity and style
check:
	$(BASE_CHECK)

# Compile check for Risc V
check-riscv:
	cargo check --target riscv64gc-unknown-linux-gnu

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

# DOS build tools
_build-dos-tools:
	@if ! rustup component list --installed --toolchain nightly-x86_64-unknown-linux-gnu | grep -q rust-src; then rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu; fi

_build-dos-debug: _build-dos-tools
	@RUSTFLAGS="-C link-arg=-Tbuild-config/link-exe.x" cargo +nightly build -Zjson-target-spec -Z build-std=core,alloc,panic_abort --target build-config/i486-dos.json --features="debug dos-build" --bin debug86 --release
	@cargo run --manifest-path tools/make_exe/Cargo.toml --quiet -- ./target/i486-dos/release/debug86 debug86.exe

# Build for DOS (EXE format)
build-dos-real: _build-dos-tools _build-dos-debug
	@RUSTFLAGS="-C link-arg=-Tbuild-config/link-exe.x" cargo +nightly build -Zjson-target-spec -Z build-std=core,alloc,panic_abort --target build-config/i486-dos.json --release --features dos-build --bin rust86
	@cargo run --manifest-path tools/make_exe/Cargo.toml --quiet -- ./target/i486-dos/release/rust86 rust86.exe
	@cargo test --test dos_binary_size_test --features dos-build

# DOS/32A build tools
_build-dos32a-tools:
	@if ! rustup component list --installed --toolchain nightly-x86_64-unknown-linux-gnu | grep -q rust-src; then rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu; fi

_build-dos32a-rustid: _build-dos32a-tools
	@RUSTFLAGS="-C link-arg=-Tbuild-config/link-dos32a.x -C link-arg=--emit-relocs -C strip=none" cargo +nightly build -Zjson-target-spec -Z build-std=core,alloc,panic_abort --target build-config/i486-dos32a.json --features="dos32a-build" --bin dos_rustid --release
	@cargo run --manifest-path tools/elf2le/Cargo.toml --quiet -- ./target/i486-dos32a/release/dos_rustid rustid.le
	@if command -v dosbox-x >/dev/null 2>&1; then dosbox-x -conf ./tools/dosbox-x.conf -fastlaunch -silent -exit -c "MOUNT C ." -c "C:" -c "COPY tools\dos32a\dos32a.exe ." -c "tools\dos32a\sb.exe /b /o /bnrustid.exe rustid.le" >/dev/null 2>&1 || true; fi
	@if [ -f RUSTID.EXE ]; then cp RUSTID.EXE rustid.exe; rm RUSTID.EXE; fi

# Build for DOS/32A (LE format bound executable)
build-dos32a: clean-files _build-dos32a-tools _build-dos32a-rustid

# Build all DOS binaries
build-dos: build-dos32a build-dos-real

# Build for modern windows (cli), requires visual studio to be installed
ifeq ($(OS),Windows_NT)
_cargo_cross:
	@where cargo-cross >nul 2>&1 || cargo install cargo-cross

build-windows:
	@rustup target list --installed | findstr /c:"x86_64-pc-windows-msvc" >nul || rustup target add x86_64-pc-windows-msvc
	cargo build --target x86_64-pc-windows-msvc --release

build-windows-arm:
	@rustup target list --installed | findstr /c:"aarch64-pc-windows-msvc" >nul || rustup target add aarch64-pc-windows-msvc
	cargo build --target aarch64-pc-windows-msvc --release

# Run Windows arm64/x86_64 hybrid build - shows simulated x86 info
run-x86-emu:
	@rustup target list --installed | findstr /c:"arm64ec-pc-windows-msvc" >nul || rustup target add arm64ec-pc-windows-msvc
	cargo run --target arm64ec-pc-windows-msvc $(ARG)

# Run the dos build in DOSBox-X
run-dos: build-dos
	"C:\DOSBox-X\dosbox-x.exe" .  -fastlaunch -conf ./tools/dosbox-x.conf rustid.exe

# Run the dos build in DOSBox-X, and return the output to a file
test-dos: build-dos
	"C:\DOSBox-X\dosbox-x.exe" . -fastlaunch -console -log-con -conf ./tools/dosbox-x.conf -time-limit 2 rustid.exe

# Run Windows arm tests
test-arm: _cargo_cross
	@rustup target list --installed | findstr /c:"aarch64-pc-windows-msvc" >nul || rustup target add aarch64-pc-windows-msvc
	cargo cross test --target aarch64-pc-windows-gnu

# Run tests for 32-bit x86
test-x86:
	@rustup target list --installed | findstr /c:"i686-pc-windows-msvc" >nul || rustup target add i686-pc-windows-msvc
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
	cargo +nightly build -Zjson-target-spec -Z build-std=std,core,alloc,panic_abort --target build-config/i486-linux.json --release

# Build for 32-bit Linux musl via cross (works on 486-class cpus)
build-486-musl: _cargo_cross
	@if ! rustup component list --installed --toolchain nightly | grep -q rust-src; then rustup component add rust-src --toolchain nightly; fi
	cargo cross +nightly build -t i586-unknown-linux-musl --rustflag '-C' --rustflag 'target-cpu=i486' --rustflag '-C' --rustflag 'link-arg=-Wl,-Bstatic' --rustflag '-C' --rustflag 'link-arg=-lgcc' --rustflag '-C' --rustflag 'link-arg=-latomic' --build-std --panic-immediate-abort --release

# Remove various artifacts in root
clean-files:
	@rm -f *.com
	@rm -f *.exe
	@rm -f *.EXE
	@rm -f *.le
	@rm -f *.lx
	@rm -f *.LX
	@rm -f *.bin
	@rm -f *.log

# Remove build files
clean: clean-files
	@cargo clean

# Build and run the app
run:
	@$(BASE_RUN) -- $(ARG)

# Run rustid, but pull cpu information from a cpuid dump
from-file:
	@$(BASE_RUN) file $(ARG)

# Run the dos build in DOSBox-X (Linux/Unix)
ifeq ($(OS),Linux)
run-dos: build-dos
	dosbox-x . -fastlaunch rustid.exe

test-dos: build-dos
	dosbox-x . -fastlaunch -conf ./tools/dosbox-x.conf -time-limit 2 -log-con rustid.exe
endif
ifeq ($(OS),Darwin)
run-dos: build-dos
	dosbox-x . -fastlaunch rustid.exe

test-dos: build-dos
	dosbox-x . -fastlaunch -conf ./tools/dosbox-x.conf -time-limit 2 -log-con rustid.exe
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
