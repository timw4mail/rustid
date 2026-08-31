# Detect architecture and OS
ifeq ($(OS),Windows_NT)
ARCH ?= $(PROCESSOR_ARCHITECTURE)
NUM_CPUS ?= $(NUMBER_OF_PROCESSORS)
else
ARCH ?= $(shell uname -m)
OS := $(shell uname -s)
NUM_CPUS ?= $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 1)
endif

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

.PHONY: default check check-efi-64 check-efi-32 check-efi check-dos-real check-dos32a check-dos check-486 check-windows-gui check-all check-riscv check-win-arm lint fix fmt quality build build-debug build-release _cargo_cross _build-dos-tools build-dos-real _build-dos32a-tools _build-dos32a-rustid build-dos32a build-dos build-windows build-windows-gui build-windows-arm build-windows-gnu build-windows-gui-gnu build-arm64 build-ppc build-mac build-mac-arm build-486 build-efi-64 build-efi-32 build-efi build-486-musl clean clean-files run from-file run-x86-emu run-dos test-dos run-efi-64 run-efi-32 test coverage test-all test-arm test-x86

# Lists the available actions
default:
	@echo "This is an $(ARCH) machine, running $(OS) on $(NUM_CPUS) cpus/cores/threads"
	@rustup default
	@just --list 2>/dev/null || echo "Install 'just' to see available commands"

ifeq ($(OS),Windows_NT)
_cargo_cross:
	@where cargo-cross >nul 2>&1 || cargo install cargo-cross
else
_cargo_cross:
	@if ! command -v cargo-cross >/dev/null 2>&1; then cargo install cargo-cross; fi
endif

# Check code validity and style
check:
	$(BASE_CHECK)

# Compile check for 64-bit x86 EFI application
check-efi-64:
	@if ! rustup target list --installed | grep -q x86_64-unknown-uefi; then rustup target add x86_64-unknown-uefi; fi
	cargo check --target x86_64-unknown-uefi --features efi-build --bin efi_rustid

# Compile check for 32-bit x86 EFI application
check-efi-32:
	@if ! rustup target list --installed | grep -q i686-unknown-uefi; then rustup target add i686-unknown-uefi; fi
	cargo check --target i686-unknown-uefi --features efi-build --bin efi_rustid

# Compile check for both 32-bit and 64-bit EFI
check-efi: check-efi-64 check-efi-32

# Compile check for DOS (real-mode EXE)
check-dos-real: _build-dos-tools
	@RUSTFLAGS="-C link-arg=-Tbuild-config/link-exe.x" cargo +nightly check -Zjson-target-spec -Z build-std=core,alloc,panic_abort --target build-config/i486-dos.json --release --features dos-build --bin rust86

# Compile check for DOS/32A (protected-mode LE)
check-dos32a: _build-dos32a-tools
	@RUSTFLAGS="-C link-arg=-Tbuild-config/link-dos32a.x -C link-arg=--emit-relocs -C strip=none" cargo +nightly check -Zjson-target-spec -Z build-std=core,alloc,panic_abort --target build-config/i486-dos32a.json --features="dos32a-build" --bin dos_rustid --release

# Compile check for all DOS targets
check-dos: check-dos32a check-dos-real

# Compile check for Risc V
check-riscv:
	@if ! rustup target list --installed | grep -q riscv64gc-unknown-linux-gnu; then rustup target add riscv64gc-unknown-linux-gnu; fi
	cargo check --target riscv64gc-unknown-linux-gnu

# Compile check for Windows ARM
check-win-arm:
	@if ! rustup target list --installed | grep -q aarch64-pc-windows-msvc; then rustup target add aarch64-pc-windows-msvc; fi
	cargo check --target aarch64-pc-windows-msvc

# Compile check for Android ARM64
check-android:
	@if ! rustup target list --installed | grep -q aarch64-linux-android; then rustup target add aarch64-linux-android; fi
	cargo check --target aarch64-linux-android

# Compile check for 32-bit Linux 486
check-486:
	@if ! rustup component list --installed --toolchain nightly | grep -q rust-src; then rustup component add rust-src --toolchain nightly; fi
	cargo +nightly check -Zjson-target-spec -Z build-std=std,core,alloc,panic_abort --target build-config/i486-linux.json --release

# Compile check for Windows GUI
check-windows-gui:
	@if ! rustup target list --installed | grep -q x86_64-pc-windows-gnu; then rustup target add x86_64-pc-windows-gnu; fi
	cargo check --target x86_64-pc-windows-gnu --features gui --bin rustid-gui

# Compile check for all supported targets and platforms
check-all: check check-efi check-dos check-riscv check-win-arm check-android check-486 check-windows-gui

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
	@if ! rustup component list --installed --toolchain nightly | grep -q rust-src; then rustup component add rust-src --toolchain nightly; fi

# Build for DOS (EXE format)
build-dos-real: _build-dos-tools
	@RUSTFLAGS="-C link-arg=-Tbuild-config/link-exe.x" cargo +nightly build -Zjson-target-spec -Z build-std=core,alloc,panic_abort --target build-config/i486-dos.json --release --features dos-build --bin rust86
	@cargo run --manifest-path tools/make_exe/Cargo.toml --quiet -- ./target/i486-dos/release/rust86 rust86.exe
	@cargo test --test dos_binary_size_test --features dos-build

