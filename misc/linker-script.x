STACK_SIZE = 0x8000;

SECTIONS
{
  /* Start address */
  . = 0x80000000;

  /* Output a text section, starting with the entry point */
  .text : ALIGN(0x4) {
    _start
    *(.text)
    *(.text.*)
  }

  /* Output the rodata */
  .rodata : ALIGN(0x8) {
    KEEP(*(__*))
    *(.rodata)
    *(.rodata.*)
  }

  /* Finally, all data                                         */
  /* NOTE: no need to page-align bss, both bss and data are RW */
  .data : ALIGN(0x8) {
    KEEP(*(__*))
    *(.data)
    *(.data.*)
  }
  .bss : {
    *(.bss)
    *(.bss.*)
  }


  /* Then we allocate some stack space */
  .stack : ALIGN(0x1000)
   {
      . = ALIGN(8);
      _stack_start = .;
      . = . + STACK_SIZE;
      . = ALIGN(8);
      _stack_end = .;
   }
}

