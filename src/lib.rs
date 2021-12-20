use itertools::{EitherOrBoth, Itertools};
use std::iter;

// Convert an u8 to a little-endian Vec representation of a binary number
pub fn byte_to_bits(byte: u8) -> Vec<u8> {
    let mut bits = Vec::new();
    let mut byte = byte;
    for _ in 0..8 {
        bits.insert(0, byte % 2);
        byte = byte / 2;
    }
    bits
}

// Convert an a little-endian Vec representation of a binary number to an u8
pub fn bits_to_byte(bits: &[u8]) -> u8 {
    bits.iter()
        .rev() // Reverts iterator, necessary for little-endian representation
        .enumerate()
        .fold(0, |acc, (i, x)| acc + 2u8.pow(i as u32) * x)
}

// Convert a Vec of u8 values to a little-endian Vec representation of a binary number
pub fn bytes_to_bits(bytes: &[u8]) -> Vec<u8> {
    bytes.iter().flat_map(|byte| byte_to_bits(*byte)).collect() // Iterate over each byte
}

// Convert more than 8 bits to a Vec of u8
pub fn bits_to_bytes(bits: &[u8]) -> Vec<u8> {
    let pad = vec![0u8; 8 - (bits.len() % 8)];
    bits.iter()
        .map(|x| x.clone())
        .chain(pad.into_iter()) // Pads iterator so its length is a multiple of 8
        .collect::<Vec<u8>>()
        .chunks(8)
        .map(|chunk| bits_to_byte(chunk))
        .collect()
}

// Verify a bit stream (Vec) with bit parity and retrieve original content (bits)
pub fn check_parity(bits: &[u8], chunksize: usize, parity: u8) -> Result<Vec<u8>, String> {
    let parity = parity % 2;

    // Errors in the lines sums
    // Sum each line and check if it matches the parity
    let mut horizontal_errors = bits
        .chunks(chunksize + 1) // Divide each line
        .map(|chunk| chunk.iter().sum::<u8>() % 2) // Sum each line
        .enumerate()
        .filter(|(_, x)| *x != parity) // Filter lines with wrong sum
        .map(|(i, _)| i);
    // Errors in the columns sums
    // Sum each column and check if it matches the parity
    let mut vertical_errors = (0..chunksize)
        .map(|offset| {
            let mut i = offset;
            let mut sum_bit = 0;
            while let Some(x) = bits.get(i) {
                // Walk over line and sum values
                sum_bit = (sum_bit + x) % 2;
                i += chunksize + 1;
            }
            sum_bit
        })
        .enumerate()
        .filter(|(_, x)| *x != parity)
        .map(|(i, _)| i);

    let length = bits.len();
    // Retrieve original content
    let mut bits: Vec<u8> = bits
        .into_iter()
        .enumerate()
        .filter(|(i, _)| ((i + 1) % (chunksize + 1) != 0) & (i < &(length - chunksize - 1)))
        .map(|(_, x)| x.clone())
        .collect();

    // Fix errors, if possible
    match (vertical_errors.next(), horizontal_errors.next()) {
        (Some(x), Some(y)) => {
            println!("Corrigindo erro de paridade na posição {}, {}", x, y);
            match (horizontal_errors.next(), vertical_errors.next()) {
                (None, None) => {
                    bits[chunksize * y + x] = (1 + bits[chunksize * y + x]) % 2; // Invert error
                    Ok(bits)
                }
                _ => Err("More than one parity error detected, can't correct".to_owned()),
            }
        }
        (None, None) => Ok(bits),
        _ => Err("Control bit is wrong".to_owned()),
    }
}

// Implement parity check for a bit stream
pub fn add_parity_check(bits: &[u8], chunksize: usize, parity: u8) -> Vec<u8> {
    let parity = parity % 2;

    // mod 2 sum for each line, plus parity
    let sum_bits = bits
        .chunks(chunksize)
        .map(|chunk| (chunk.iter().sum::<u8>() + parity) % 2);

    // Sum for each column
    let last_line = (0..chunksize)
        .map(|offset| {
            let mut i = offset;
            let mut sum_bit = 0;
            while let Some(x) = bits.get(i) {
                sum_bit = (sum_bit + x) % 2;
                i += chunksize;
            }
            (sum_bit + parity) % 2
        })
        .collect::<Vec<u8>>();

    // Pad out missing values and chain original content and horizontal
    // and vertical sums
    let pad = vec![0u8; chunksize - (bits.len() % chunksize)];
    bits.into_iter()
        .map(|x| x.clone())
        .chain(pad.into_iter())
        .collect::<Vec<u8>>()
        .chunks(chunksize)
        .zip(sum_bits)
        .flat_map(|(bits, check)| {
            let mut line = bits.to_vec();
            line.push(check);
            line
        })
        .chain(last_line.clone().into_iter())
        .chain(iter::once((last_line.iter().sum::<u8>() + parity) % 2)) // Add last element
        .collect()
}

// Perform XOR sum between two binary numbers (Vec)
pub fn xor(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter()
        .zip_longest(b)
        .map(|x| match x {
            EitherOrBoth::Left(a) => a.clone(),
            EitherOrBoth::Right(b) => b.clone(),
            EitherOrBoth::Both(a, b) => (a + b) % 2,
        })
        .collect()
}

// Perform XOR division for CRC algorithm
pub fn xor_divide(dividend: &[u8], divisor: &[u8]) -> Vec<u8> {
    let length = divisor.len();
    let mut current: Vec<u8> = Vec::new();

    for x in dividend.iter() {
        // Fill current vector if it is too short
        if current.len() < length {
            current.push(x.clone());
        }
        if current.len() == length {
            if current[0] != 0 {
                current = xor(&current, divisor);
            }
            if current[0] == 0 {
                current.remove(0);
            }
        }
    }

    // Remove leading zeros
    for _ in 0..(current.len() - 1) {
        if current[0] == 0 {
            current.remove(0);
        }
    }

    current
}

// Generate CRC hash according to IEEE 802
pub fn generate_crc_hash(bits: &[u8]) -> Vec<u8> {
    let generator = vec![
        1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 1, 0, 1, 1, 0,
        1, 1, 1,
    ];

    let mut bits_padded = Vec::from(bits);
    bits_padded.extend([0u8; 32]);
    let mut remainder = xor_divide(&bits_padded, &generator);
    while remainder.len() < generator.len()-1 {
        remainder.insert(0, 0u8);
    }
    remainder
}

// Appends the CRC hash to the bit stream
pub fn append_crc_hash(bits: &[u8]) -> Vec<u8> {
    let mut bits_extended = bits.to_vec();
    let hash = generate_crc_hash(bits);
    bits_extended.extend(&hash);
    bits_extended
}

// Recover from bits and hash
pub fn recover_from_crc_hash(bits: &[u8]) -> Result<Vec<u8>, String> {
    let generator = [
        1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 1, 0, 1, 1, 0,
        1, 1, 1,
    ];
    let verify = xor_divide(&bits, &generator);
    let mut bits = Vec::from(bits);
    if verify == [0] {
        for _ in 0..generator.len() {
            bits.pop();
        }
        Ok(bits)
    }
    else {
        Err(String::from("Falha na verificação por CRC"))
    }
}
