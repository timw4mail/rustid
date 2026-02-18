# Lists the available actions
default:
	@just --list

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

# Build for DOS (32-bit DPMI)
build-dos:
	rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
	cargo +nightly build -Zjson-target-spec --target i386-dos.json -Z build-std=core,alloc --release
	cp ./target/i386-dos/release/rustid rustid.exe

# Build for 32-bit Linux
build-486:
	rustup target add i586-unknown-linux-gnu
	RUSTFLAGS="-C target-cpu=i486" cargo build --target i586-unknown-linux-gnu --release

# Remove build files
clean:
	cargo clean
	rm -f rustid.com
	rm -f rustid.exe

# Build and run the app
run:
	cargo run

run-dos: build-dos
	dosbox-x .

# Run all the tests
test:
	cargo test

