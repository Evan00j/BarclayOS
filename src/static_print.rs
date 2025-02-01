// Courtesy of Evan "Sunglasses Emoji" Jenkins

pub trait Printable {
    fn stringify(&self) -> &str;
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
