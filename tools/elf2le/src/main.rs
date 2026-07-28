use std::env;
use std::fs::File;
use std::io::{Read, Result, Write};

#[repr(C, packed)]
struct LxHeader {
    signature: [u8; 2],      // "LX" or "LE"
    linker_version: [u8; 2], // Linker version
    module_type: [u8; 2],    // Module type (0x0001 = executable)
    min_alloc_size: [u8; 4], // Min/Max memory addresses
    ip: [u8; 2],             // Initial IP
    cs_ip: [u8; 4],          // Initial CS:IP
    ss_sp: [u8; 4],          // Initial SS:SP
    stack_alloc: [u8; 4],    // Initial Stack Size
    checksum: [u8; 4],       // Checksum
    object_flags: [u8; 2],   // Object file flags
    pages_count: [u8; 2],    // Number of pages in the file
    table_offset: [u8; 4],   // Offset to table directory
    table_size: [u8; 4],     // Size of table directory
}

#[repr(C, packed)]
struct TableEntry {
    table_type: u16,   // Table type
    table_offset: u32, // Table offset
    table_size: u16,   // Table size
}

const TABLE_TYPE_CODE: u16 = 0x0001;
const TABLE_TYPE_DATA: u16 = 0x0002;
const TABLE_TYPE_FIXUP: u16 = 0x0004;

fn extract_binary_from_elf(elf_data: &[u8]) -> Result<(Vec<u8>, u32, u32, u32, u32)> {
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

    let mut min_addr = u32::MAX;
    let mut max_addr = 0u32;
    let mut text_offset = 0u32;
    let mut data_offset = 0u32;
    let mut bss_size = 0u32;

    // Find min/max addresses of PT_LOAD segments
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

        let (p_vaddr, p_filesz, p_memsz, p_offset) = if is_64bit {
            if ph_off + 56 > elf_data.len() {
                continue;
            }
            let vaddr = u32::from_le_bytes(elf_data[ph_off + 16..ph_off + 24].try_into().unwrap());
            let filesz = u32::from_le_bytes(elf_data[ph_off + 32..ph_off + 40].try_into().unwrap());
            let memsz = u32::from_le_bytes(elf_data[ph_off + 40..ph_off + 48].try_into().unwrap());
            let offset = u32::from_le_bytes(elf_data[ph_off + 8..ph_off + 16].try_into().unwrap());
            (vaddr, filesz, memsz, offset)
        } else {
            if ph_off + 32 > elf_data.len() {
                continue;
            }
            let vaddr = u32::from_le_bytes(elf_data[ph_off + 8..ph_off + 12].try_into().unwrap());
            let filesz = u32::from_le_bytes(elf_data[ph_off + 16..ph_off + 20].try_into().unwrap());
            let memsz = u32::from_le_bytes(elf_data[ph_off + 20..ph_off + 24].try_into().unwrap());
            let offset = u32::from_le_bytes(elf_data[ph_off + 4..ph_off + 8].try_into().unwrap());
            (vaddr, filesz, memsz, offset)
        };

        if p_filesz > 0 {
            min_addr = min_addr.min(p_vaddr);
            let end_addr = p_vaddr.checked_add(p_filesz).unwrap_or(u32::MAX);
            max_addr = max_addr.max(end_addr);

            if p_vaddr == 0x10000 || (p_vaddr >= 0x10000 && p_vaddr < 0x20000) {
                text_offset = p_offset;
            } else if p_vaddr >= 0x20000 {
                data_offset = p_offset;
            }
        }

        if p_memsz > p_filesz {
            bss_size = p_memsz - p_filesz;
        }
    }

    if min_addr == u32::MAX {
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
            let vaddr = u32::from_le_bytes(elf_data[ph_off + 16..ph_off + 24].try_into().unwrap());
            let offset = u32::from_le_bytes(elf_data[ph_off + 8..ph_off + 16].try_into().unwrap());
            let filesz = u32::from_le_bytes(elf_data[ph_off + 32..ph_off + 40].try_into().unwrap());
            (offset, filesz, vaddr)
        } else {
            if ph_off + 32 > elf_data.len() {
                continue;
            }
            let offset = u32::from_le_bytes(elf_data[ph_off + 4..ph_off + 8].try_into().unwrap());
            let vaddr = u32::from_le_bytes(elf_data[ph_off + 8..ph_off + 12].try_into().unwrap());
            let filesz = u32::from_le_bytes(elf_data[ph_off + 16..ph_off + 20].try_into().unwrap());
            (offset, filesz, vaddr)
        };

        if p_filesz > 0 && p_vaddr >= min_addr {
            let bin_offset = (p_vaddr - min_addr) as usize;
            let filesz = p_filesz as usize;
            if (p_offset as usize) + filesz <= elf_data.len() && bin_offset + filesz <= binary.len()
            {
                binary[bin_offset..bin_offset + filesz]
                    .copy_from_slice(&elf_data[(p_offset as usize)..(p_offset as usize) + filesz]);
            }
        }
    }

    // Adjust addresses for DOS/32A (load at 0x10000)
    let load_base = 0x10000u32;
    let adjusted_min_addr = min_addr.saturating_sub(load_base);

    // Calculate stack and heap info
    let stack_size = 0x4000u32; // 16KB stack
    let heap_start = max_addr.max(0x10000 + total_size as u32 + 0x4000);
    let heap_size = 0x100000u32; // 1MB heap

    Ok((binary, adjusted_min_addr, stack_size, heap_start, heap_size))
}