_build-dos32a-tools:
	# Fetch required tools (if they aren't already installed)
	@if ! rustup component list --installed --toolchain nightly | grep -q rust-src; then rustup component add rust-src --toolchain nightly; fi

_build-dos32a-rustid: _build-dos32a-tools
	@RUSTFLAGS="-C link-arg=-Tbuild-config/link-dos32a.x -C link-arg=--emit-relocs -C strip=none" cargo +nightly build -Zjson-target-spec -Z build-std=core,alloc,panic_abort --target build-config/i486-dos32a.json --features="dos32a-build" --bin dos_rustid --release
	@cargo run --manifest-path tools/elf2le/Cargo.toml --quiet -- ./target/i486-dos32a/release/dos_rustid rustid.le
	@if command -v dosbox-x >/dev/null 2>&1; then \
		dosbox-x -conf ./tools/dosbox-x.conf -fastlaunch -silent -exit -c "MOUNT C ." -c "C:" -c "COPY tools\dos32a\dos32a.exe ." -c "tools\dos32a\sb.exe /b /o /bnrustid.exe rustid.le" >/dev/null 2>&1 || true; \
	elif [ -f "C:/DOSBox-X/dosbox-x.exe" ]; then \
		"C:/DOSBox-X/dosbox-x.exe" -conf ./tools/dosbox-x.conf -fastlaunch -silent -exit -c "MOUNT C ." -c "C:" -c "COPY tools\dos32a\dos32a.exe ." -c "tools\dos32a\sb.exe /b /o /bnrustid.exe rustid.le" >/dev/null 2>&1 || true; \
	fi
	@if [ -f RUSTID.EXE ]; then mv RUSTID.EXE rustid.exe; fi

# Build for DOS/32A (LE format bound executable)
build-dos32a: clean-files _build-dos32a-tools _build-dos32a-rustid

# Build all dos binaries
build-dos: build-dos32a build-dos-real

ifeq ($(OS),Windows_NT)
# Build for modern windows (cli), requires visual studio to be installed
build-windows:
	@rustup target list --installed | findstr /c:"x86_64-pc-windows-msvc" >nul || rustup target add x86_64-pc-windows-msvc
	cargo build --target x86_64-pc-windows-msvc --release

# Build for modern windows (GUI), requires visual studio to be installed
build-windows-gui:
	@rustup target list --installed | findstr /c:"x86_64-pc-windows-msvc" >nul || rustup target add x86_64-pc-windows-msvc
	cargo build --target x86_64-pc-windows-msvc --features gui --bin rustid-gui --release

build-windows-arm:
	@rustup target list --installed | findstr /c:"aarch64-pc-windows-msvc" >nul || rustup target add aarch64-pc-windows-msvc
	cargo build --target aarch64-pc-windows-msvc --release
endif

# Build for modern windows (cli), can be easier than msvc build
build-windows-gnu: _cargo_cross
	@if ! rustup target list --installed | grep -q x86_64-pc-windows-gnu; then rustup target add x86_64-pc-windows-gnu; fi
	cargo cross build --target x86_64-pc-windows-gnu --release

# Build Windows GUI using MinGW/GNU target (cross-compilable from Linux)
build-windows-gui-gnu: _cargo_cross
	@if ! rustup target list --installed | grep -q x86_64-pc-windows-gnu; then rustup target add x86_64-pc-windows-gnu; fi
	cargo cross build --target x86_64-pc-windows-gnu --features gui --bin rustid-gui --release

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

# Build 64-bit x86 EFI application
build-efi-64:
	@if ! rustup target list --installed | grep -q x86_64-unknown-uefi; then rustup target add x86_64-unknown-uefi; fi
	cargo build --target x86_64-unknown-uefi --features efi-build --bin efi_rustid --release
	@mkdir -p target/efi-disk/EFI/BOOT
	@cp target/x86_64-unknown-uefi/release/efi_rustid.efi target/efi-disk/EFI/BOOT/BOOTX64.EFI

# Build 32-bit x86 EFI application
build-efi-32:
	@if ! rustup target list --installed | grep -q i686-unknown-uefi; then rustup target add i686-unknown-uefi; fi
	cargo build --target i686-unknown-uefi --features efi-build --bin efi_rustid --release
	@mkdir -p target/efi-disk/EFI/BOOT
	@cp target/i686-unknown-uefi/release/efi_rustid.efi target/efi-disk/EFI/BOOT/BOOTIA32.EFI

# Build both 32-bit and 64-bit EFI binaries
build-efi: build-efi-64 build-efi-32

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
run:
	@$(BASE_RUN) -- $(ARG)

# Run rustid, but pull cpu information from a cpuid dump
from-file:
	@$(BASE_RUN) file $(ARG)

ifeq ($(OS),Windows_NT)
# Run Windows arm64/x86_64 hybrid build - shows simulated x86 info
run-x86-emu:
	@rustup target list --installed | findstr /c:"arm64ec-pc-windows-msvc" >nul || rustup target add arm64ec-pc-windows-msvc
	cargo run --target arm64ec-pc-windows-msvc $(ARG)

