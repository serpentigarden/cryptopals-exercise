#[allow(dead_code)]
use std::{cmp::max, fmt};

fn main() {}

fn print_hex(v: &Vec<u8>) {
    for b in v {
        print!("{:02x}", b);
    }
    println!();
}

fn print_chars(v: &Vec<u8>) {
    for &b in v {
        print!("{}", b as char);
    }
    println!();
}

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
    assert!(h1.len() == h2.len());
    for i in 0..h1.len() {
        result[i] = h1[i] ^ h2[i];
    }
}

#[cfg(test)]
mod tests {
    use crate::{Base64Sequence, fixed_xor, print_chars};
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
        fixed_xor(&h1, &h2, &mut result);

        assert_eq!(result, hex!("746865206b696420646f6e277420706c6179"));
    }

    #[test]
    fn decode_ciphertxt() {
        let ciphertxt =
            String::from("1b37373331363f78151b7f2b783431333d78397828372d363c78373e783a393b3736");
        try_decode_ciphertxt_with_single_char(&ciphertxt);

        let plaintxt = "Cooking MC's like a pound of bacon".as_bytes();
        let mut result: Vec<u8> = vec![0; plaintxt.len()];
        fixed_xor(&plaintxt, &vec!['X' as u8; plaintxt.len()], &mut result);
        assert_eq!(
            result,
            hex!("1b37373331363f78151b7f2b783431333d78397828372d363c78373e783a393b3736")
        );
    }

    fn maybe_valid_english(s: &String) -> bool {
        for c in s.chars() {
            if c == ' ' {
                return true;
            }
        }
        false
    }

    fn try_decode_ciphertxt_with_single_char(hex_cipher: &String) {
        let mut candidate_mask: Vec<u8> = vec![0; hex_cipher.len() / 2];
        for i in 0..128 {
            let mut xor_result: Vec<u8> = vec![0; hex_cipher.len() / 2];
            candidate_mask.fill(i);

            fixed_xor(
                &hex::decode(hex_cipher).unwrap(),
                &candidate_mask,
                &mut xor_result,
            );
            match String::from_utf8(xor_result) {
                Ok(s) => {
                    if maybe_valid_english(&s) {
                        println!("{} ^ [{} ({})] => {}", hex_cipher, i, i as char, s);
                    }
                }
                Err(_) => {
                    // println!("{} ^ [{} ({})] => invalid utf8", hex_cipher, i, i as char, s);
                }
            };
        }
    }

    #[test]
    fn decode_many_ciphertxts() {
        use std::fs;
        use std::io::{BufRead, BufReader};

        let file = fs::File::open("edited_set1_challenge4.txt").unwrap();
        let mut reader = BufReader::new(file);
        let mut line_buf = String::with_capacity(1000);

        let mut amt_read = reader.read_line(&mut line_buf).unwrap();
        while amt_read > 0 {
            let last_char = line_buf.chars().last().unwrap();
            let size = if last_char == '\n' {
                amt_read - 1
            } else {
                amt_read
            };
            line_buf.truncate(size);
            try_decode_ciphertxt_with_single_char(&line_buf);

            line_buf.clear();
            amt_read = reader.read_line(&mut line_buf).unwrap()
        }
    }
}