fn create_lx_header(
    binary_size: u32,
    min_addr: u32,
    stack_size: u32,
    heap_start: u32,
    heap_size: u32,
) -> (Vec<u8>, Vec<u8>) {
    let header_size = 32u32;
    let table_dir_size = 3 * 8u32; // 3 table entries (code, data, fixup)
    let total_size = header_size + table_dir_size + binary_size;

    let pages_count = total_size.div_ceil(512) as u16;

    // LX Header
    let mut lx_header = Vec::with_capacity(32);

    // Signature: "LX"
    lx_header.extend_from_slice(b"LX");

    // Linker version: 0x0300
    lx_header.extend_from_slice(&0x0300u16.to_le_bytes());

    // Module type: 0x0001 (executable)
    lx_header.extend_from_slice(&0x0001u16.to_le_bytes());

    // Min/Max memory addresses
    let min_addr_le = (0x10000u32).to_le_bytes();
    let max_addr_le = (heap_start.saturating_add(heap_size)).to_le_bytes();
    lx_header.extend_from_slice(&min_addr_le);
    lx_header.extend_from_slice(&max_addr_le);

    // Initial IP: 0x0000 (relative to code segment)
    lx_header.extend_from_slice(&0x0000u16.to_le_bytes());

    // Initial CS:IP: CS=0, IP=0
    let cs_ip = 0u32;
    lx_header.extend_from_slice(&cs_ip.to_le_bytes());

    // Initial SS:SP: SS=0, SP=stack_size
    let ss_sp = (0u32 << 16) | (stack_size & 0xFFFF);
    lx_header.extend_from_slice(&ss_sp.to_le_bytes());

    // Initial Stack Size
    lx_header.extend_from_slice(&stack_size.to_le_bytes());

    // Checksum: 0
    lx_header.extend_from_slice(&0u32.to_le_bytes());

    // Object file flags: 0x0001 (executable)
    lx_header.extend_from_slice(&0x0001u16.to_le_bytes());

    // Pages count
    lx_header.extend_from_slice(&pages_count.to_le_bytes());

    // Table offset: 32 (after LX header)
    let table_offset = 32u32;
    lx_header.extend_from_slice(&table_offset.to_le_bytes());

    // Table size: 3 * 8 = 24 bytes
    let table_size = 24u32;
    lx_header.extend_from_slice(&table_size.to_le_bytes());

    // Table directory
    let mut table_dir = Vec::with_capacity(24);

    // Code table
    let code_table = TableEntry {
        table_type: TABLE_TYPE_CODE,
        table_offset: 32 + 24,                // After header and table dir
        table_size: (binary_size / 2) as u16, // Code size in paragraphs
    };
    table_dir.extend_from_slice(&code_table.table_type.to_le_bytes());
    table_dir.extend_from_slice(&code_table.table_offset.to_le_bytes());
    table_dir.extend_from_slice(&code_table.table_size.to_le_bytes());

    // Data table
    let data_table = TableEntry {
        table_type: TABLE_TYPE_DATA,
        table_offset: 32 + 24 + (binary_size / 2) as u32,
        table_size: (binary_size / 2) as u16,
    };
    table_dir.extend_from_slice(&data_table.table_type.to_le_bytes());
    table_dir.extend_from_slice(&data_table.table_offset.to_le_bytes());
    table_dir.extend_from_slice(&data_table.table_size.to_le_bytes());

    // Fixup table (empty)
    let fixup_table = TableEntry {
        table_type: TABLE_TYPE_FIXUP,
        table_offset: 0,
        table_size: 0,
    };
    table_dir.extend_from_slice(&fixup_table.table_type.to_le_bytes());
    table_dir.extend_from_slice(&fixup_table.table_offset.to_le_bytes());
    table_dir.extend_from_slice(&fixup_table.table_size.to_le_bytes());

    (lx_header, table_dir)
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: elf2le <input_elf_or_bin> <output_lx>");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    let mut input_file = File::open(input_path)?;
    let mut file_data = Vec::new();
    input_file.read_to_end(&mut file_data)?;

    // Check if this is an ELF file by checking for the ELF magic number
    let (binary_data, min_addr, stack_size, heap_start, heap_size) =
        if file_data.len() >= 4 && &file_data[0..4] == b"\x7FELF" {
            extract_binary_from_elf(&file_data)?
        } else {
            // Assume flat binary
            let size = file_data.len() as u32;
            (file_data, 0x10000, 0x4000, 0x20000, 0x100000)
        };

    let (lx_header, table_dir) = create_lx_header(
        binary_data.len() as u32,
        min_addr,
        stack_size,
        heap_start,
        heap_size,
    );

    let mut output_file = File::create(output_path)?;

    output_file.write_all(&lx_header)?;
    output_file.write_all(&table_dir)?;
    output_file.write_all(&binary_data)?;

    let total_size = lx_header.len() + table_dir.len() + binary_data.len();
    println!("Created {} ({} bytes)", output_path, total_size);
    println!("  Load base: 0x{:08X}", 0x10000);
    println!("  Stack size: 0x{:04X}", stack_size);
    println!("  Heap start: 0x{:08X}", heap_start);

    Ok(())
}
