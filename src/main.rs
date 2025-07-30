use std::fmt;

fn main() {}

#[allow(dead_code)]
struct Base64Sequence<'a>(&'a [u8]);

#[allow(dead_code)]
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
    #[allow(dead_code)]
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

#[allow(dead_code)]
fn fixed_xor(h1: &[u8], h2: &[u8], result: &mut [u8]) {
    assert!(h1.len() == h2.len(), "{} == {}", h1.len(), h2.len());
    assert!(h1.len() == result.len(), "{} == {}", h1.len(), result.len());
    for i in 0..h1.len() {
        result[i] = h1[i] ^ h2[i];
    }
}

fn xor_repeat_key(s: &String, key: &String) -> Vec<u8> {
    let mut cipher = vec![];
    let mut s_bytes = s.bytes().peekable();
    while let Some(_) = s_bytes.peek() {
        let mut k_bytes = key.bytes();
        while let (Some(&b1), Some(b2)) = (s_bytes.peek(), k_bytes.next()) {
            cipher.push(b1 ^ b2);
            s_bytes.next();
        }
    }
    cipher
}

#[allow(dead_code)]
// doesn't account for differing lengths
fn get_hamming_distance(b1: &[u8], b2: &[u8]) -> u32 {
    let mut result = vec![0; b1.len()];
    fixed_xor(b1, b2, &mut result);
    result.iter().map(|b| b.count_ones()).sum()
}

#[allow(dead_code)]
#[cfg(test)]
mod tests {
    use std::io::Read;
    use crate::{fixed_xor, get_hamming_distance, Base64Sequence};
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

        assert_eq!(hex::encode(result), "746865206b696420646f6e277420706c6179");
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
            hex::encode(result),
            "1b37373331363f78151b7f2b783431333d78397828372d363c78373e783a393b3736"
        );
    }

    fn maybe_valid_english(s: &String) -> bool {
        let mut num_spaces = 0;
        for c in s.chars() {
            if c == ' ' {
                num_spaces += 1;
            }
        }
        return num_spaces > 2;
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
        let file = fs::File::open("1_4.txt").unwrap();
        let mut reader = BufReader::new(file);
        let mut line_buf = String::with_capacity(1000);

        let mut amt_read = reader.read_line(&mut line_buf).unwrap();
        while amt_read > 0 {
            rm_newline(&mut line_buf);
            try_decode_ciphertxt_with_single_char(&line_buf);

            line_buf.clear();
            amt_read = reader.read_line(&mut line_buf).unwrap()
        }
        let plaintxt = "Now that the party is jumping\n".as_bytes();
        let mut result: Vec<u8> = vec![0; plaintxt.len()];
        fixed_xor(&plaintxt, &vec!['5' as u8; plaintxt.len()], &mut result);
        assert_eq!(
            hex::encode(result),
            "7b5a4215415d544115415d5015455447414c155c46155f4058455c5b523f"
        );
    }

    fn rm_newline(s: &mut String) {
        let last_char = s.chars().last().unwrap();
        if last_char == '\n' {
            s.truncate(s.len() - 1)
        }
    }

    #[test]
    fn with_repeating_key() {
        let msg = String::from(
            "Burning 'em, if you ain't quick and nimble\nI go crazy when I hear a cymbal",
        );
        let key = String::from("ICE");
        let expected = "0b3637272a2b2e63622c2e69692a23693a2a3c6324202d623d63343c2a26226324272765272a282b2f20430a652e2c652a3124333a653e2b2027630c692b20283165286326302e27282f";
        assert_eq!(hex::encode(crate::xor_repeat_key(&msg, &key)), expected);
    }

    #[test]
    fn test_hamming_distance() {
        assert_eq!(
            get_hamming_distance("this is a test".as_bytes(), "wokka wokka!!!".as_bytes()),
            37
        );
    }

    // script to poke at different key sizes
    fn p1_6_check_key_sizes() {
        use std::fs;
        use std::io::BufReader;
        let file = fs::File::open("1_6.txt").unwrap();
        let mut reader = BufReader::new(file);
        // Grabbing a chunk to get Hamming distance
        let mut chunk = [0; 200];
        reader.read(&mut chunk).unwrap();

        const MAX_KEY_SIZE: usize = 100;
        let mut scores = [(0, 0); MAX_KEY_SIZE];
        for key_size in 1..=MAX_KEY_SIZE {
            let normalized_hamming_distance =
                get_hamming_distance(&chunk[0..key_size], &chunk[key_size..key_size * 2]) * 1000
                    / key_size as u32;
            scores[key_size - 1] = (normalized_hamming_distance, key_size);
        }
        scores.sort();
        for (score, key_size) in scores {
            println!("(KEYSIZE {}) {}", key_size, score);
        }
    }

    #[test]
    fn decodes_base64_then_encrypted_repeating_key_msg() {
        use std::fs;
        use std::io::BufReader;
        let file = fs::File::open("1_6.txt").unwrap();
        let mut reader = BufReader::new(file);
        // Trying key sizes showed 3, 2, 7 to be promising.
        const KEY_SIZE: usize = 3;

        let mut chunks = vec!();
        let mut chunk = vec![0; KEY_SIZE];

        // Break txt into chunks
        let mut amt_read = 1;
        while amt_read > 0 {
            amt_read = reader.read(&mut chunk).unwrap();
            chunks.push(chunk.clone());
            chunk.fill(0);
        }

        println!("{}", chunks.len());
        // transpose
        for i in 0..KEY_SIZE {
            let mut block = vec![0; chunks.len()];
            for (j, chunk) in chunks.iter().enumerate() {
                block[j] = chunk[i]; 
            }
            println!("{}", chunks.len());
            try_decode_ciphertxt_with_single_char(&String::from_utf8(block).unwrap());
        }
    }
}
