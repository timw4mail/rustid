use std::env;
use std::fs::File;
use std::io::{Read, Write, Result};

#[repr(C, packed)]
struct MzHeader {
    signature: [u8; 2],     // 'MZ'
    extra_bytes: u16,       // Bytes on last page
    pages: u16,             // 512-byte pages in file
    relocations: u16,       // Number of relocation entries
    header_paragraphs: u16, // Header size in 16-byte paragraphs
    min_alloc: u16,         // Minimum extra paragraphs needed
    max_alloc: u16,         // Maximum extra paragraphs needed
    ss: u16,                // Initial relative SS
    sp: u16,                // Initial SP
    checksum: u16,          // Checksum (usually 0)
    ip: u16,                // Initial IP
    cs: u16,                // Initial relative CS
    reloc_table_offset: u16,// Offset to relocation table
    overlay_number: u16,    // Overlay number
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: make-exe <input_bin> <output_exe>");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    let mut input_file = File::open(input_path)?;
    let mut binary_data = Vec::new();
    input_file.read_to_end(&mut binary_data)?;

    let binary_size = binary_data.len() as u32;
    
    // Header size is 32 bytes (2 paragraphs)
    let header_paragraphs = 2;
    let header_size = (header_paragraphs as u32) * 16;
    
    let total_size = binary_size + header_size;
    let pages = ((total_size + 511) / 512) as u16;
    let extra_bytes = (total_size % 512) as u16;

    let header = MzHeader {
        signature: *b"MZ",
        extra_bytes,
        pages,
        relocations: 0,
        header_paragraphs,
        min_alloc: 0x1000, // 64KB additional for heap/stack
        max_alloc: 0xFFFF,
        ss: 0x0000,
        sp: 0xFFFE, // Top of 64K segment
        checksum: 0,
        ip: 0x0010,
        cs: 0x0000,
        reloc_table_offset: 0x001C,
        overlay_number: 0,
    };

    let mut output_file = File::create(output_path)?;
    
    // Safety check for alignment/padding (though repr(C, packed) should handle it)
    let header_bytes: &[u8] = unsafe {
        std::slice::from_raw_parts(
            &header as *const MzHeader as *const u8,
            std::mem::size_of::<MzHeader>()
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

    Ok(())
}
