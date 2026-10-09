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
REZ_INCLUDES  = -I$(RETRO68_ROOT)/RIncludes
REZ_FLAGS     = $(REZ_INCLUDES) \
                -D RUSTID_VERSION_STR="\"$(VERSION)\"" \
                -D VERSION_MAJOR_BCD=$(VERSION_MAJOR_BCD) \
                -D VERSION_MINOR_BCD=$(VERSION_MINOR_BCD)

CFLAGS        = -O2 -Wall -I../../ports/classic_mac -DRUSTID_VERSION=\"$(VERSION)\"
CFLAGS_CARBON = $(CFLAGS) -carbon -DTARGET_API_MAC_CARBON=1
SRC_C         = ../../ports/classic_mac/common/gestalt.c \
                ../../ports/classic_mac/common/display.c \
                ../../ports/classic_mac/common/cpu.c \
                ../../ports/classic_mac/models/models.c \
                ../../ports/classic_mac/m68k/cpu.c \
                ../../ports/classic_mac/ppc/cpu.c \
                ../../ports/classic_mac/gui/window.c \
                ../../ports/classic_mac/gui/menu.c \
                ../../ports/classic_mac/gui/dialogs.c \
                ../../ports/classic_mac/main.c
R_SRC         = rustid.r
R_DEPS        = rustid.r rustid_icons.r

OUT_DIR       ?= ../../target/classic_mac

all: directories $(OUT_DIR)/rustid.bin $(OUT_DIR)/rustid_68k.bin $(OUT_DIR)/rustid_ppc.bin $(OUT_DIR)/rustid_carbon.bin $(OUT_DIR)/Rustid.app $(OUT_DIR)/rustid_hybrid.dmg $(OUT_DIR)/rustid_hybrid.dsk $(OUT_DIR)/Rustid-osx-ppc.tar.gz

classic: directories $(OUT_DIR)/rustid.bin $(OUT_DIR)/rustid_68k.bin $(OUT_DIR)/rustid_ppc.bin
carbon: directories $(OUT_DIR)/rustid_carbon.bin
osx: directories $(OUT_DIR)/Rustid.app $(OUT_DIR)/rustid_hybrid.dmg $(OUT_DIR)/rustid_hybrid.dsk $(OUT_DIR)/Rustid-osx-ppc.tar.gz

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

# PowerPC Classic PEF Executable (InterfaceLib: System 7.1.2 – Mac OS 9.2.2)
$(OUT_DIR)/rustid_ppc.pef: $(SRC_C)
	$(CC_PPC) $(CFLAGS) -o $(OUT_DIR)/rustid_ppc.xcoff $(SRC_C)
	$(MAKEPEF) $(OUT_DIR)/rustid_ppc.xcoff -o $(OUT_DIR)/rustid_ppc.pef

# PowerPC Classic Standalone Binary (System 7.1.2 – Mac OS 9.2.2)
$(OUT_DIR)/rustid_ppc.bin: $(OUT_DIR)/rustid_ppc.pef $(R_DEPS)
	$(REZ) $(REZ_FLAGS) -DTARGET_PPC -o $(OUT_DIR)/rustid_ppc.bin $(R_SRC) \
		--data $(OUT_DIR)/rustid_ppc.pef --cc $(OUT_DIR)/rustid_ppc.dsk -t APPL -c RsId

# PowerPC Carbon PEF Executable (CarbonLib: Mac OS 8.6 – 9.2.2 & Mac OS X Aqua)
$(OUT_DIR)/rustid_carbon.pef: $(SRC_C)
	$(CC_PPC) $(CFLAGS_CARBON) -o $(OUT_DIR)/rustid_carbon.xcoff $(SRC_C)
	$(MAKEPEF) $(OUT_DIR)/rustid_carbon.xcoff -o $(OUT_DIR)/rustid_carbon.pef

# PowerPC Carbon Standalone Binary (Mac OS 8.6 – 9.2.2 with CarbonLib)
$(OUT_DIR)/rustid_carbon.bin: $(OUT_DIR)/rustid_carbon.pef $(R_DEPS)
	$(REZ) $(REZ_FLAGS) -DTARGET_CARBON -o $(OUT_DIR)/rustid_carbon.bin $(R_SRC) \
		--data $(OUT_DIR)/rustid_carbon.pef --cc $(OUT_DIR)/rustid_carbon.dsk -t APPL -c RsId

