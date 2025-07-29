use std::{cmp::max, fmt};

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

fn fixed_xor(h1: &[u8], h2: &[u8], result: &mut [u8]) {
    let n = max(h1.len(), h2.len());
    for i in 0..n {
        result[i] = h1[i] ^ h2[i];
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

    #[test]
    fn fixed_xor_example() {
        let h1 = hex!("1c0111001f010100061a024b53535009181c");
        let h2 = hex!("686974207468652062756c6c277320657965");
        use std::cmp::max;
        let mut result: Vec<u8> = vec![0; max(h1.len(), h2.len())];
        crate::fixed_xor(&h1, &h2, &mut result);

        assert_eq!(result, hex!("746865206b696420646f6e277420706c6179"));
    }
}
