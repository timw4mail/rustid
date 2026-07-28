ENTRY(_start)

SECTIONS {
    . = 0x10000; /* DOS/32A typically loads executables at 0x10000 or higher */
    
    .text : {
        *(.startup)
        *(.text .text.*)
    }
    
    .rodata : { *(.rodata .rodata.*) }
    .data : { *(.data .data.*) }
    .bss : { *(.bss .bss.*) *(COMMON) }
    
    . = ALIGN(16);
    _stack_bottom = .;
    . += 0x4000; /* 16KB stack */
    _stack_top = .;
    
    _heap = ALIGN(., 4);
    
    /DISCARD/ : {
        *(.comment)
        *(.note*)
        *(.eh_frame)
        *(.eh_frame_hdr)
    }
}
