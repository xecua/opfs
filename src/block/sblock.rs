use crate::BLOCK_SIZE;

// Super Block
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct superblock {
    pub magic: u32,      // Must be FSMAGIC
    pub size: u32,       // Size of file system image (blocks)
    nblocks: u32,        // Number of data blocks
    pub ninodes: u32,    // Number of inodes.
    nlog: u32,           // Number of log blocks
    logstart: u32,       // Block number of first log block
    pub inodestart: u32, // Block number of first inode block
    bmapstart: u32,      // Block number of first free map block
}

pub fn u8_slice_as_superblock(s: &[u8]) -> superblock {
    let p =
        s[BLOCK_SIZE..BLOCK_SIZE * 2].as_ptr() as *const [u8; std::mem::size_of::<superblock>()];
    unsafe { std::mem::transmute(*p) }
}

pub fn check_magic_number(s: &superblock) {
    if s.magic != 0x10203040 {
        panic!("magic number is invalid: {:x}", s.magic);
    }
}
