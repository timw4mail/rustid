ENTRY(_start)

SECTIONS {
    . = 0x10000; /* DOS/32A loads executable objects starting at 0x10000 */
    
    .text : {
        *(.startup)
        *(.text .text.*)
    }
    
    . = ALIGN(4096);
    
    .rodata : { *(.rodata .rodata.*) }
    .data : { *(.data .data.*) }
    .bss : {
        *(.bss .bss.*) *(COMMON)
        . = ALIGN(16);
        _stack_bottom = .;
        . += 0x10000; /* 64KB stack */
        _stack_top = .;
        
        _heap = ALIGN(., 4);
    }
    
    /DISCARD/ : {
        *(.comment)
        *(.note*)
        *(.eh_frame)
        *(.eh_frame_hdr)
    }
}