# Universal Icon File (.icns with Classic QuickDraw, OS X 10.0-10.4 32-bit, and modern PNGs)
$(OUT_DIR)/rustid.icns: rustid_icons.r ../../assets/rustid.png ../../assets/rustid.icns generate_icns.py
	python3 generate_icns.py rustid_icons.r ../../assets/rustid.png ../../assets/rustid.icns $(OUT_DIR)/rustid.icns

# Classic Mac OS Custom Folder Icon (MacBinary Icon\r with ID -16455)
$(OUT_DIR)/folder_icon.bin: rustid_icons.r setup_bundle_icon.py
	python3 setup_bundle_icon.py make-bin rustid_icons.r $(OUT_DIR)/folder_icon.bin

# Mac OS X & Classic Mac OS Hybrid Application Bundle (OS X 10.0–10.5 & Mac OS 9 Classic Environment)
$(OUT_DIR)/Rustid.app: $(OUT_DIR)/rustid_carbon.pef $(OUT_DIR)/rustid_carbon.bin $(OUT_DIR)/rustid.icns Info.plist
	@rm -rf $(OUT_DIR)/Rustid.app $(OUT_DIR)/.rsrc_tmp
	@mkdir -p $(OUT_DIR)/Rustid.app/Contents/MacOS \
		$(OUT_DIR)/Rustid.app/Contents/MacOSClassic \
		$(OUT_DIR)/Rustid.app/Contents/Resources \
		$(OUT_DIR)/.rsrc_tmp
	# 1. Native Mac OS X Aqua executable
	cp $(OUT_DIR)/rustid_carbon.pef $(OUT_DIR)/Rustid.app/Contents/MacOS/rustid
	chmod +x $(OUT_DIR)/Rustid.app/Contents/MacOS/rustid
	# 2. Extract raw data-fork resource map and copy universal icon for OS X
	$(REZ) $(REZ_FLAGS) -DTARGET_CARBON -o $(OUT_DIR)/.rsrc_tmp/dummy.bin $(R_SRC) \
		--data $(OUT_DIR)/rustid_carbon.pef --cc $(OUT_DIR)/.rsrc_tmp/rustid -t APPL -c RsId
	cp $(OUT_DIR)/.rsrc_tmp/.rsrc/rustid $(OUT_DIR)/Rustid.app/Contents/Resources/rustid.rsrc
	cp $(OUT_DIR)/rustid.icns $(OUT_DIR)/Rustid.app/Contents/Resources/rustid.icns
	@rm -rf $(OUT_DIR)/.rsrc_tmp
	# 3. Classic Mac OS executable slice (Contents/MacOSClassic/rustid with native resource fork & AppleDouble)
	$(REZ) $(REZ_FLAGS) -DTARGET_CARBON -o $(OUT_DIR)/Rustid.app/Contents/MacOSClassic/rustid.bin $(R_SRC) \
		--data $(OUT_DIR)/rustid_carbon.pef \
		--cc $(OUT_DIR)/Rustid.app/Contents/MacOSClassic/rustid \
		--cc $(OUT_DIR)/Rustid.app/Contents/MacOSClassic/._rustid \
		-t APPL -c RsId
	chmod +x $(OUT_DIR)/Rustid.app/Contents/MacOSClassic/rustid
	# 4. Standalone Classic executable at the bundle root for Mac OS 9 Finder navigation
	$(REZ) $(REZ_FLAGS) -DTARGET_CARBON -o "$(OUT_DIR)/Rustid.app/Rustid.bin" $(R_SRC) \
		--data $(OUT_DIR)/rustid_carbon.pef \
		--cc "$(OUT_DIR)/Rustid.app/Rustid" \
		--cc "$(OUT_DIR)/Rustid.app/._Rustid" \
		-t APPL -c RsId
	chmod +x "$(OUT_DIR)/Rustid.app/Rustid"
	# 5. Bundle manifests
	cp Info.plist $(OUT_DIR)/Rustid.app/Contents/Info.plist
	echo -n "APPLRsId" > $(OUT_DIR)/Rustid.app/Contents/PkgInfo

