use std::fmt;

fn main() {}

struct Base64Sequence<'a>(&'a [u8]);

fn to_base64_char(n: u8) -> char {
    (match n {
        // A-Z
        0..=25 => 65 + n,
        // a-z
        26..=51 => 71 + n,
        // 0-9
        52..=61 => n - 4,
        // +
        62 => 43,
        // /
        63 => 47,
        _ => panic!("Unexpected {}", n),
    } as char)
}

impl<'a> Base64Sequence<'a> {
    fn to_base64_repr(&self) -> String {
        let mut bit_p: usize = 0;
        let mut last_bit: usize = 0;
        let mut buf: u16 = 0;
        let mut result = String::with_capacity(100);
        let mut peekable = self.0.iter().peekable();

        while let Some(&&byte) = peekable.peek() {
            if last_bit - bit_p < 6 {
                let remaining = last_bit - bit_p;
                let new_bits = (byte as u16) << 8 - remaining;
                buf = (buf << bit_p) | new_bits;
                last_bit = remaining + 8;
                bit_p = 0;
                peekable.next().unwrap();
            }
            let n = (buf << bit_p >> 10 & 0x3F) as u8;
            let c = to_base64_char(n);
            result.push(c);
            bit_p += 6;
            // println!("Popped {c} | Buf {:016b} | {} {}", buf, bit_p, last_bit);
        }
        if last_bit > bit_p {
            let n = (buf << bit_p >> 10 & 0x3F) as u8;
            let c = to_base64_char(n);
            result.push(c);
        }
        result
    }
}

impl<'a> fmt::Display for Base64Sequence<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_base64_repr())
    }
}

#[cfg(test)]
mod tests {
    use crate::Base64Sequence;
    use hex_literal::hex;

    #[test]
    fn hex_to_base64() {
        let hex_input = hex!(
            "49276d206b696c6c696e6720796f757220627261696e206c696b65206120706f69736f6e6f7573206d757368726f6f6d"
        );
        assert_eq!(
            format!("{}", Base64Sequence(hex_input.as_slice())),
            "SSdtIGtpbGxpbmcgeW91ciBicmFpbiBsaWtlIGEgcG9pc29ub3VzIG11c2hyb29t"
        );
    }
}
