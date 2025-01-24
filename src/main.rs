#![no_main]
#![no_std]

extern crate core;
use core::{arch::asm, fmt::Display, panic::PanicInfo};

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::{boxed::Box, vec, vec::Vec};

mod debug;
mod riker;
mod sbi;

extern "C" {
    static mut __bss: u8;
    static mut __bss_end: u8;
    static mut __stack_top: u8;
}

fn putchar(ch: char) {
    let _ = sbi::ecall(ch as u64, 0, 0, 0, 0, 0, 0, 1);
}

fn print(text: String) {
    for ch in text.chars() {
        putchar(ch);
    }
}

fn println(text: String) {
    print(text);
    putchar('\n');
}

fn _printf(format: &str, vals: Vec<Box<dyn Display>>) {
    let mut format_it = format.chars();
    let mut vals_it = vals.iter();

    while let Some(ch) = format_it.next() {
        if ch == '%' {
            match format_it.next() {
                None | Some('%') => putchar('%'),
                Some(next_ch) => {
                    print(vals_it.next().map_or("<?>".to_string(), |a| a.to_string()));
                    putchar(next_ch);
                }
            }
        } else {
            putchar(ch);
        }
    }
}

macro_rules! printf {
    ($fmt:literal, $($es:expr),*) => {{
        _printf($fmt, vec![$(Box::new($es)),*]);
    }};
}

fn memset<T>(buf: *mut T, c: u8, n: usize) {
    let p = buf as *mut u8;

    unsafe {
        for i in 0..n {
            *p.add(i) = c;
        }
    }
}

fn kernel_main() -> ! {
    printf!("Heap space before: %\n", riker::ALLOC.remaining());
    printf!("Hello, % and %! % %\n", "Evan", "Luke", 123);
    printf!("Heap space after: %\n", riker::ALLOC.remaining());

    unsafe {
        asm!("wfi");
    }

    loop {}
}

#[no_mangle]
#[link_section = ".text.boot"]
pub extern "C" fn _start() -> ! {
    unsafe {
        // Initialize the stack pointer
        asm!("mv sp, {stack_top}", stack_top = in(reg) &raw const __stack_top);

        // Initialize .bss to zeros
        let u8_count = (&raw const __bss_end).offset_from(&raw const __bss);
        memset(&raw mut __bss, 0, u8_count as usize);
    }

    kernel_main();
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn kernel_entry() {
    unsafe {
        asm!(
            // Save registers to the stack
            "csrw sscratch, sp",
            "addi sp, sp, -4 * 31",
            "sw ra,  4 * 0(sp)",
            "sw gp,  4 * 1(sp)",
            "sw tp,  4 * 2(sp)",
            "sw t0,  4 * 3(sp)",
            "sw t1,  4 * 4(sp)",
            "sw t2,  4 * 5(sp)",
            "sw t3,  4 * 6(sp)",
            "sw t4,  4 * 7(sp)",
            "sw t5,  4 * 8(sp)",
            "sw t6,  4 * 9(sp)",
            "sw a0,  4 * 10(sp)",
            "sw a1,  4 * 11(sp)",
            "sw a2,  4 * 12(sp)",
            "sw a3,  4 * 13(sp)",
            "sw a4,  4 * 14(sp)",
            "sw a5,  4 * 15(sp)",
            "sw a6,  4 * 16(sp)",
            "sw a7,  4 * 17(sp)",
            "sw s0,  4 * 18(sp)",
            "sw s1,  4 * 19(sp)",
            "sw s2,  4 * 20(sp)",
            "sw s3,  4 * 21(sp)",
            "sw s4,  4 * 22(sp)",
            "sw s5,  4 * 23(sp)",
            "sw s6,  4 * 24(sp)",
            "sw s7,  4 * 25(sp)",
            "sw s8,  4 * 26(sp)",
            "sw s9,  4 * 27(sp)",
            "sw s10, 4 * 28(sp)",
            "sw s11, 4 * 29(sp)",
            // Save the old stack pointer
            "csrr a0, sscratch",
            "sw a0, 4 * 30(sp)",
            // Call the trap handler
            "mv a0, sp",
            "call handle_trap",
            // Restore registers from the stack
            "lw ra,  4 * 0(sp)",
            "lw gp,  4 * 1(sp)",
            "lw tp,  4 * 2(sp)",
            "lw t0,  4 * 3(sp)",
            "lw t1,  4 * 4(sp)",
            "lw t2,  4 * 5(sp)",
            "lw t3,  4 * 6(sp)",
            "lw t4,  4 * 7(sp)",
            "lw t5,  4 * 8(sp)",
            "lw t6,  4 * 9(sp)",
            "lw a0,  4 * 10(sp)",
            "lw a1,  4 * 11(sp)",
            "lw a2,  4 * 12(sp)",
            "lw a3,  4 * 13(sp)",
            "lw a4,  4 * 14(sp)",
            "lw a5,  4 * 15(sp)",
            "lw a6,  4 * 16(sp)",
            "lw a7,  4 * 17(sp)",
            "lw s0,  4 * 18(sp)",
            "lw s1,  4 * 19(sp)",
            "lw s2,  4 * 20(sp)",
            "lw s3,  4 * 21(sp)",
            "lw s4,  4 * 22(sp)",
            "lw s5,  4 * 23(sp)",
            "lw s6,  4 * 24(sp)",
            "lw s7,  4 * 25(sp)",
            "lw s8,  4 * 26(sp)",
            "lw s9,  4 * 27(sp)",
            "lw s10, 4 * 28(sp)",
            "lw s11, 4 * 29(sp)",
            "lw sp,  4 * 30(sp)",
            // Return from the trap
            "sret",
        );
    }
}
