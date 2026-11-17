#define UART0 0x10000000
void uart_putc(char c) {
  volatile char *uart = (volatile *uart = c;
}
void uart_puts(const char *s) {
  while (*s) {
uart_putc(*s++);
  }
}
void kernel_main(void) {
  uart_puts("\n============================\n");
uart_puts("   MacroMag32 Boot...\n");
uart_puts("  Modular Architecture\n");
uart_puts("=========================\n");
while (1) {
// pętla powłoki tutaj idzie
}
}
