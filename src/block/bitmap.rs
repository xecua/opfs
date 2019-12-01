use crate::block::sblock::superblock;

pub fn check(img: &[u8], block_num: usize, sblock: &superblock) -> bool {
    let bmapstart = sblock.bmapstart as usize;
    let byte: u8 = img[bmapstart + block_num / 8];
    match block_num % 8 {
        0 => byte & 0b00000001 == 0b00000001,
        1 => byte & 0b00000010 == 0b00000010,
        2 => byte & 0b00000100 == 0b00000100,
        3 => byte & 0b00001000 == 0b00001000,
        4 => byte & 0b00010000 == 0b00010000,
        5 => byte & 0b00100000 == 0b00100000,
        6 => byte & 0b01000000 == 0b01000000,
        7 => byte & 0b10000000 == 0b10000000,
        _ => panic!("bitmap::switch: exhausting pattern must not happen"),
    }
}

pub fn switch(img: &mut [u8], block_num: usize, sblock: &superblock) {
    let bmapstart = sblock.bmapstart as usize;
    let byte = &mut img[bmapstart + block_num / 8];
    match block_num % 8 {
        0 => *byte ^= 0b00000001,
        1 => *byte ^= 0b00000010,
        2 => *byte ^= 0b00000100,
        3 => *byte ^= 0b00001000,
        4 => *byte ^= 0b00010000,
        5 => *byte ^= 0b00100000,
        6 => *byte ^= 0b01000000,
        7 => *byte ^= 0b10000000,
        _ => panic!("bitmap::switch: exhausting pattern must not happen"),
    }
}
