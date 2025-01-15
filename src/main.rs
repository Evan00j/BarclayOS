#![no_main]
#![no_std]

use core::arch::asm;
use core::ffi::c_void;
use core::panic::PanicInfo;

extern "C" {
    static __bss: *mut u8;
    static __bss_end: *mut u8;
    static __stack_top: *mut u8;
}

fn memset(buf: *mut c_void, c: u8, n: usize) {
    let mut p: *mut u8 = buf as *mut u8;
    let mut z = n;

    while z > 0 {
        unsafe {
            *p = c;
            p = p.add(1);
        }

        z -= 1;
    }
}

fn kernel_main() {
    unsafe {
        let number_of_u8s = __bss_end.offset_from(__bss);
        memset(__bss as *mut c_void, 0, number_of_u8s as usize);
    }

    loop {}
}

#[no_mangle]
#[link_section = ".text.boot"]
pub extern "C" fn _start() -> ! {
    unsafe {
        asm!("mv sp, {stack_top}", stack_top = in(reg) __stack_top);
        asm!("j kernel_main");
    }

    loop {}
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}
