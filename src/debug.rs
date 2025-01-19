use crate::sbi;

pub fn putchar(ch: char) {
    let _ = sbi::ecall(ch as u64, 0, 0, 0, 0, 0, 0, 1);
}

pub fn print(text: &str) {
    for ch in text.chars() {
        let _ = sbi::ecall(ch as u64, 0, 0, 0, 0, 0, 0, 1);
    }
}

pub fn print_hex(val: u64) {
    putchar('0');
    putchar('x');

    for i in (0..16).rev() {
        let mut num = ((val >> (4 * i)) & 0xf) as u8;
        num += if num < 0xa { 0x30 } else { 0x57 };
        putchar(num as char);
    }
}
