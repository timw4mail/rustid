# Lists the available actions
default:
	@echo "This is an {{arch()}} machine, running {{os()}} on {{num_cpus()}} cpus/cores/threads"
	@rustup default
	@just --list

base_run := if arch() == "powerpc" { "cargo +nightly run -Z build-std" } else { "cargo run" }
base_check := if arch() == "powerpc" { "cargo +nightly check -Z build-std --all-targets" } else { "cargo check --all-targets" }

[linux, unix]
_cargo_cross:
	@if ! command -v cargo-cross >/dev/null 2>&1; then cargo install cargo-cross; fi

[windows]
_cargo_cross:
	@where cargo-cross >nul 2>&1 || cargo install cargo-cross

# Check code validity and style
check:
	{{ base_check }}

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

_build-dos-tools:
	# Fetch required tools (if they aren't already installed)
	@if ! rustup component list --installed --toolchain nightly-x86_64-unknown-linux-gnu | grep -q rust-src; then rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu; fi

_build-dos-debug: _build-dos-tools
	@RUSTFLAGS="-C link-arg=-Tbuild-config/link-exe.x" cargo +nightly build -Zjson-target-spec -Z build-std=core,alloc,panic_abort --target build-config/i486-dos.json --features="debug dos-build" --bin debug86 --release
	@cargo run --manifest-path tools/make_exe/Cargo.toml --quiet -- ./target/i486-dos/release/debug86 debug86.exe

# Build for DOS (EXE format)
build-dos-real: _build-dos-tools _build-dos-debug
	@RUSTFLAGS="-C link-arg=-Tbuild-config/link-exe.x" cargo +nightly build -Zjson-target-spec -Z build-std=core,alloc,panic_abort --target build-config/i486-dos.json --release --features dos-build --bin rust86
	@cargo run --manifest-path tools/make_exe/Cargo.toml --quiet -- ./target/i486-dos/release/rust86 rust86.exe
	@cargo test --test dos_binary_size_test --features dos-build

_build-dos32a-tools:
	# Fetch required tools (if they aren't already installed)
	@if ! rustup component list --installed --toolchain nightly-x86_64-unknown-linux-gnu | grep -q rust-src; then rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu; fi

_build-dos32a-rustid: _build-dos32a-tools
	@RUSTFLAGS="-C link-arg=-Tbuild-config/link-dos32a.x -C link-arg=--emit-relocs -C strip=none" cargo +nightly build -Zjson-target-spec -Z build-std=core,alloc,panic_abort --target build-config/i486-dos32a.json --features="dos32a-build" --bin dos_rustid --release
	@cargo run --manifest-path tools/elf2le/Cargo.toml --quiet -- ./target/i486-dos32a/release/dos_rustid rustid.le
	@if command -v dosbox-x >/dev/null 2>&1; then dosbox-x -conf ./tools/dosbox-x.conf -fastlaunch -silent -exit -c "MOUNT C ." -c "C:" -c "COPY tools\dos32a\dos32a.exe ." -c "tools\dos32a\sb.exe /b /o /bnrustid.exe rustid.le" >/dev/null 2>&1 || true; fi
	@if [ -f RUSTID.EXE ]; then cp RUSTID.EXE rustid.exe; rm RUSTID.EXE; fi

# Build for DOS/32A (LE format bound executable)
build-dos32a: clean-files _build-dos32a-tools _build-dos32a-rustid

# Build all dos binaries
build-dos: build-dos32a build-dos-real

# Build for modern windows (cli), requires visual studio to be installed
[windows]
build-windows:
	@rustup target list --installed | findstr /c:"x86_64-pc-windows-msvc" >nul || rustup target add x86_64-pc-windows-msvc
	cargo build --target x86_64-pc-windows-msvc --release

[windows]
build-windows-arm:
	@rustup target list --installed | findstr /c:"aarch64-pc-windows-msvc" >nul || rustup target add aarch64-pc-windows-msvc
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
	cargo +nightly build -Zjson-target-spec -Z build-std=std,core,alloc,panic_abort --target build-config/i486-linux.json --release

# Build for 32-bit Linux musl via cross (works on 486-class cpus)
build-486-musl: _cargo_cross
	@if ! rustup component list --installed --toolchain nightly | grep -q rust-src; then rustup component add rust-src --toolchain nightly; fi
	cargo cross +nightly build -t i586-unknown-linux-musl --rustflag '-C' --rustflag 'target-cpu=i486' --rustflag '-C' --rustflag 'link-arg=-Wl,-Bstatic' --rustflag '-C' --rustflag 'link-arg=-lgcc' --rustflag '-C' --rustflag 'link-arg=-latomic' --build-std --panic-immediate-abort --release

# Remove build files
clean: clean-files
	@cargo clean

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

# Build and run the app
run arg="":
	@{{base_run}} -- {{arg}}

# Run rustid, but pull cpu information from a cpuid dump
from-file arg="":
	@{{base_run}} file {{arg}}

# Run Windows arm64/x86_64 hybrid build - shows simulated x86 info
[windows]
run-x86-emu arg="":
	@rustup target list --installed | findstr /c:"arm64ec-pc-windows-msvc" >nul || rustup target add arm64ec-pc-windows-msvc
	cargo run --target arm64ec-pc-windows-msvc {{arg}}

# Run the dos build in DOSBox-X
[windows]
run-dos: build-dos
	"C:\DOSBox-X\dosbox-x.exe" .  -fastlaunch -conf ./tools/dosbox-x.conf rustid.exe

# Run the dos build in DOSBox-X
[linux, unix]
run-dos: build-dos
	dosbox-x . -fastlaunch rustid.exe

# Run the dos build in DOSBox-x, and return the output to a file
[linux, unix]
test-dos: build-dos
	dosbox-x . -fastlaunch -conf ./tools/dosbox-x.conf -time-limit 2 -log-con rustid.exe

# Run the dos build in DOSBox-x, and return the output to a file
[windows]
test-dos: build-dos
	"C:\DOSBox-X\dosbox-x.exe" . -fastlaunch -console -log-con -conf ./tools/dosbox-x.conf -time-limit 2 rustid.exe

# Run all the (native) tests
test:
	cargo test

# Run tests and generate code coverage
coverage:
	cargo llvm-cov --open

# Run 64 and 32 bit tests (on 64bit platform)
test-all: test test-x86 test-arm

# Run linux aarch64 tests
[linux, unix]
test-arm: _cargo_cross
	@if ! rustup target list --installed | grep -q aarch64-unknown-linux-musl; then rustup target add aarch64-unknown-linux-musl; fi
	cargo cross test --target aarch64-unknown-linux-musl

# Run Windows arm tests
[windows]
test-arm: _cargo_cross
	@rustup target list --installed | findstr /c:"aarch64-pc-windows-msvc" >nul || rustup target add aarch64-pc-windows-msvc
	cargo cross test --target aarch64-pc-windows-gnu

# Run tests for 32-bit x86 (musl target - no system dependencies)
[linux, unix]
test-x86: _cargo_cross
	@if ! rustup target list --installed | grep -q i686-unknown-linux-musl; then rustup target add i686-unknown-linux-musl; fi
	cargo cross test --target i686-unknown-linux-musl

# Run tests for 32-bit x86
[windows]
test-x86:
	@rustup target list --installed | findstr /c:"i686-pc-windows-msvc" >nul || rustup target add i686-pc-windows-msvc
	cargo test --target i686-pc-windows-msvc
