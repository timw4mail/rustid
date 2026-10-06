#!/usr/bin/env python3
"""
setup_bundle_icon.py - Generate Classic Mac custom folder icon and set kHasCustomIcon.

1. Generate 'folder_icon.bin' (MacBinary II format of an invisible 'Icon\\r' file)
   with resource ID -16455 (kCustomIconResource) containing the icon suite
   parsed from 'rustid_icons.r'.
2. Set the 'kHasCustomIcon' bit (0x0004) in the folder's DInfo.frFlags in the HFS disk image.
"""

import sys
import os
import struct
import re

def calc_crc(data: bytes) -> int:
    crc_table = []
    for i in range(256):
        curr = i << 8
        for _ in range(8):
            if curr & 0x8000:
                curr = ((curr << 1) ^ 0x1021) & 0xFFFF
            else:
                curr = (curr << 1) & 0xFFFF
        crc_table.append(curr)

    crc = 0
    for b in data:
        crc = ((crc << 8) & 0xFFFF) ^ crc_table[(crc >> 8) ^ b]
    return crc

def build_resource_fork(resources):
    res_data_bytes = bytearray()
    ref_entries = []

    for rtype, rid, rdata in resources:
        offset = len(res_data_bytes)
        res_data_bytes.extend(struct.pack('>I', len(rdata)))
        res_data_bytes.extend(rdata)
        ref_entries.append((rtype, rid, offset))

    data_len = len(res_data_bytes)
    header_len = 256

    from collections import OrderedDict
    by_type = OrderedDict()
    for rtype, rid, offset in ref_entries:
        if rtype not in by_type:
            by_type[rtype] = []
        by_type[rtype].append((rid, offset))

    type_list = bytearray()
    ref_list = bytearray()

    type_count = len(by_type)
    type_list.extend(struct.pack('>H', type_count - 1))

    type_list_offset = 28
    ref_list_offset_from_types = 2 + type_count * 8

    curr_ref_offset = ref_list_offset_from_types
    for rtype, items in by_type.items():
        type_list.extend(struct.pack('>4sHH', rtype, len(items) - 1, curr_ref_offset))
        curr_ref_offset += len(items) * 12
        for rid, roffset in items:
            ref_list.extend(struct.pack('>Hh', rid, -1))
            ref_list.append(0)
            ref_list.extend(roffset.to_bytes(3, 'big'))
            ref_list.extend(b'\x00\x00\x00\x00')

    name_list = b'\x00\x00'
    map_data = bytearray()
    map_data.extend(b'\x00' * 16)
    map_data.extend(struct.pack('>IH', 0, 0))
    map_data.extend(struct.pack('>HH', 0, type_list_offset))
    map_data.extend(struct.pack('>H', type_list_offset + len(type_list) + len(ref_list)))
    map_data.extend(type_list)
    map_data.extend(ref_list)
    map_data.extend(name_list)
    map_len = len(map_data)

    header = struct.pack('>IIII', header_len, header_len + data_len, data_len, map_len)
    header += b'\x00' * (header_len - 16)

    map_data[0:16] = header[:16]
    return header + res_data_bytes + map_data

def build_macbinary(filename, ftype, creator, flags, data_fork, rsrc_fork):
    hdr = bytearray(128)
    name_bytes = filename.encode('latin1')
    hdr[1] = len(name_bytes)
    hdr[2:2+len(name_bytes)] = name_bytes
    hdr[65:69] = ftype
    hdr[69:73] = creator
    hdr[73:75] = struct.pack('>H', flags)
    hdr[83:87] = struct.pack('>I', len(data_fork))
    hdr[87:91] = struct.pack('>I', len(rsrc_fork))
    hdr[122] = 129
    hdr[123] = 129
    crc = calc_crc(hdr[:124])
    hdr[124:126] = struct.pack('>H', crc)

    def pad(b):
        rem = len(b) % 128
        return b if rem == 0 else b + (b'\x00' * (128 - rem))

    return bytes(hdr) + pad(data_fork) + pad(rsrc_fork)

def make_folder_icon_macbinary(icons_r_path, out_bin_path):
    with open(icons_r_path, 'r', encoding='latin1') as f:
        r_text = f.read()

    def get_hex(r_type):
        m = re.search(r'resource \'' + r_type + r'\' \(128[^\)]*\) \{(.*?)\};', r_text, re.DOTALL)
        if not m: return None
        body = m.group(1)
        hex_str = ''.join(re.findall(r'\"([0-9a-fA-F ]+)\"', body)).replace(' ', '')
        return bytes.fromhex(hex_str)

    rid = 49081 # (unsigned short)-16455
    resources = []
    for tag in ['ICN#', 'ics#', 'icl8', 'ics8', 'icl4', 'ics4']:
        data = get_hex(tag)
        if data:
            resources.append((tag.encode('latin1'), rid, data))

    rsrc = build_resource_fork(resources)
    mb = build_macbinary('Icon\r', b'icon', b'MACS', 0x4000, b'', rsrc)
    with open(out_bin_path, 'wb') as f:
        f.write(mb)
    print(f"Generated {out_bin_path} ({len(mb)} bytes)")

def set_hfs_folder_custom_icon(dsk_path, folder_name):
    name_bytes = folder_name.encode('latin1')
    target_key = bytes([len(name_bytes)]) + name_bytes
    with open(dsk_path, 'r+b') as f:
        data = f.read()
        idx = 0
        found = False
        while True:
            idx = data.find(target_key, idx)
            if idx == -1: break
            # After target_key, check if next byte is pad or record
            for pad in [1, 0]:
                offset = idx + len(target_key) + pad
                if offset + 32 <= len(data):
                    cdrType = struct.unpack('>H', data[offset:offset+2])[0]
                    if cdrType == 1: # Directory record
                        flags_offset = offset + 2 + 2 + 2 + 4 + 4 + 4 + 4 + 8
                        flags = struct.unpack('>H', data[flags_offset:flags_offset+2])[0]
                        flags |= 0x0004 # kHasCustomIcon
                        f.seek(flags_offset)
                        f.write(struct.pack('>H', flags))
                        print(f"Set kHasCustomIcon (0x{flags:04x}) on '{folder_name}' in {dsk_path}")
                        found = True
                        break
            if found:
                break
            idx += 1
        if not found:
            print(f"Warning: Folder '{folder_name}' not found in HFS catalog of {dsk_path}")

import subprocess

def copy_icon_to_hfs(hcopy_path, folder_icon_bin, folder_name):
    target = f":{folder_name}:Icon\r"
    res = subprocess.run([hcopy_path, '-m', folder_icon_bin, target], capture_output=True, text=True)
    if res.returncode != 0:
        print(f"Error running hcopy: {res.stderr}")
        sys.exit(res.returncode)
    print(f"Successfully copied Icon\\r into :{folder_name}:")

if __name__ == '__main__':
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} make-bin <icons.r> <out.bin>")
        print(f"   or: {sys.argv[0]} copy-icon <hcopy_path> <folder_icon.bin> <folder_name>")
        print(f"   or: {sys.argv[0]} set-flag <image.dsk> <folder_name>")
        sys.exit(1)

    cmd = sys.argv[1]
    if cmd == 'make-bin':
        make_folder_icon_macbinary(sys.argv[2], sys.argv[3])
    elif cmd == 'copy-icon':
        copy_icon_to_hfs(sys.argv[2], sys.argv[3], sys.argv[4])
    elif cmd == 'set-flag':
        set_hfs_folder_custom_icon(sys.argv[2], sys.argv[3])
    else:
        print(f"Unknown command: {cmd}")
        sys.exit(1)
