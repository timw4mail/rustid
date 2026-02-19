# Lists the available actions
default:
	@just --list

info:
	@echo "This is an {{arch()}} machine, running {{os()}} on {{num_cpus()}} cpus"


# Check code validity and style
check:
	cargo check

# Automatic code formatting
fmt:
	cargo fmt

# Build the app
build:
	cargo build

# Do an optimized, release build
build-release:
	cargo build --release

# Build for DOS
build-dos:
	# Fetch required tools (if they aren't already installed)
	cargo install cargo-binutils
	rustup component add llvm-tools-preview
	rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
	# Build initial binary
	cargo +nightly build -Zjson-target-spec --target i486-dos.json -Z build-std=core,alloc --release
	# Convert to proper DOS com binary
	rust-objcopy -I elf32-i386 -O binary ./target/i486-dos/release/rustid rustid.com

# Build for 32-bit Linux
build-486:
	rustup target add i586-unknown-linux-gnu
	cargo build --target i586-unknown-linux-gnu --release

# Remove build files
clean:
	cargo clean
	rm -f rustid.com
	rm -f rustid.exe

# Build and run the app
run:
	cargo run

[windows]
run-dos: build-dos
	"C:\DOSBox-X\dosbox-x.exe" .

[linux, unix]
run-dos: build-dos
	dosbox-x .

# Run all the tests
test:
	cargo test

