#![no_main]
#![no_std]

extern crate core;
use core::{arch::asm, fmt::Display, panic::PanicInfo};

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::{boxed::Box, vec, vec::Vec};
use static_print::Printable;

mod debug;
mod riker;
mod sbi;
mod static_print;

extern "C" {
    static mut __bss: u8;
    static mut __bss_end: u8;
    static mut __stack_top: u8;
}

fn print(text: String) {
    for ch in text.chars() {
        debug::putchar(ch);
    }
}

fn println(text: String) {
    print(text);
    debug::putchar('\n');
}

fn _printf(format: &str, vals: Vec<Box<dyn Display>>) {
    let mut format_it = format.chars();
    let mut vals_it = vals.iter();

    while let Some(ch) = format_it.next() {
        if ch == '%' {
            match format_it.next() {
                None | Some('%') => debug::putchar('%'),
                Some(next_ch) => {
                    print(vals_it.next().map_or("<?>".to_string(), |a| a.to_string()));
                    debug::putchar(next_ch);
                }
            }
        } else {
            debug::putchar(ch);
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
    debug::print(123u64.stringify());
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
