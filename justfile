# Lists the available actions
default:
	@just --list

# Check code validity and style
check:
    cargo check

# Build the app
build:
    cargo build

# Do an optimized, release build
build-release:
    cargo build --release

# Build for DOS (32-bit DPMI)
build-dos:
    cargo build --target i386-dos.json --release

# Build for 32-bit Linux
build-486:
    rustup target add i586-unknown-linux-gnu
    RUSTFLAGS="-C target-cpu=i486" cargo build --target i586-unknown-linux-gnu --release

# Build and run the app
run:
    cargo run

# Run all the tests
test:
    cargo test
