# Makefile for building rustid for Classic Mac OS (68k and PowerPC) using Retro68

CC_68K   ?= m68k-apple-macos-gcc
CC_PPC   ?= powerpc-apple-macos-gcc
REZ      ?= Rez
MAKEPEF  ?= MakePEF

CFLAGS   = -O2 -Wall -I../../src/gui/classic_mac/bridge
SRC_68K  = ../../src/gui/classic_mac/bridge/mac_bridge.c ../../src/gui/classic_mac/bridge/classic_mac_engine.c
R_SRC    = rustid.r

OUT_DIR  ?= ../../target/classic_mac

all: directories $(OUT_DIR)/rustid_68k.bin $(OUT_DIR)/rustid_ppc.bin

directories:
	@mkdir -p $(OUT_DIR)

# 68k Binary (System 6 - 8.1)
$(OUT_DIR)/rustid_68k.bin: $(SRC_68K) $(R_SRC)
	$(CC_68K) $(CFLAGS) -o $(OUT_DIR)/rustid_68k $(SRC_68K)
	$(REZ) -o $(OUT_DIR)/rustid_68k.bin $(R_SRC) --append $(OUT_DIR)/rustid_68k

# PowerPC PEF Binary (System 7.1.2 - Mac OS 9.2.2)
$(OUT_DIR)/rustid_ppc.bin: $(SRC_68K) $(R_SRC)
	$(CC_PPC) $(CFLAGS) -o $(OUT_DIR)/rustid_ppc $(SRC_68K)
	$(REZ) -o $(OUT_DIR)/rustid_ppc.bin $(R_SRC) --append $(OUT_DIR)/rustid_ppc

clean:
	rm -rf $(OUT_DIR)

.PHONY: all directories clean
