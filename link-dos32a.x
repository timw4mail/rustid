ENTRY(_start)

SECTIONS {
    /* In protected mode, we can start anywhere, but we use a dummy address for relocations */
    . = 0x1000;

    .text : {
        *(.startup)
        *(.text .text.*)
    }

    .rodata : { *(.rodata .rodata.*) }
    .data : { *(.data .data.*) }
    .bss : { *(.bss .bss.*) *(COMMON) }
    
    . = ALIGN(16);
    _stack_bottom = .;
    . += 0x2000; /* 8KB stack */
    _stack_top = .;

    __binary_end = .;
}
