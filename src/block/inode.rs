use crate::block::sblock::superblock;
use crate::BLOCK_SIZE;

pub const NDIRECT: usize = 12;
const DIRSIZ: usize = 14;

pub const DINODE_SIZE: usize = std::mem::size_of::<dinode>();
const DIRENT_SIZE: usize = std::mem::size_of::<dirent>();

// dinode.type
#[repr(i16)]
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum InodeType {
    T_DIR = 1,
    T_FILE = 2,
    T_DEV = 3,
}
impl std::fmt::Display for InodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use InodeType::*;
        match self {
            T_DIR => write!(f, "directory"),
            T_FILE => write!(f, "file"),
            T_DEV => write!(f, "device file"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct dinode {
    pub r#type: InodeType,         // file type
    major: i16,                    // device id
    minor: i16,                    // device id
    pub jnlink: i16,               // number of links
    pub size: u32,                 // file size
    pub addrs: [u32; NDIRECT + 1], // data block reference
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct dirent {
    pub inum: u16,
    pub name: [u8; DIRSIZ],
}

pub fn u8_slice_as_dinode(m: &[u8], inode_num: u32, super_block: &superblock) -> dinode {
    if inode_num >= super_block.ninodes {
        panic!(
            "inode access number limit exceeded: must be less than {}, given {}",
            super_block.ninodes, inode_num
        );
    }
    let inodestart_byte = super_block.inodestart as usize * BLOCK_SIZE;
    let p = m[inodestart_byte + (inode_num as usize) * DINODE_SIZE
        ..inodestart_byte + ((inode_num as usize) + 1) * DINODE_SIZE]
        .as_ptr() as *const [u8; DINODE_SIZE];
    unsafe { std::mem::transmute(*p) }
}

pub fn u8_slice_as_dirents(m: &[u8], block_num: usize) -> Vec<dirent> {
    let mut dirents = Vec::new();
    for i in 0..BLOCK_SIZE / DIRENT_SIZE {
        let p = m[block_num * BLOCK_SIZE + DIRENT_SIZE * i
            ..block_num * BLOCK_SIZE + DIRENT_SIZE * (i + 1)]
            .as_ptr() as *const [u8; std::mem::size_of::<dirent>()];
        unsafe { dirents.push(std::mem::transmute(*p)) };
    }
    dirents
}

// for indirect reference
const U32_PER_BLOCK: usize = BLOCK_SIZE / std::mem::size_of::<u32>();
pub fn u8_slice_as_u32_slice(m: &[u8], block_num: usize) -> [u32; U32_PER_BLOCK] {
    let p =
        m[block_num * BLOCK_SIZE..(block_num + 1) * BLOCK_SIZE].as_ptr() as *const [u8; BLOCK_SIZE];
    unsafe { std::mem::transmute(*p) }
}
