# Haiku Icon & Resource Assets for Rustid

This directory contains application resources and icons for Haiku OS.

## Files

- `rustid.rdef`: Haiku Resource Definition script containing:
  - Application signature (`application/x-vnd.rustid-gui`)
  - Application flags (`B_SINGLE_LAUNCH`)
  - Version information
  - Native Vector Icon (`vector_icon`, HVIF)
  - Standard 32x32 BeOS/Haiku bitmap icon (`large_icon`, B_CMAP8 fallback)
  - Mini 16x16 BeOS/Haiku bitmap icon (`mini_icon`, B_CMAP8 fallback)
- `rustid.hvif`: Haiku Vector Icon Format binary icon
- `rustid_16.png`: 16×16 PNG icon
- `rustid_32.png`: 32×32 PNG icon
- `rustid_64.png`: 64×64 PNG icon
- `rustid_128.png`: 128×128 PNG icon

## Building Resources on Haiku

### 1. Compile .rdef with Resource Compiler (`rc`):
```sh
rc -o rustid.rsrc rustid.rdef
```

### 2. Merge .rsrc into the Application Binary (`xres`):
```sh
xres -o target/dist/rustid_haiku rustid.rsrc
mimeset -f target/dist/rustid_haiku
```

### 3. Vector Icon (HVIF):
The native Vector Icon Format (`.hvif`) is compiled directly into the `.rsrc` from `rustid.rdef`.
You can also attach `rustid.hvif` directly to any binary:
```sh
addattr -f assets/haiku/rustid.hvif -t icon BEOS:ICON target/dist/rustid_haiku
```
