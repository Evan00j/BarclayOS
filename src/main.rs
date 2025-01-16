#![no_main]
#![no_std]

use core::{arch::asm, ffi::c_void, panic::PanicInfo};

extern "C" {
    static mut __bss: u8;
    static mut __bss_end: u8;
    static mut __stack_top: u8;
}

fn sbi_call(
    arg0: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
    fid: u64,
    eid: u64,
) -> Result<u64, u64> {
    let mut a0 = arg0;
    let mut a1 = arg1;

    unsafe {
        asm!("ecall", inlateout("a0") a0, inlateout("a1") a1, in("a2") arg2, in("a3") arg3, in("a4") arg4, in("a5") arg5, in("a6") fid, in("a7") eid);

        if a0 == 0 {
            return Ok(a1);
        }

        Err(a0)
    }
}

fn putchar(ch: char) -> Result<u64, u64> {
    sbi_call(ch as u64, 0, 0, 0, 0, 0, 0, 1)
}

fn memset(buf: *mut c_void, c: u8, n: usize) {
    let p = buf as *mut u8;

    unsafe {
        for i in 0..n {
            *p.add(i) = c;
        }
    }
}

fn kernel_main() -> ! {
    for ch in "Hello, World!\n".chars() {
        let _ = putchar(ch);
    }

    unsafe {
        asm!("wfi");
    }

    loop {}
}

#[no_mangle]
#[link_section = ".text.boot"]
pub extern "C" fn _start() -> ! {
    unsafe {
        asm!("mv sp, {stack_top}", stack_top = in(reg) &raw const __stack_top);
        let u8_count = (&raw const __bss_end).offset_from(&raw const __bss);
        memset(__bss as *mut c_void, 0, u8_count as usize);
    }

    kernel_main();
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}
