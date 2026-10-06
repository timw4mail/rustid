#!/usr/bin/env python3
"""
Generate a Universal Mac OS X / Classic Mac OS composite ICNS file.
Contains:
  - Classic Mac OS QuickDraw icon suite ('ICN#', 'icl8', 'ics#', 'ics8')
  - Mac OS X 10.0-10.4 Tiger Icon Services chunks ('it32'+'t8mk', 'ih32'+'h8mk', 'il32'+'l8mk', 'is32'+'s8mk')
  - Modern Mac OS X 10.5+ / macOS high-resolution PNG chunks ('ic07' through 'ic14')
"""

import sys
import os
import struct
import subprocess
import shutil
import re

def packbits(data: bytes) -> bytes:
    i = 0
    out = bytearray()
    while i < len(data):
        run_len = 1
        while i + run_len < len(data) and run_len < 128 and data[i + run_len] == data[i]:
            run_len += 1
        if run_len >= 3:
            out.append(257 - run_len)
            out.append(data[i])
            i += run_len
        else:
            lit_len = 1
            while i + lit_len < len(data) and lit_len < 128:
                if i + lit_len + 2 < len(data) and data[i + lit_len] == data[i + lit_len + 1] == data[i + lit_len + 2]:
                    break
                lit_len += 1
            out.append(lit_len - 1)
            out.extend(data[i:i + lit_len])
            i += lit_len
    return bytes(out)

def get_rgba(im_bin, src_png, size):
    cmd = [im_bin, src_png, '-resize', f'{size}x{size}!', 'rgba:-']
    res = subprocess.run(cmd, stdout=subprocess.PIPE, check=True)
    return res.stdout

def main():
    if len(sys.argv) < 5:
        print(f"Usage: {sys.argv[0]} <icons.r> <src.png> <src.icns> <out.icns>")
        sys.exit(1)

    icons_r_path = sys.argv[1]
    src_png_path = sys.argv[2]
    src_icns_path = sys.argv[3]
    out_icns_path = sys.argv[4]

    chunks = []

    # 1. Parse Classic QuickDraw icon resources from rustid_icons.r
    with open(icons_r_path, 'r', encoding='latin1') as f:
        r_text = f.read()

    def get_hex(r_type):
        m = re.search(r'resource \'' + r_type + r'\' \(128[^\)]*\) \{(.*?)\};', r_text, re.DOTALL)
        if not m:
            return None
        body = m.group(1)
        hex_str = ''.join(re.findall(r'\"([0-9a-fA-F ]+)\"', body)).replace(' ', '')
        return bytes.fromhex(hex_str)

    icn_data = get_hex('ICN#')
    icl8_data = get_hex('icl8')
    ics_data = get_hex('ics#')
    ics8_data = get_hex('ics8')

    if icn_data:
        chunks.append((b'ICN#', icn_data))
    if icl8_data:
        chunks.append((b'icl8', icl8_data))
    if ics_data:
        chunks.append((b'ics#', ics_data))
    if ics8_data:
        chunks.append((b'ics8', ics8_data))

    # 2. Generate OS X 10.0-10.4 32-bit RGB + 8-bit Alpha mask chunks
    im_bin = shutil.which('magick') or shutil.which('convert')
    if im_bin and os.path.exists(src_png_path):
        for size, rgb_tag, mask_tag in [
            (128, b'it32', b't8mk'),
            (48,  b'ih32', b'h8mk'),
            (32,  b'il32', b'l8mk'),
            (16,  b'is32', b's8mk')
        ]:
            raw = get_rgba(im_bin, src_png_path, size)
            r = raw[0::4]
            g = raw[1::4]
            b = raw[2::4]
            a = raw[3::4]
            # it32/ih32/il32/is32 have 4 reserved bytes of 0x00 followed by PackBits-compressed R, G, B
            rgb_data = b'\x00\x00\x00\x00' + packbits(r) + packbits(g) + packbits(b)
            chunks.append((rgb_tag, rgb_data))
            chunks.append((mask_tag, a))

    # 3. Modern PNG chunks from existing icns
    if os.path.exists(src_icns_path):
        with open(src_icns_path, 'rb') as f:
            header = f.read(8)
            if len(header) == 8 and header[:4] == b'icns':
                total_len = struct.unpack('>I', header[4:8])[0]
                while f.tell() < total_len:
                    chunk_hdr = f.read(8)
                    if len(chunk_hdr) < 8:
                        break
                    tag, chunk_len = struct.unpack('>4sI', chunk_hdr)
                    data = f.read(chunk_len - 8)
                    # Include modern icon chunks and info
                    if tag.startswith(b'ic') or tag in (b'info',):
                        chunks.append((tag, data))

    # 4. Serialize into output .icns file
    body = bytearray()
    for tag, data in chunks:
        body.extend(tag)
        body.extend(struct.pack('>I', len(data) + 8))
        body.extend(data)

    total_file_len = len(body) + 8
    full_icns = b'icns' + struct.pack('>I', total_file_len) + body

    os.makedirs(os.path.dirname(os.path.abspath(out_icns_path)), exist_ok=True)
    with open(out_icns_path, 'wb') as f:
        f.write(full_icns)

    print(f"Generated universal icns at {out_icns_path} ({total_file_len} bytes, {len(chunks)} chunks)")

if __name__ == '__main__':
    main()
