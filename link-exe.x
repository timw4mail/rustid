ENTRY(_start)

SECTIONS {
    . = 0x0;

    /* Header for segment offsets */
    .metadata : {
        SHORT(__data_seg_offset)
        SHORT(__bss_seg_offset)
        SHORT(__stack_seg_offset)
    }

    . = ALIGN(16);
    .text : {
        *(.startup)
        *(.text .text.*)
    }

    . = ALIGN(16);
    __data_start = .;
    __data_seg_offset = __data_start >> 4;

    .rodata : {
        *(.rodata .rodata.*)
    }

    .data : {
        *(.data .data.*)
    }

    . = ALIGN(16);
    __bss_start = .;
    __bss_seg_offset = __bss_start >> 4;
    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    }

    . = ALIGN(16);
    __stack_start = .;
    __stack_seg_offset = __stack_start >> 4;
    /* Stack at the end */
    .stack : {
        _stack_bottom = .;
        . += 0x2000;
        _stack_top = .;
    }

    . = ALIGN(4);
    _heap = .;
    
    /DISCARD/ : {
        *(.comment)
        *(.note*)
        *(.eh_frame)
        *(.eh_frame_hdr)
        *(.rel.plt)
    }
}
