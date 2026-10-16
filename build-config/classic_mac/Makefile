# Makefile for building rustid for Classic Mac OS (68k and PowerPC) and Mac OS X using Retro68

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
PORT_DIR      = ../../ports/classic_mac
REZ_INCLUDES  = -I$(RETRO68_ROOT)/RIncludes -I$(PORT_DIR)
REZ_FLAGS     = $(REZ_INCLUDES) \
                -D RUSTID_VERSION_STR="\"$(VERSION)\"" \
                -D VERSION_MAJOR_BCD=$(VERSION_MAJOR_BCD) \
                -D VERSION_MINOR_BCD=$(VERSION_MINOR_BCD)

CFLAGS        = -O2 -Wall -I$(PORT_DIR) -DRUSTID_VERSION=\"$(VERSION)\"
CFLAGS_CARBON = $(CFLAGS) -carbon -DTARGET_API_MAC_CARBON=1
SRC_C         = $(PORT_DIR)/common/gestalt.c \
                $(PORT_DIR)/common/display.c \
                $(PORT_DIR)/common/cpu.c \
                $(PORT_DIR)/models/models.c \
                $(PORT_DIR)/m68k/cpu.c \
                $(PORT_DIR)/ppc/cpu.c \
                $(PORT_DIR)/gui/window.c \
                $(PORT_DIR)/gui/menu.c \
                $(PORT_DIR)/gui/dialogs.c \
                $(PORT_DIR)/main.c
R_SRC         = $(PORT_DIR)/rustid.r
R_DEPS        = $(PORT_DIR)/rustid.r $(PORT_DIR)/rustid_icons.r

OUT_DIR       ?= ../../target/classic_mac
DIST_DIR      ?= ../../target/dist/classic_mac

all: directories $(DIST_DIR)/rustid.bin $(DIST_DIR)/Rustid.app $(DIST_DIR)/Rustid-osx-ppc.zip

classic: directories $(DIST_DIR)/rustid.bin
osx: directories $(DIST_DIR)/Rustid.app $(DIST_DIR)/Rustid-osx-ppc.zip

directories:
	@mkdir -p $(OUT_DIR) $(DIST_DIR)

# Universal Fat Binary (System 6 through Mac OS 9: native 68k + native PowerPC)
$(DIST_DIR)/rustid.bin: $(OUT_DIR)/rustid_68k $(OUT_DIR)/rustid_ppc.pef $(R_DEPS)
	$(REZ) $(REZ_FLAGS) -DTARGET_FAT -o $(DIST_DIR)/rustid.bin $(R_SRC) \
		--copy $(OUT_DIR)/rustid_68k --data $(OUT_DIR)/rustid_ppc.pef \
		--cc $(DIST_DIR)/rustid.dsk -t APPL -c RsId

# 68k Raw Executable
$(OUT_DIR)/rustid_68k: $(SRC_C)
	$(CC_68K) $(CFLAGS) -o $(OUT_DIR)/rustid_68k $(SRC_C)

# PowerPC Classic PEF Executable (InterfaceLib: System 7.1.2 – Mac OS 9.2.2)
$(OUT_DIR)/rustid_ppc.pef: $(SRC_C)
	$(CC_PPC) $(CFLAGS) -o $(OUT_DIR)/rustid_ppc.xcoff $(SRC_C)
	$(MAKEPEF) $(OUT_DIR)/rustid_ppc.xcoff -o $(OUT_DIR)/rustid_ppc.pef

# PowerPC Carbon PEF Executable (Mac OS X Aqua & CarbonLib)
$(OUT_DIR)/rustid_carbon.pef: $(SRC_C)
	$(CC_PPC) $(CFLAGS_CARBON) -o $(OUT_DIR)/rustid_carbon.xcoff $(SRC_C)
	$(MAKEPEF) $(OUT_DIR)/rustid_carbon.xcoff -o $(OUT_DIR)/rustid_carbon.pef

# Application Icon (.icns directly from project assets)
$(OUT_DIR)/rustid.icns: ../../assets/rustid.icns
	cp ../../assets/rustid.icns $(OUT_DIR)/rustid.icns

# Mac OS X Application Bundle (PowerPC G3/G4/G5, OS X 10.0–10.5)
$(DIST_DIR)/Rustid.app: $(OUT_DIR)/rustid_carbon.pef $(OUT_DIR)/rustid.icns Info.plist $(R_DEPS)
	@rm -rf $(DIST_DIR)/Rustid.app $(OUT_DIR)/.rsrc_tmp
	@mkdir -p $(DIST_DIR)/Rustid.app/Contents/MacOS \
		$(DIST_DIR)/Rustid.app/Contents/Resources \
		$(OUT_DIR)/.rsrc_tmp
	# 1. Native Mac OS X Aqua executable
	cp $(OUT_DIR)/rustid_carbon.pef $(DIST_DIR)/Rustid.app/Contents/MacOS/rustid
	chmod +x $(DIST_DIR)/Rustid.app/Contents/MacOS/rustid
	# 2. Extract raw data-fork resource map and copy universal icon for OS X
	$(REZ) $(REZ_FLAGS) -DTARGET_CARBON -o $(OUT_DIR)/.rsrc_tmp/dummy.bin $(R_SRC) \
		--data $(OUT_DIR)/rustid_carbon.pef --cc $(OUT_DIR)/.rsrc_tmp/rustid -t APPL -c RsId
	cp $(OUT_DIR)/.rsrc_tmp/.rsrc/rustid $(DIST_DIR)/Rustid.app/Contents/Resources/rustid.rsrc
	cp $(OUT_DIR)/rustid.icns $(DIST_DIR)/Rustid.app/Contents/Resources/rustid.icns
	@rm -rf $(OUT_DIR)/.rsrc_tmp
	# 3. Bundle manifests
	cp Info.plist $(DIST_DIR)/Rustid.app/Contents/Info.plist
	echo -n "APPLRsId" > $(DIST_DIR)/Rustid.app/Contents/PkgInfo

# Mac OS X Compressed Distribution Archive
$(DIST_DIR)/Rustid-osx-ppc.zip: $(DIST_DIR)/Rustid.app
	(cd $(DIST_DIR) && rm -f Rustid-osx-ppc.zip && zip -qr Rustid-osx-ppc.zip Rustid.app)

clean:
	rm -rf $(OUT_DIR) $(DIST_DIR)

TEST_SRCS = $(filter-out ../../ports/classic_mac/main.c,$(SRC_C)) ../../ports/classic_mac/test_classic_mac.c

test: directories
	gcc -O2 -Wall -Werror -DNO_STANDALONE_MAIN -I../../ports/classic_mac -o $(OUT_DIR)/test_classic_mac $(TEST_SRCS)
	$(OUT_DIR)/test_classic_mac

.PHONY: all classic osx directories clean test
