use std::fmt;
use std::vec::Vec;

pub struct Base64Sequence<'a>(&'a [u8]);

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

fn from_base64_char(c: char) -> u8 {
    match c as u8 {
        // A-Z
        65..=90 => c as u8 - 65,
        // a-z
        97..=123 => c as u8 - 71,
        // 0-9
        48..=57 => c as u8 + 4,
        // +
        43 => 62,
        // /
        47 => 63,
        _ => panic!("Unexpected {}", c),
    }
}

pub fn decode(encoded: &str) -> Vec<u8> {
    let mut result = Vec::new();
    let chars: Vec<char> = encoded.chars().filter(|&c| c != '=').collect();

    for chunk in chars.chunks(4) {
        let mut buf: u32 = 0;

        for (i, &c) in chunk.iter().enumerate() {
            buf |= (from_base64_char(c) as u32) << (18 - i * 6);
        }

        if chunk.len() > 1 {
            result.push((buf >> 16) as u8);
        }
        if chunk.len() > 2 {
            result.push((buf >> 8) as u8);
        }
        if chunk.len() > 3 {
            result.push(buf as u8);
        }
    }

    result
}

pub fn encode(bytes: &[u8]) -> String {
    let mut bit_p: usize = 0;
    let mut last_bit: usize = 0;
    let mut buf: u16 = 0;
    let mut result = String::with_capacity(100);
    let mut peekable = bytes.iter().peekable();

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
    }
    if last_bit > bit_p {
        let n = (buf << bit_p >> 10 & 0x3F) as u8;
        let c = to_base64_char(n);
        result.push(c);
    }
    result
}

impl<'a> Base64Sequence<'a> {
    pub fn new(b: &'a [u8]) -> Self {
        Base64Sequence(b)
    }
}

impl<'a> fmt::Display for Base64Sequence<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", encode(self.0))
    }
}