# Hybrid HFS Disk Image (contains Rustid.app with custom icon, and standalone Classic applications)
# .dmg is the native Apple Disk Image format on Mac OS X (mounted natively by DiskImageMounter.app)
$(OUT_DIR)/rustid_hybrid.dmg: $(OUT_DIR)/Rustid.app $(OUT_DIR)/folder_icon.bin $(OUT_DIR)/rustid_carbon.bin $(OUT_DIR)/rustid_ppc.bin $(OUT_DIR)/rustid.bin
	dd if=/dev/zero of=$(OUT_DIR)/rustid_hybrid.dmg bs=1M count=14 status=none
	$(RETRO68_ROOT)/bin/hformat -l "Rustid Hybrid" $(OUT_DIR)/rustid_hybrid.dmg
	$(RETRO68_ROOT)/bin/hmount $(OUT_DIR)/rustid_hybrid.dmg
	$(RETRO68_ROOT)/bin/hcopy -m $(OUT_DIR)/rustid_carbon.bin ":Rustid (Carbon)"
	$(RETRO68_ROOT)/bin/hcopy -m $(OUT_DIR)/rustid_ppc.bin ":Rustid (Classic PPC)"
	$(RETRO68_ROOT)/bin/hcopy -m $(OUT_DIR)/rustid.bin ":Rustid (Universal Fat)"
	$(RETRO68_ROOT)/bin/hmkdir ":Rustid.app"
	$(RETRO68_ROOT)/bin/hcopy -m $(OUT_DIR)/rustid_carbon.bin ":Rustid.app:Rustid"
	python3 setup_bundle_icon.py copy-icon $(RETRO68_ROOT)/bin/hcopy $(OUT_DIR)/folder_icon.bin "Rustid.app"
	$(RETRO68_ROOT)/bin/hmkdir ":Rustid.app:Contents"
	$(RETRO68_ROOT)/bin/hmkdir ":Rustid.app:Contents:MacOS"
	$(RETRO68_ROOT)/bin/hmkdir ":Rustid.app:Contents:MacOSClassic"
	$(RETRO68_ROOT)/bin/hmkdir ":Rustid.app:Contents:Resources"
	$(RETRO68_ROOT)/bin/hcopy Info.plist ":Rustid.app:Contents:Info.plist"
	$(RETRO68_ROOT)/bin/hcopy $(OUT_DIR)/Rustid.app/Contents/PkgInfo ":Rustid.app:Contents:PkgInfo"
	$(RETRO68_ROOT)/bin/hcopy $(OUT_DIR)/Rustid.app/Contents/MacOS/rustid ":Rustid.app:Contents:MacOS:rustid"
	$(RETRO68_ROOT)/bin/hcopy $(OUT_DIR)/rustid.icns ":Rustid.app:Contents:Resources:rustid.icns"
	$(RETRO68_ROOT)/bin/hcopy $(OUT_DIR)/Rustid.app/Contents/Resources/rustid.rsrc ":Rustid.app:Contents:Resources:rustid.rsrc"
	$(RETRO68_ROOT)/bin/hcopy -m $(OUT_DIR)/rustid_carbon.bin ":Rustid.app:Contents:MacOSClassic:rustid"
	$(RETRO68_ROOT)/bin/humount
	python3 setup_bundle_icon.py set-flag $(OUT_DIR)/rustid_hybrid.dmg "Rustid.app"

# Classic emulators (SheepShaver, Basilisk II, Floppy Emu) expecting .dsk format
$(OUT_DIR)/rustid_hybrid.dsk: $(OUT_DIR)/rustid_hybrid.dmg
	cp -f $(OUT_DIR)/rustid_hybrid.dmg $(OUT_DIR)/rustid_hybrid.dsk

# Mac OS X Compressed Distribution Archive
$(OUT_DIR)/Rustid-osx-ppc.tar.gz: $(OUT_DIR)/Rustid.app
	tar -C $(OUT_DIR) -czf $(OUT_DIR)/Rustid-osx-ppc.tar.gz Rustid.app

clean:
	rm -rf $(OUT_DIR)

TEST_SRCS = $(filter-out ../../ports/classic_mac/main.c,$(SRC_C)) ../../ports/classic_mac/test_classic_mac.c

test: directories
	gcc -O2 -Wall -Werror -DNO_STANDALONE_MAIN -I../../ports/classic_mac -o $(OUT_DIR)/test_classic_mac $(TEST_SRCS)
	$(OUT_DIR)/test_classic_mac

.PHONY: all classic carbon osx directories clean test
