use std::env;
use std::fs::File;
use std::io::{Read, Result, Write};

#[repr(C, packed)]
struct MzHeader {
    signature: [u8; 2],      // 'MZ'
    extra_bytes: u16,        // Bytes on last page
    pages: u16,              // 512-byte pages in file
    relocations: u16,        // Number of relocation entries
    header_paragraphs: u16,  // Header size in 16-byte paragraphs
    min_alloc: u16,          // Minimum extra paragraphs needed
    max_alloc: u16,          // Maximum extra paragraphs needed
    ss: u16,                 // Initial relative SS
    sp: u16,                 // Initial SP
    checksum: u16,           // Checksum (usually 0)
    ip: u16,                 // Initial IP
    cs: u16,                 // Initial relative CS
    reloc_table_offset: u16, // Offset to relocation table
    overlay_number: u16,     // Overlay number
}

fn extract_binary_from_elf(elf_data: &[u8]) -> Result<Vec<u8>> {
    if elf_data.len() < 52 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "ELF file too small",
        ));
    }

    // Parse ELF header
    let is_64bit = elf_data[4] == 2;
    let is_little_endian = elf_data[5] == 1;

    if !is_little_endian {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Only little-endian ELF files are supported",
        ));
    }

    // Get program header info
    let (ph_offset, ph_entsize, ph_num) = if is_64bit {
        if elf_data.len() < 56 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "ELF64 file too small",
            ));
        }
        let offset = u64::from_le_bytes(elf_data[32..40].try_into().unwrap()) as usize;
        let entsize = u16::from_le_bytes(elf_data[54..56].try_into().unwrap()) as usize;
        let num = u16::from_le_bytes(elf_data[56..58].try_into().unwrap()) as usize;
        (offset, entsize, num)
    } else {
        let offset = u32::from_le_bytes(elf_data[28..32].try_into().unwrap()) as usize;
        let entsize = u16::from_le_bytes(elf_data[42..44].try_into().unwrap()) as usize;
        let num = u16::from_le_bytes(elf_data[44..46].try_into().unwrap()) as usize;
        (offset, entsize, num)
    };

    let mut min_addr = u64::MAX;
    let mut max_addr = 0u64;

    // Find min/max addresses of PT_LOAD segments based on file size, not memory size
    // This skips uninitialized data (.bss sections)
    for i in 0..ph_num {
        let ph_off = ph_offset + i * ph_entsize;
        if ph_off + 8 >= elf_data.len() {
            break;
        }

        let p_type = u32::from_le_bytes(elf_data[ph_off..ph_off + 4].try_into().unwrap());

        // PT_LOAD = 1
        if p_type != 1 {
            continue;
        }

        let (p_vaddr, p_filesz) = if is_64bit {
            if ph_off + 56 > elf_data.len() {
                continue;
            }
            let vaddr = u64::from_le_bytes(elf_data[ph_off + 16..ph_off + 24].try_into().unwrap());
            let filesz = u64::from_le_bytes(elf_data[ph_off + 32..ph_off + 40].try_into().unwrap());
            (vaddr, filesz)
        } else {
            if ph_off + 32 > elf_data.len() {
                continue;
            }
            let vaddr =
                u32::from_le_bytes(elf_data[ph_off + 8..ph_off + 12].try_into().unwrap()) as u64;
            let filesz =
                u32::from_le_bytes(elf_data[ph_off + 16..ph_off + 20].try_into().unwrap()) as u64;
            (vaddr, filesz)
        };

        if p_filesz > 0 {
            min_addr = min_addr.min(p_vaddr);
            max_addr = max_addr.max(p_vaddr + p_filesz);
        }
    }

    if min_addr == u64::MAX {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No loadable segments found in ELF",
        ));
    }

    let total_size = (max_addr - min_addr) as usize;
    let mut binary = vec![0u8; total_size];

    // Load segments into binary
    for i in 0..ph_num {
        let ph_off = ph_offset + i * ph_entsize;
        if ph_off + 8 >= elf_data.len() {
            break;
        }

        let p_type = u32::from_le_bytes(elf_data[ph_off..ph_off + 4].try_into().unwrap());

        if p_type != 1 {
            continue;
        }

        let (p_offset, p_filesz, p_vaddr) = if is_64bit {
            if ph_off + 56 > elf_data.len() {
                continue;
            }
            let vaddr = u64::from_le_bytes(elf_data[ph_off + 16..ph_off + 24].try_into().unwrap());
            let offset =
                u64::from_le_bytes(elf_data[ph_off + 8..ph_off + 16].try_into().unwrap()) as usize;
            let filesz = u64::from_le_bytes(elf_data[ph_off + 32..ph_off + 40].try_into().unwrap());
            (offset, filesz, vaddr)
        } else {
            if ph_off + 32 > elf_data.len() {
                continue;
            }
            let offset =
                u32::from_le_bytes(elf_data[ph_off + 4..ph_off + 8].try_into().unwrap()) as usize;
            let vaddr =
                u32::from_le_bytes(elf_data[ph_off + 8..ph_off + 12].try_into().unwrap()) as u64;
            let filesz =
                u32::from_le_bytes(elf_data[ph_off + 16..ph_off + 20].try_into().unwrap()) as u64;
            (offset, filesz, vaddr)
        };

        if p_filesz > 0 && p_vaddr >= min_addr {
            let bin_offset = (p_vaddr - min_addr) as usize;
            let filesz = p_filesz as usize;
            if p_offset + filesz <= elf_data.len() && bin_offset + filesz <= binary.len() {
                binary[bin_offset..bin_offset + filesz]
                    .copy_from_slice(&elf_data[p_offset..p_offset + filesz]);
            }
        }
    }

    Ok(binary)
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: make-exe <input_elf_or_bin> <output_exe>");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    let mut input_file = File::open(input_path)?;
    let mut file_data = Vec::new();
    input_file.read_to_end(&mut file_data)?;

    // Check if this is an ELF file by checking for the ELF magic number
    let binary_data = if file_data.len() >= 4 && &file_data[0..4] == b"\x7FELF" {
        extract_binary_from_elf(&file_data)?
    } else {
        file_data
    };

    let binary_size = binary_data.len() as u32;

    // Read metadata from the first 6 bytes of the binary
    if binary_data.len() < 6 {
        eprintln!("Error: Binary too small to contain metadata");
        std::process::exit(1);
    }

    let _data_seg_offset = u16::from_le_bytes([binary_data[0], binary_data[1]]);
    let stack_seg_offset = u16::from_le_bytes([binary_data[2], binary_data[3]]);
    let stack_size = u16::from_le_bytes([binary_data[4], binary_data[5]]);

    // Header size is 32 bytes (2 paragraphs)
    let header_paragraphs = 2;
    let header_size = (header_paragraphs as u32) * 16;

    let total_size = binary_size + header_size;
    let pages = total_size.div_ceil(512) as u16;
    let extra_bytes = (total_size % 512) as u16;

    let header = MzHeader {
        signature: *b"MZ",
        extra_bytes,
        pages,
        relocations: 0,
        header_paragraphs,
        min_alloc: 0x1000, // 64KB additional for heap
        max_alloc: 0xFFFF,
        ss: stack_seg_offset,
        sp: stack_size,
        checksum: 0,
        ip: 0x0010, // Start at .text
        cs: 0x0000,
        reloc_table_offset: 0x001C,
        overlay_number: 0,
    };

    let mut output_file = File::create(output_path)?;

    let header_bytes: &[u8] = unsafe {
        std::slice::from_raw_parts(
            &header as *const MzHeader as *const u8,
            std::mem::size_of::<MzHeader>(),
        )
    };

    output_file.write_all(header_bytes)?;

    // If the header is smaller than our allocated paragraphs, pad it
    let padding = (header_size as usize).saturating_sub(header_bytes.len());
    if padding > 0 {
        output_file.write_all(&vec![0u8; padding])?;
    }

    output_file.write_all(&binary_data)?;

    println!("Created {} ({} bytes)", output_path, total_size);
    println!(
        "  Segments: CS:0000 DS:CS+{:04X} SS:CS+{:04X} SP:{:04X}",
        _data_seg_offset, stack_seg_offset, stack_size
    );

    Ok(())
}
