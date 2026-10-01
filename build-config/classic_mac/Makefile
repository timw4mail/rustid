# Makefile for building rustid for Classic Mac OS (68k and PowerPC) using Retro68

CC_68K   ?= m68k-apple-macos-gcc
CC_PPC   ?= powerpc-apple-macos-gcc
REZ      ?= Rez
MAKEPEF  ?= MakePEF

# Extract version from Cargo.toml
VERSION           ?= $(shell sed -n 's/^version = "\(.*\)"/\1/p' ../../Cargo.toml | head -n 1)
VERSION_MAJOR     ?= $(shell echo $(VERSION) | cut -d. -f1)
VERSION_MINOR     ?= $(shell echo $(VERSION) | cut -d. -f2)
VERSION_PATCH     ?= $(shell echo $(VERSION) | cut -d. -f3)
VERSION_MAJOR_BCD ?= 0x0$(VERSION_MAJOR)
VERSION_MINOR_BCD ?= 0x$(VERSION_MINOR)$(VERSION_PATCH)

# Locate the toolchain root relative to the compiler that is in PATH
RETRO68_ROOT ?= $(shell dirname $$(dirname $$(which $(CC_68K))))
REZ_INCLUDES  = -I$(RETRO68_ROOT)/RIncludes
REZ_FLAGS     = $(REZ_INCLUDES) \
                -D RUSTID_VERSION_STR="\"$(VERSION)\"" \
                -D VERSION_MAJOR_BCD=$(VERSION_MAJOR_BCD) \
                -D VERSION_MINOR_BCD=$(VERSION_MINOR_BCD)

CFLAGS   = -O2 -Wall -I../../ports/classic_mac -DRUSTID_VERSION=\"$(VERSION)\"
SRC_C    = ../../ports/classic_mac/mac_bridge.c \
           ../../ports/classic_mac/classic_mac_engine.c
R_SRC    = rustid.r
R_DEPS   = rustid.r rustid_icons.r

OUT_DIR  ?= ../../target/classic_mac

all: directories $(OUT_DIR)/rustid.bin $(OUT_DIR)/rustid_68k.bin $(OUT_DIR)/rustid_ppc.bin

directories:
	@mkdir -p $(OUT_DIR)

# Universal Fat Binary (System 6 through Mac OS 9: native 68k + native PowerPC)
$(OUT_DIR)/rustid.bin: $(OUT_DIR)/rustid_68k $(OUT_DIR)/rustid_ppc.pef $(R_DEPS)
	$(REZ) $(REZ_FLAGS) -DTARGET_FAT -o $(OUT_DIR)/rustid.bin $(R_SRC) \
		--copy $(OUT_DIR)/rustid_68k --data $(OUT_DIR)/rustid_ppc.pef \
		--cc $(OUT_DIR)/rustid.dsk -t APPL -c RsId

# 68k Raw Executable
$(OUT_DIR)/rustid_68k: $(SRC_C)
	$(CC_68K) $(CFLAGS) -o $(OUT_DIR)/rustid_68k $(SRC_C)

# 68k Standalone Binary (System 6 – 8.1)
$(OUT_DIR)/rustid_68k.bin: $(OUT_DIR)/rustid_68k $(R_DEPS)
	$(REZ) $(REZ_FLAGS) -o $(OUT_DIR)/rustid_68k.bin $(R_SRC) \
		--copy $(OUT_DIR)/rustid_68k --cc $(OUT_DIR)/rustid_68k.dsk -t APPL -c RsId

# PowerPC PEF Executable
$(OUT_DIR)/rustid_ppc.pef: $(SRC_C)
	$(CC_PPC) $(CFLAGS) -o $(OUT_DIR)/rustid_ppc.xcoff $(SRC_C)
	$(MAKEPEF) $(OUT_DIR)/rustid_ppc.xcoff -o $(OUT_DIR)/rustid_ppc.pef

# PowerPC Standalone Binary (System 7.1.2 – Mac OS 9.2.2)
$(OUT_DIR)/rustid_ppc.bin: $(OUT_DIR)/rustid_ppc.pef $(R_DEPS)
	$(REZ) $(REZ_FLAGS) -DTARGET_PPC -o $(OUT_DIR)/rustid_ppc.bin $(R_SRC) \
		--data $(OUT_DIR)/rustid_ppc.pef --cc $(OUT_DIR)/rustid_ppc.dsk -t APPL -c RsId

clean:
	rm -rf $(OUT_DIR)

.PHONY: all directories clean
