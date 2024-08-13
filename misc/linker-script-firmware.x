STACK_SIZE = 0x8000;

SECTIONS
{
  /* Start address */
  . = _firmware_address;

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
  .sdata : ALIGN(0x8) {
    KEEP(*(__*))
    *(.sdata)
    *(.sdata.*)
  }
  . = ALIGN(0x8);
  _firmware_bss_start = .;
  .sbss : {
    *(.sbss)
    *(.sbss.*)
  }
  .bss : ALIGN(0x8) {
    *(.bss)
    *(.bss.*)
  }
  _firmware_bss_stop = .;


  /* Then we allocate some stack space */
  .stack : ALIGN(0x1000)
   {
      . = ALIGN(8);
      _stack_bottom = .;
      . = . + STACK_SIZE;
      . = ALIGN(8);
      _stack_top = .;
   }
}

