# Haiku Icon & Resource Assets for Rustid

This directory contains application resources and icons for Haiku OS.

## Files

- `rustid.rdef`: Haiku Resource Definition script containing:
  - Application signature (`application/x-vnd.rustid`)
  - Application flags (`B_SINGLE_LAUNCH`)
  - Version information
  - Standard 32x32 BeOS/Haiku bitmap icon (`large_icon`, B_CMAP8)
  - Mini 16x16 BeOS/Haiku bitmap icon (`mini_icon`, B_CMAP8)
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
To create Haiku's native Vector Icon Format (`.hvif`), you can:
- Open `rustid_128.png` in **SVGear** (available via HaikuDepot) and trace/export as `rustid.hvif`.
- Or use **Icon-O-Matic** on Haiku to save `rustid.hvif`.
- Then attach it to the binary:
  ```sh
  addattr -f rustid.hvif -t icon BEOS:ICON target/dist/rustid_haiku
  ```
