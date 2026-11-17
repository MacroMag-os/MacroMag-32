OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS {
    . = 0x80000000;
    .text : { *(.text._start) *(.text*) }
    .rodata : { *(.rodata*) }
    .data : { *(.data*) }
    .bss : { *(.bss*) }
    . = ALIGN(16);
    . += 0x10000; /* 64 KB stack (stack) */
    _stack_top = .;
}
