
fn round(input: [u8; 8], key: [u8; 10]) -> [u8; 8] {
    todo!();
}

fn key_addition(input: [u8; 8], key: [u8; 10]) -> [u8; 8] {
    todo!();
}

fn key_schedule(input: [u8; 10], round: u8) -> [u8; 10] {
    assert!(round <= 31);
    //left rotate by 61
    let mut bits: [_; 80] = bytes_to_bits(input);
    bits.rotate_left(61);
    let mut bytes: [_; 10] = bits_to_bytes(bits);
    //sbox last 4 bits
    let last_nibble = bytes[0] >> 4;
    let new_last_nibble = sbox(last_nibble);
    bytes[0] &= 0x0f;
    bytes[0] |= new_last_nibble << 4;
    //xor bits 15 to 19 with the round number
    let mut bits: [_; 80] = bytes_to_bits(bytes);
    let fifteen_to_nineteen = &mut bits[60..=64];
    let round_bits = &byte_to_bits(round)[3..8];
    for i in 0..5 {
        fifteen_to_nineteen[i] ^= round_bits[i]
    }
    bits_to_bytes(bits)
}

fn bits_to_bytes<const N: usize, const M: usize>(bits: [bool; N]) -> [u8; M] {
    assert_eq!(N,M*8);
    let mut bytes = [0; M];
    for i in (0..N).step_by(8) {
        let byte_bits: [_;8] = bits[i..(i+8)].try_into().unwrap();
        let byte = bits_to_byte(byte_bits);
        bytes[i/8] = byte;
    }
    bytes
}

fn bytes_to_bits<const N: usize, const M: usize>(bytes: [u8; N]) -> [bool; M] {
    bytes
        .map(byte_to_bits)
        .into_iter()
        .flatten()
        .collect::<Vec<bool>>()
        .try_into()
        .unwrap()
}

fn bits_to_byte(bits: [bool; 8]) -> u8 {
    let mut byte = 0;
    for i in 0..8 {
        let bit = if bits[i] {1} else {0};
        byte |= bit << (7-i)
    }
    byte
}

fn byte_to_bits(byte: u8) -> [bool; 8] {
    let mut bits = [0; 8];
    for i in 0..8 {
        bits[i] = (byte >> (7-i)) & 1
    }
    bits.map(|b| if b == 1 {true} else {false})
}

fn sbox(input: u8) -> u8 {
    if input > 0xf {panic!("sbox input out of range");}
    let lookup = [0xc,0x5,0x6,0xb,0x9,0x0,0xa,0xd,0x3,0xe,0xf,0x8,0x4,0x7,0x1,0x2];
    lookup[input as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_to_bits() {
        assert_eq!(byte_to_bits(0xff),[true; 8]);
        assert_eq!(byte_to_bits(0xc3),[true,true,false,false,false,false,true,true]);
        assert_eq!(byte_to_bits(0x34),[false,false,true,true,false,true,false,false]);
        assert_eq!(byte_to_bits(0x01),[false,false,false,false,false,false,false,true]);
    }

    #[test]
    fn test_bytes_to_bits() {
        assert_eq!(bytes_to_bits([0xff,0xff]),[true; 16]);
        assert_eq!(bytes_to_bits([0x0,0x0]),[false; 16]);
        assert_eq!(bytes_to_bits([0xc3,0x34]),[
            true,true,false,false,false,false,true,true,
            false,false,true,true,false,true,false,false
        ]);
    }

    #[test]
    fn test_bits_to_byte() {
        assert_eq!(bits_to_byte([true; 8]),0xff);
        assert_eq!(bits_to_byte([true,true,false,false,false,false,true,true]),0xc3);
        assert_eq!(bits_to_byte([false,false,true,true,false,true,false,false]),0x34);
        assert_eq!(bits_to_byte([false,false,false,false,false,false,false,true]),0x01);
    }

    #[test]
    fn test_key_schedule() {
        assert_eq!(
            key_schedule([0xdc,0x87,0x70,0xe9,0x3e,0xa1,0x41,0xe1,0xfc,0x67],1),
            [0xbf,0x8c,0xfb,0x90,0xee,0x1d,0x27,0xd4,0xa8,0x3c]
        );
        assert_eq!(
            key_schedule([0xbf,0x8c,0xfb,0x90,0xee,0x1d,0x27,0xd4,0xa8,0x3c],2),
            [0xe5,0x07,0x97,0xf1,0x9f,0x72,0x1d,0xc2,0xa4,0xfa]
        );
    }

    #[test]
    fn test_sbox() {
        assert_eq!(sbox(0x0), 0xc);
        assert_eq!(sbox(0x1), 0x5);
        assert_eq!(sbox(0x2), 0x6);
        assert_eq!(sbox(0x3), 0xb);
        assert_eq!(sbox(0x4), 0x9);
        assert_eq!(sbox(0x5), 0x0);
        assert_eq!(sbox(0x6), 0xa);
        assert_eq!(sbox(0x7), 0xd);
        assert_eq!(sbox(0x8), 0x3);
        assert_eq!(sbox(0x9), 0xe);
        assert_eq!(sbox(0xa), 0xf);
        assert_eq!(sbox(0xb), 0x8);
        assert_eq!(sbox(0xc), 0x4);
        assert_eq!(sbox(0xd), 0x7);
        assert_eq!(sbox(0xe), 0x1);
        assert_eq!(sbox(0xf), 0x2);
    }
}