# Run the dos build in DOSBox-X
run-dos: build-dos
	"C:\DOSBox-X\dosbox-x.exe" .  -fastlaunch -conf ./tools/dosbox-x.conf rustid.exe

# Run the dos build in DOSBox-x, and return the output to a file
test-dos: build-dos
	"C:\DOSBox-X\dosbox-x.exe" . -fastlaunch -console -log-con -conf ./tools/dosbox-x.conf -time-limit 2 rustid.exe
else
# Run the dos build in DOSBox-X
run-dos: build-dos
	dosbox-x . -fastlaunch rustid.exe

# Run the dos build in DOSBox-x, and return the output to a file
test-dos: build-dos
	dosbox-x . -fastlaunch -conf ./tools/dosbox-x.conf -time-limit 2 -log-con rustid.exe

# Run 64-bit EFI build in QEMU
run-efi-64: build-efi-64
	@CODE=$$(find /usr/share /usr/lib -name "*OVMF_CODE*.fd" -o -name "*ovmf_code*.fd" 2>/dev/null | grep -v 32 | grep -v -E "secboot|snakeoil|\.ms\." | head -n1); VARS=$$(find /usr/share /usr/lib -name "*OVMF_VARS*.fd" -o -name "*ovmf_vars*.fd" 2>/dev/null | grep -v 32 | grep -v -E "secboot|snakeoil|\.ms\." | head -n1); SINGLE=$$(find /usr/share /usr/lib -name "OVMF.fd" -o -name "ovmf.fd" 2>/dev/null | head -n1); if [ -n "$$CODE" ] && [ -n "$$VARS" ]; then cp "$$VARS" target/OVMF64_VARS.fd && qemu-system-x86_64 -drive if=pflash,format=raw,readonly=on,file="$$CODE" -drive if=pflash,format=raw,file=target/OVMF64_VARS.fd -drive file=fat:rw:target/efi-disk,format=raw -nographic -net none -no-reboot; elif [ -n "$$SINGLE" ]; then qemu-system-x86_64 -bios "$$SINGLE" -drive file=fat:rw:target/efi-disk,format=raw -nographic -net none -no-reboot; else echo "OVMF firmware not found"; exit 1; fi

# Run 32-bit EFI build in QEMU
run-efi-32: build-efi-32
	@CODE=$$(find /usr/share /usr/lib -name "*OVMF32_CODE*.fd" -o -name "*ovmf32_code*.fd" 2>/dev/null | grep -v -E "secboot|snakeoil|\.ms\." | head -n1); VARS=$$(find /usr/share /usr/lib -name "*OVMF32_VARS*.fd" -o -name "*ovmf32_vars*.fd" 2>/dev/null | grep -v -E "secboot|snakeoil|\.ms\." | head -n1); SINGLE=$$(find /usr/share /usr/lib -name "*OVMF32.fd" -o -name "*ovmf32.fd" 2>/dev/null | head -n1); if [ -n "$$CODE" ] && [ -n "$$VARS" ]; then cp "$$VARS" target/OVMF32_VARS.fd && qemu-system-i386 -drive if=pflash,format=raw,readonly=on,file="$$CODE" -drive if=pflash,format=raw,file=target/OVMF32_VARS.fd -drive file=fat:rw:target/efi-disk,format=raw -nographic -net none -no-reboot; elif [ -n "$$SINGLE" ]; then qemu-system-i386 -bios "$$SINGLE" -drive file=fat:rw:target/efi-disk,format=raw -nographic -net none -no-reboot; else echo "OVMF32 firmware not found. On Debian/Ubuntu, install with: sudo apt install ovmf-ia32"; exit 1; fi
endif

# Run all the (native) tests
test:
	cargo test

# Run tests and generate code coverage
coverage:
	cargo llvm-cov --open

# Run 64 and 32 bit tests (on 64bit platform)
test-all: test test-x86 test-arm

ifeq ($(OS),Windows_NT)
# Run Windows arm tests
test-arm: _cargo_cross
	@rustup target list --installed | findstr /c:"aarch64-pc-windows-msvc" >nul || rustup target add aarch64-pc-windows-msvc
	cargo cross test --target aarch64-pc-windows-gnu

# Run tests for 32-bit x86
test-x86:
	@rustup target list --installed | findstr /c:"i686-pc-windows-msvc" >nul || rustup target add i686-pc-windows-msvc
	cargo test --target i686-pc-windows-msvc
else
# Run linux aarch64 tests
test-arm: _cargo_cross
	@if ! rustup target list --installed | grep -q aarch64-unknown-linux-musl; then rustup target add aarch64-unknown-linux-musl; fi
	cargo cross test --target aarch64-unknown-linux-musl

# Run tests for 32-bit x86 (musl target - no system dependencies)
test-x86: _cargo_cross
	@if ! rustup target list --installed | grep -q i686-unknown-linux-musl; then rustup target add i686-unknown-linux-musl; fi
	cargo cross test --target i686-unknown-linux-musl
endif
