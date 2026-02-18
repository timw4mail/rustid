ENTRY(_start)

SECTIONS {
    . = 0x100;
    
    .text : {
        *(.startup)
        *(.text .text.*)
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

    .data : {
        *(.data .data.*)
    }

    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    }

    /* Stack at the end of the segment */
    .stack : {
        . = ALIGN(16);
        _stack_bottom = .;
        . += 0x1000;
        _stack_top = .;
    }

    _heap = ALIGN(4);
    
    /DISCARD/ : {
        *(.comment)
        *(.note*)
        *(.eh_frame)
        *(.rel.plt)
    }
}
