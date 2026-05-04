ENTRY(_start)

SECTIONS {
    . = 0x0;
    /* Metadata at the very beginning of the binary */
    .metadata : {
        SHORT(ABSOLUTE(__data_seg_offset))
        SHORT(ABSOLUTE(__stack_seg_offset))
        SHORT(ABSOLUTE(__stack_size))
    }

    .text 0x10 : {
        *(.startup)
        *(.text .text.*)
    }

    .rodata : { *(.rodata .rodata.*) }
    .data : { *(.data .data.*) }
    .bss : { *(.bss .bss.*) *(COMMON) }
    
    . = ALIGN(16);
    _stack_bottom = .;
    . += 0x2000;
    _stack_top = .;
    
    __binary_end = .;
    _heap = ALIGN(., 4);

    /* Unreal Mode GDT - placed at known address for inline assembly reference */
    .unreal_gdt : AT(0x1000) {
        *(.unreal_gdt)
    }

    /* Flat model metadata */
    __data_seg_offset = 0;
    __stack_seg_offset = 0;
    __stack_size = _stack_top;

    /DISCARD/ : {
        *(.comment)
        *(.note*)
        *(.eh_frame)
        *(.eh_frame_hdr)
        *(.rel.plt)
    }
}
