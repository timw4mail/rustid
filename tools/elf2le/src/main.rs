/// elf2le: Convert a 32-bit i486 ELF binary to a DOS Linear Executable (LE) format
/// suitable for use with the DOS/32A extender.
///
/// The output is a 2-object LE file (Code RX, Data RW) with relocation records
/// parsed directly from ELF SHT_REL sections (R_386_32 relocations).
use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::{Read, Result, Write};

const PAGE_SIZE: usize = 4096;
const SHT_REL: u32 = 9;
const R_386_32: u32 = 1;

fn u32_le(data: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(data[off..off + 4].try_into().unwrap())
}

fn u16_le(data: &[u8], off: usize) -> u16 {
    u16::from_le_bytes(data[off..off + 2].try_into().unwrap())
}

fn io_err(msg: &'static str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, msg)
}

fn align_up(val: usize, align: usize) -> usize {
    (val + align - 1) & !(align - 1)
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let input_path = args.get(1).map(String::as_str).unwrap_or("dos32a_rustid");
    let output_path = args.get(2).map(String::as_str).unwrap_or("rustid.le");

    let mut file = File::open(input_path)?;
    let mut elf = Vec::new();
    file.read_to_end(&mut elf)?;

    if elf.len() < 52 || &elf[0..4] != b"\x7FELF" {
        return Err(io_err("Not an ELF file"));
    }

    let e_entry = u32_le(&elf, 24);
    let e_phoff = u32_le(&elf, 28) as usize;
    let e_phentsize = u16_le(&elf, 42) as usize;
    let e_phnum = u16_le(&elf, 44) as usize;
    let e_shoff = u32_le(&elf, 32) as usize;
    let e_shentsize = u16_le(&elf, 46) as usize;
    let e_shnum = u16_le(&elf, 48) as usize;
    let e_shstrndx = u16_le(&elf, 50) as usize;

    let mut min_vaddr = u32::MAX;
    let mut max_vaddr = 0u32;
    for i in 0..e_phnum {
        let off = e_phoff + i * e_phentsize;
        let p_type = u32_le(&elf, off);
        let p_va = u32_le(&elf, off + 8);
        let p_memsz = u32_le(&elf, off + 20);
        if p_type == 1 && p_memsz > 0 {
            min_vaddr = min_vaddr.min(p_va);
            max_vaddr = max_vaddr.max(p_va + p_memsz);
        }
    }

    let sh_entry = |i: usize| {
        let off = e_shoff + i * e_shentsize;
        (
            u32_le(&elf, off),
            u32_le(&elf, off + 4),
            u32_le(&elf, off + 12),
            u32_le(&elf, off + 16),
            u32_le(&elf, off + 20),
        )
    };

    let (_, _, _, shstr_foff, shstr_size) = sh_entry(e_shstrndx);
    let shstrtab = &elf[shstr_foff as usize..(shstr_foff + shstr_size) as usize];

    let sh_name_str = |name_off: usize| {
        if name_off >= shstrtab.len() {
            return "";
        }
        let end = shstrtab[name_off..]
            .iter()
            .position(|&b| b == 0)
            .map(|p| name_off + p)
            .unwrap_or(shstrtab.len());
        std::str::from_utf8(&shstrtab[name_off..end]).unwrap_or("")
    };

    let mut text_va = 0u32;
    let mut text_size = 0u32;
    let mut text_foff = 0u32;
    let mut data_va = 0u32;
    let mut data_filesz = 0u32;
    let mut data_memsz = 0u32;

    for i in 0..e_shnum {
        let (s_name, _, s_va, s_foff, s_size) = sh_entry(i);
        let name = sh_name_str(s_name as usize);
        if name == ".text" {
            text_va = s_va;
            text_size = s_size;
            text_foff = s_foff;
        } else if name == ".rodata" {
            if data_va == 0 || s_va < data_va {
                data_va = s_va;
            }
            data_filesz += s_size;
        } else if name == ".bss" {
            if data_va == 0 || s_va < data_va {
                data_va = s_va;
            }
            data_memsz = (s_va + s_size) - data_va;
        }
    }

    let heap_size = 0x100000; // 1MB heap
    let obj2_total_memsz = data_memsz as usize + heap_size;
    let obj1_pages = align_up(text_size as usize, PAGE_SIZE) / PAGE_SIZE;
    let obj2_file_pages = align_up(data_filesz as usize, PAGE_SIZE) / PAGE_SIZE;
    let obj2_total_pages = align_up(obj2_total_memsz, PAGE_SIZE) / PAGE_SIZE;
    let total_page_count = obj1_pages + obj2_total_pages;

    let mut code_data = vec![0u8; text_size as usize];
    code_data.copy_from_slice(&elf[text_foff as usize..(text_foff + text_size) as usize]);
    let code_padded_len = align_up(code_data.len(), PAGE_SIZE);
    code_data.resize(code_padded_len, 0);

    let mut data_file_data = vec![0u8; data_filesz as usize];
    for i in 0..e_shnum {
        let (s_name, _, s_va, s_foff, s_size) = sh_entry(i);
        let name = sh_name_str(s_name as usize);
        if (name == ".rodata" || name == ".data") && s_size > 0 {
            let rel = (s_va - data_va) as usize;
            let src = &elf[s_foff as usize..(s_foff + s_size) as usize];
            data_file_data[rel..rel + s_size as usize].copy_from_slice(src);
        }
    }
    let data_padded_len = align_up(data_file_data.len(), PAGE_SIZE);
    data_file_data.resize(data_padded_len, 0);

    // Collect R_386_32 relocations
    let mut fixups_by_page: BTreeMap<usize, Vec<(u16, u8, u32)>> = BTreeMap::new();
    let mut reloc_count = 0;

    for i in 0..e_shnum {
        let (_, s_type, _, _, s_foff, s_size) = {
            let off = e_shoff + i * e_shentsize;
            (
                u32_le(&elf, off),
                u32_le(&elf, off + 4),
                u32_le(&elf, off + 8),
                u32_le(&elf, off + 12),
                u32_le(&elf, off + 16),
                u32_le(&elf, off + 20),
            )
        };
        if s_type == SHT_REL {
            let n_rel = (s_size / 8) as usize;
            for r_i in 0..n_rel {
                let r_off = u32_le(&elf, s_foff as usize + r_i * 8);
                let r_info = u32_le(&elf, s_foff as usize + r_i * 8 + 4);
                let r_type = r_info & 0xFF;

                if r_type == R_386_32 {
                    reloc_count += 1;
                    let abs_addr = r_off;
                    let val = if abs_addr < data_va {
                        u32_le(&code_data, (abs_addr - text_va) as usize)
                    } else {
                        u32_le(&data_file_data, (abs_addr - data_va) as usize)
                    };

                    let (page_idx, src_off_in_page) = if abs_addr < data_va {
                        let rel_off = (abs_addr - text_va) as usize;
                        (rel_off / PAGE_SIZE, (rel_off % PAGE_SIZE) as u16)
                    } else {
                        let rel_off = (abs_addr - data_va) as usize;
                        (obj1_pages + (rel_off / PAGE_SIZE), (rel_off % PAGE_SIZE) as u16)
                    };

                    let (tgt_obj, tgt_off) = if val < data_va {
                        (1u8, val - text_va)
                    } else {
                        (2u8, val - data_va)
                    };

                    fixups_by_page
                        .entry(page_idx)
                        .or_default()
                        .push((src_off_in_page, tgt_obj, tgt_off));
                }
            }
        }
    }

    let mut fixup_records = Vec::new();
    let mut fixup_page_table = Vec::new();

    for p in 0..total_page_count {
        fixup_page_table.push(fixup_records.len() as u32);
        if let Some(list) = fixups_by_page.get_mut(&p) {
            list.sort_by_key(|x| x.0);
            list.dedup_by_key(|x| x.0);
            for &(src_off, tgt_obj, tgt_off) in list.iter() {
                fixup_records.push(7);    // 32-bit offset
                fixup_records.push(0x10); // internal reference, 32-bit target offset
                fixup_records.extend_from_slice(&src_off.to_le_bytes());
                fixup_records.push(tgt_obj);
                fixup_records.extend_from_slice(&tgt_off.to_le_bytes());
            }
        }
    }
    fixup_page_table.push(fixup_records.len() as u32);

    let eip_offset = e_entry - text_va;
    let lea_esp_offset = (eip_offset + 11) as usize;
    let esp_abs = u32_le(&code_data, lea_esp_offset + 2);
    let esp_offset = esp_abs - data_va;

    let le_hdr_size: u32 = 192;
    let obj_count: u32 = 2;
    let obj_table_rel = le_hdr_size;
    let obj_page_map_rel = obj_table_rel + obj_count * 24;

    let module_name = b"RUSTID";
    let mut res_names_bytes = vec![module_name.len() as u8];
    res_names_bytes.extend_from_slice(module_name);
    res_names_bytes.extend_from_slice(&[0, 0, 0]);
    let res_names_rel = obj_page_map_rel + total_page_count as u32 * 4;

    let entry_table_rel = res_names_rel + res_names_bytes.len() as u32;
    let entry_table_size: u32 = 1;

    let fixup_page_tbl_rel = entry_table_rel + entry_table_size;
    let fixup_page_tbl_size = (total_page_count as u32 + 1) * 4;

    let fixup_rec_tbl_rel = fixup_page_tbl_rel + fixup_page_tbl_size;
    let fixup_rec_tbl_size = fixup_records.len() as u32;

    let imported_modules_rel = fixup_rec_tbl_rel + fixup_rec_tbl_size;
    let loader_section_size = fixup_page_tbl_rel - obj_table_rel + fixup_page_tbl_size;
    let fixup_section_size = fixup_page_tbl_size + fixup_rec_tbl_size;

    let tables_end = imported_modules_rel as usize;
    let data_pages_offset = align_up(tables_end, PAGE_SIZE);

    let mut le_hdr = vec![0u8; 192];
    le_hdr[0..2].copy_from_slice(b"LE");
    le_hdr[8..10].copy_from_slice(&2u16.to_le_bytes()); // i386
    le_hdr[10..12].copy_from_slice(&1u16.to_le_bytes()); // OS/2
    le_hdr[16..20].copy_from_slice(&0u32.to_le_bytes()); // fixups to be applied by loader
    le_hdr[20..24].copy_from_slice(&(total_page_count as u32).to_le_bytes());

    le_hdr[24..28].copy_from_slice(&1u32.to_le_bytes()); // CS = 1
    le_hdr[28..32].copy_from_slice(&eip_offset.to_le_bytes());
    le_hdr[32..36].copy_from_slice(&2u32.to_le_bytes()); // SS = 2
    le_hdr[36..40].copy_from_slice(&esp_offset.to_le_bytes());

    le_hdr[40..44].copy_from_slice(&(PAGE_SIZE as u32).to_le_bytes());
    le_hdr[48..52].copy_from_slice(&fixup_section_size.to_le_bytes());
    le_hdr[56..60].copy_from_slice(&loader_section_size.to_le_bytes());

    le_hdr[0x40..0x44].copy_from_slice(&obj_table_rel.to_le_bytes());
    le_hdr[0x44..0x48].copy_from_slice(&obj_count.to_le_bytes());
    le_hdr[0x48..0x4C].copy_from_slice(&obj_page_map_rel.to_le_bytes());
    le_hdr[0x58..0x5C].copy_from_slice(&res_names_rel.to_le_bytes());
    le_hdr[0x5C..0x60].copy_from_slice(&entry_table_rel.to_le_bytes());
    le_hdr[0x68..0x6C].copy_from_slice(&fixup_page_tbl_rel.to_le_bytes());
    le_hdr[0x6C..0x70].copy_from_slice(&fixup_rec_tbl_rel.to_le_bytes());
    le_hdr[0x70..0x74].copy_from_slice(&imported_modules_rel.to_le_bytes());
    le_hdr[0x80..0x84].copy_from_slice(&(data_pages_offset as u32).to_le_bytes());
    le_hdr[0x94..0x98].copy_from_slice(&2u32.to_le_bytes()); // Auto DS = 2
    le_hdr[0xA8..0xAC].copy_from_slice(&(heap_size as u32).to_le_bytes()); // Extra heap

    let mut obj_table = Vec::new();
    // Object 1: Code (RX)
    let obj1_vsize = (obj1_pages * PAGE_SIZE) as u32;
    obj_table.extend_from_slice(&obj1_vsize.to_le_bytes());
    obj_table.extend_from_slice(&text_va.to_le_bytes());
    obj_table.extend_from_slice(&0x2045u32.to_le_bytes());
    obj_table.extend_from_slice(&1u32.to_le_bytes());
    obj_table.extend_from_slice(&(obj1_pages as u32).to_le_bytes());
    obj_table.extend_from_slice(&0u32.to_le_bytes());

    // Object 2: Data (RW)
    let obj2_vsize = (obj2_total_pages * PAGE_SIZE) as u32;
    obj_table.extend_from_slice(&obj2_vsize.to_le_bytes());
    obj_table.extend_from_slice(&data_va.to_le_bytes());
    obj_table.extend_from_slice(&0x2043u32.to_le_bytes());
    obj_table.extend_from_slice(&((obj1_pages + 1) as u32).to_le_bytes());
    obj_table.extend_from_slice(&(obj2_total_pages as u32).to_le_bytes());
    obj_table.extend_from_slice(&0u32.to_le_bytes());

    let mut obj_page_map = Vec::new();
    let mut page_idx = 1u16;
    for _ in 0..obj1_pages {
        obj_page_map.extend_from_slice(&[0, (page_idx >> 8) as u8, page_idx as u8, 0]);
        page_idx += 1;
    }
    for p in 0..obj2_total_pages {
        if p < obj2_file_pages {
            obj_page_map.extend_from_slice(&[0, (page_idx >> 8) as u8, page_idx as u8, 0]);
            page_idx += 1;
        } else {
            obj_page_map.extend_from_slice(&[0, 0, 0, 3]); // zero-fill
        }
    }

    let mut fixup_page_tbl_bytes = Vec::new();
    for v in fixup_page_table {
        fixup_page_tbl_bytes.extend_from_slice(&v.to_le_bytes());
    }

    let mut file_buf = Vec::new();
    file_buf.extend_from_slice(&le_hdr);
    file_buf.extend_from_slice(&obj_table);
    file_buf.extend_from_slice(&obj_page_map);
    file_buf.extend_from_slice(&res_names_bytes);
    file_buf.push(0); // entry table terminator
    file_buf.extend_from_slice(&fixup_page_tbl_bytes);
    file_buf.extend_from_slice(&fixup_records);

    while file_buf.len() < data_pages_offset {
        file_buf.push(0);
    }

    file_buf.extend_from_slice(&code_data);
    file_buf.extend_from_slice(&data_file_data);

    let final_len = align_up(file_buf.len(), PAGE_SIZE);
    file_buf.resize(final_len, 0);

    let mut out = File::create(output_path)?;
    out.write_all(&file_buf)?;

    println!(
        "elf2le: parsed {} R_386_32 relocations -> wrote {} ({} bytes)",
        reloc_count,
        output_path,
        file_buf.len()
    );

    Ok(())
}
