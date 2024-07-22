SECTIONS
{
  /* Start address */
  /* The address is read from the configuration file and passed to the linker */
  . = _start_address;

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
  . = ALIGN(0x8);
  _bss_start = .;
  .bss : {
    *(.bss)
    *(.bss.*)
  }
  _bss_stop = .;

  /* Then we mark the start of the stack (or the end, as the stack grows
   * downard). */
  . = ALIGN(0x1000);
  _stack_start = .;
}

