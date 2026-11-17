#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

global_asm!(
    ".section .text._start",
    ".globl _start",
    "_start:",
    "    la sp, _stack_top",
    "    call kmain",
    "1:  wfi",
    "    j 1b"
);

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    let uart = 0x1000_0000 as *mut u8;
    let msg = b"MacroMag Microkernel v0.1 (RISC-V 64) booted!\n";
    for &byte in msg {
        unsafe { uart.write_volatile(byte); }
    }
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
