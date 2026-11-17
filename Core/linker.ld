OUTPUT_ARCH( "riscv" )
ENTRY( _start )

SECTIONS
{
  /* Standardowy adres startowy pamięci RAM w RISC-V QEMU */
  . = 0x80000000;

  .text : {
    *(.text.init)
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
  }

  /* Rezerwacja 64 KB na stos jądra */
  . = ALIGN(16);
  . += 0x10000;
  _stack_top = .;
}
