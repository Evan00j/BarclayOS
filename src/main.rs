#![no_main]
#![no_std]

use core::{any::Any, arch::asm, ffi::c_void, panic::PanicInfo, str::from_utf8_unchecked};

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

fn print(text: &str) {
    for ch in text.chars() {
        let _ = putchar(ch);
    }
}

fn println(text: &str) {
    print(text);
    let _ = putchar('\n');
}

trait Printable {
    fn stringify(&self) -> &str;
}

impl Printable for str {
    fn stringify(&self) -> &str {
        return self;
    }
}

impl Printable for u64 {
    fn stringify(&self) -> &str {
        const MAX_DIGITS: usize = 19;
        //create a static mut buffer to store the string since we do not have heap allocation
        static mut BUFFER: [u8; MAX_DIGITS] = [0; MAX_DIGITS];
        unsafe {
            let mut num = *self;
            let mut end = MAX_DIGITS;

            if num == 0 {
                BUFFER[MAX_DIGITS - 1] = b'0'; // Place '0' as the last digit
                return core::str::from_utf8_unchecked(&BUFFER[MAX_DIGITS - 1..MAX_DIGITS]);
            }

            while num > 0 {
                end -= 1;
                BUFFER[end] = b'0' + (num % 10) as u8;
                num /= 10;
            }

            core::str::from_utf8_unchecked(&BUFFER[end..MAX_DIGITS])
        }
    }
}

fn printf_butt_ugly(format_str: &str, values: &mut [&impl Printable]) {
    let mut i = 0;
    let mut format_chars = format_str.chars();

    loop {
        match format_chars.next() {
            None => break,
            Some(ch) => {
                if ch == '%' {
                    match format_chars.next() {
                        None => break,
                        Some(_) => {
                            let value = values[i];
                            let nice_val = value.stringify();
                            print(nice_val);
                            i += 1;
                        }
                    }
                } else {
                    let _ = putchar(ch);
                }
            }
        }
    }
}

macro_rules! printf {
    ($e:expr) => {{
        print($e);
    }};

    ($e:expr, $($es:expr),+) => {{
        printf! { $e }
        printf! { $($es),+ }
    }};
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
    printf!("thing", "thang", "thangin", "thung");
    //let input: &str = "test";
    let number: u64 = 12345;
    printf_butt_ugly("Put that %s away", &mut [&number]);
    println("peepee");
    println("poopoo");
    print("Hello, World!\n");

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
