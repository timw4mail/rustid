set unstable

# Lists the available actions
default:
	echo "This is an {{arch()}} machine, running {{os()}} on {{num_cpus()}} cpus"
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
	cargo clippy

# Fix linting erros
fix:
	cargo fix --all-targets

# Automatic code formatting
fmt:
	cargo fmt

# Build the app
build:
	cargo build

# Do an optimized, release build for the current platform
build-release:
	cargo build --release

# Build for DOS
build-dos:
	# Fetch required tools (if they aren't already installed)
	@if ! command -v cargo-binutils >/dev/null 2>&1; then cargo install cargo-binutils; fi
	@if ! rustup component list --installed | grep -q llvm-tools-preview; then rustup component add llvm-tools-preview; fi
	@if ! rustup component list --installed --toolchain nightly-x86_64-unknown-linux-gnu | grep -q rust-src; then rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu; fi
	# Cleanup old binary
	rm -f rustid.com
	# Build initial binary
	cargo +nightly build -Zjson-target-spec --target i486-dos.json --release
	# Convert to proper DOS com binary
	rust-objcopy -I elf32-i386 -O binary ./target/i486-dos/release/rustid rustid.com

# Build for modern windows (cli),  requires visual studio to be installed
[windows]
build-windows:
	@if ! rustup target list --installed | grep -q x86_64-pc-windows-msvc; then rustup target add x86_64-pc-windows-msvc; fi
	cargo build --target x86_64-pc-windows-msvc --release

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
	@if ! rustup target list --installed | grep -q i586-unknown-linux-gnu; then rustup target add i586-unknown-linux-gnu; fi
	cargo build --target i586-unknown-linux-gnu --release

# Remove build files
clean:
	cargo clean
	rm -f rustid.com
	rm -f rustid.exe

# Build and run the app
run arg="":
	{{base_run}} {{arg}}

# Run Windows arm64/x86_64 hybrid build - shows simulated x86 info
[windows]
run-x86-emu arg="":
	@if ! rustup target list --installed | grep -q arm64ec-pc-windows-msvc; then rustup target add arm64ec-pc-windows-msvc; fi
	cargo run --target arm64ec-pc-windows-msvc {{arg}}

# Run the dos build in DOSBox-X
[windows]
run-dos: build-dos
	"C:\DOSBox-X\dosbox-x.exe" rustid.com /fastlaunch

# Run the dos build in DOSBox-X
[linux, unix]
run-dos: build-dos
	dosbox-x rustid.com -fastlaunch

# Run all the tests
test:
	cargo test

