use crate::block::sblock::superblock;
use crate::BLOCK_SIZE;
use memmap::MmapMut;
use std::mem::transmute;

pub const NDIRECT: usize = 12;
pub const DIRSIZ: usize = 14;

pub const DINODE_SIZE: usize = std::mem::size_of::<dinode>();
pub const DIRENT_SIZE: usize = std::mem::size_of::<dirent>();

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

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct dinode {
    pub r#type: InodeType,         // file type
    pub major: i16,                // device id
    pub minor: i16,                // device id
    pub nlink: i16,                // number of links
    pub size: u32,                 // file size
    pub addrs: [u32; NDIRECT + 1], // data block reference
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct dirent {
    pub inum: u16,
    pub name: [u8; DIRSIZ],
}

// ポインタのキャストだけをしたい
pub unsafe fn u8_slice_as_dinode<'a>(p: &'a mut [u8]) -> &'a mut dinode {
    transmute(p.as_ptr())
}

pub fn extract_inode_pointer<'a>(
    img: &'a mut [u8],
    inode_num: usize,
    sblock: &superblock,
) -> &'a mut dinode {
    if inode_num >= sblock.ninodes as usize {
        eprintln!(
            "inode access number limit exceeded: must be less than {}, given {}",
            sblock.ninodes, inode_num
        );
        std::process::exit(1);
    }
    let inodestart_byte = sblock.inodestart as usize * BLOCK_SIZE;
    unsafe {
        u8_slice_as_dinode(
            &mut img[inodestart_byte + inode_num * DINODE_SIZE
                ..inodestart_byte + (inode_num + 1) * DINODE_SIZE],
        )
    }
}

pub fn assign_inode<'a>(src: &'a dinode, dst: &'a mut dinode) {
    *dst = *src;
}

// ポインタのキャストだけをしたい
pub unsafe fn u8_slice_as_dinode_im<'a>(p: &'a [u8]) -> &'a dinode {
    transmute(p.as_ptr())
}

pub fn extract_inode_pointer_im<'a>(
    img: &'a [u8],
    inode_num: usize,
    sblock: &superblock,
) -> &'a dinode {
    if inode_num >= sblock.ninodes as usize {
        eprintln!(
            "inode access number limit exceeded: must be less than {}, given {}",
            sblock.ninodes, inode_num
        );
        std::process::exit(1);
    }
    let inodestart_byte = sblock.inodestart as usize * BLOCK_SIZE;
    unsafe {
        u8_slice_as_dinode_im(
            &img[inodestart_byte + inode_num * DINODE_SIZE
                ..inodestart_byte + (inode_num + 1) * DINODE_SIZE],
        )
    }
}
pub fn dinode_as_u8_slice(d: &dinode) -> &[u8] {
    unsafe { std::slice::from_raw_parts((d as *const dinode) as *const u8, DINODE_SIZE) }
}

pub fn dirent_as_u8_slice(d: &dirent) -> &[u8] {
    unsafe { std::slice::from_raw_parts((d as *const dirent) as *const u8, DIRENT_SIZE) }
}

// ポインタのキャストだけをしたい
pub unsafe fn u8_slice_as_dirent<'a>(p: &'a mut [u8]) -> &'a mut dirent {
    transmute(p.as_ptr())
}

pub fn extract_dirent_pointer<'a>(img: &'a mut [u8], addr: usize) -> &'a mut dirent {
    unsafe { u8_slice_as_dirent(&mut img[addr..addr + DIRENT_SIZE]) }
}

pub fn assign_dirent<'a>(src: &'a dirent, dst: &'a mut dirent) {
    *dst = *src;
}

pub fn u8_slice_as_dirents(m: &[u8], block_num: usize) -> Vec<dirent> {
    let mut dirents = Vec::new();
    for i in 0..BLOCK_SIZE / DIRENT_SIZE {
        let p = m[block_num * BLOCK_SIZE + DIRENT_SIZE * i
            ..block_num * BLOCK_SIZE + DIRENT_SIZE * (i + 1)]
            .as_ptr() as *const [u8; DIRENT_SIZE];
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

pub fn u32_slice_as_u8_slice(src: &[u32]) -> [u8; BLOCK_SIZE] {
    let p = src.as_ptr() as *const [u32; U32_PER_BLOCK];
    unsafe { std::mem::transmute(*p) }
}

pub fn search_for_available_inode(
    img: &memmap::MmapMut,
    sblock: &superblock,
) -> Result<usize, &'static str> {
    let inodestart_addr = sblock.inodestart as usize * BLOCK_SIZE;
    for i in 0..sblock.ninodes as usize {
        if img[inodestart_addr + i * DINODE_SIZE as usize
            ..inodestart_addr + (i + 1) * DINODE_SIZE as usize]
            .iter()
            .all(|&x| x == 0)
        {
            return Ok(i);
        }
    }
    Err("cannot allocate inode.")
}

pub fn search_for_available_dblock(
    img: &memmap::MmapMut,
    sblock: &superblock,
) -> Result<usize, &'static str> {
    let datastart = (sblock.bmapstart + sblock.size / 8) as usize; // start block number of datablock
    for i in datastart..sblock.size as usize {
        if img[i * BLOCK_SIZE..(i + 1) * BLOCK_SIZE]
            .iter()
            .all(|&x| x == 0)
        {
            return Ok(i);
        }
    }
    Err("cannot allocate data block.")
}
