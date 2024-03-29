use crate::block::sblock::superblock;
use crate::BLOCK_SIZE;
use memmap::MmapMut;
use std::mem::transmute;
use std::str::from_utf8;

pub const NDIRECT: usize = 12;
pub const DIRSIZ: usize = 14;

pub const DINODE_SIZE: usize = std::mem::size_of::<dinode>();
pub const DIRENT_SIZE: usize = std::mem::size_of::<dirent>();

pub const ROOT_INODE: usize = 1; // inode number of root directory("/")

// dinode.type
#[repr(i16)]
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum InodeType {
    T_DIR = 1,
    T_FILE = 2,
    T_DEV = 3,
    ZERO = 0, // for resetting
}
impl std::fmt::Display for InodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use InodeType::*;
        match self {
            T_DIR => write!(f, "directory"),
            T_FILE => write!(f, "file"),
            T_DEV => write!(f, "device file"),
            ZERO => panic!("type field is not set."),
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

// explore the given path, and return its inode
pub fn explore_path<'a>(
    img: &'a MmapMut,
    path: &str,
    sblock: &superblock,
) -> Result<(&'a dinode, usize), String> {
    let mut current_inode: &dinode = extract_inode_pointer_im(&img, ROOT_INODE, &sblock);
    let mut current_inode_num: usize = 0;
    if path != "/" {
        'directory: for file_name in path.split("/").skip(1) {
            for i in 0..NDIRECT {
                if current_inode.addrs[i] == 0 {
                    break;
                }
                for entry in
                    extract_dirents_pointer_im(&img, current_inode.addrs[i] as usize).into_iter()
                {
                    if file_name == from_utf8(&entry.name).unwrap().trim_matches(char::from(0)) {
                        current_inode = extract_inode_pointer_im(&img, entry.inum.into(), &sblock);
                        current_inode_num = entry.inum.into();
                        continue 'directory;
                    }
                }
            }
            if current_inode.addrs[NDIRECT] != 0 {
                // indirect reference block
                for i in extract_indirect_reference_block_pointer_im(
                    &img,
                    current_inode.addrs[NDIRECT] as usize,
                )
                .into_iter()
                {
                    if *i == 0u32 {
                        continue;
                    }
                    for entry in extract_dirents_pointer_im(&img, (*i) as usize).into_iter() {
                        if file_name == from_utf8(&entry.name).unwrap().trim_matches(char::from(0))
                        {
                            current_inode =
                                extract_inode_pointer_im(&img, entry.inum.into(), &sblock);
                            current_inode_num = entry.inum.into();
                            continue 'directory;
                        }
                    }
                }
            }
            // coming here means file does not exist.
            return Err(format!("{}: no such file or directory", path));
        }
    }
    Ok((current_inode, current_inode_num))
}

// pointer casting
pub unsafe fn u8_slice_as_dinode<'a>(p: &'a [u8]) -> &'a mut dinode {
    transmute(p.as_ptr())
}

pub fn extract_inode_pointer<'a>(
    img: &'a [u8],
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
            &img[inodestart_byte + inode_num * DINODE_SIZE
                ..inodestart_byte + (inode_num + 1) * DINODE_SIZE],
        )
    }
}

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

pub unsafe fn u8_slice_as_dirent<'a>(p: &'a [u8]) -> &'a mut dirent {
    transmute(p.as_ptr())
}

pub fn extract_dirent_pointer<'a>(
    img: &'a [u8],
    block_num: usize,
    offset: usize,
) -> &'a mut dirent {
    let addr = block_num * BLOCK_SIZE + offset * DIRENT_SIZE;
    unsafe { u8_slice_as_dirent(&img[addr..addr + DIRENT_SIZE]) }
}

pub unsafe fn u8_slice_as_dirents_im<'a>(p: &'a [u8]) -> &'a [dirent; BLOCK_SIZE / DIRENT_SIZE] {
    transmute(p.as_ptr())
}

pub fn extract_dirents_pointer_im<'a>(
    img: &'a [u8],
    block_num: usize,
) -> &'a [dirent; BLOCK_SIZE / DIRENT_SIZE] {
    unsafe { u8_slice_as_dirents_im(&img[block_num * BLOCK_SIZE..(block_num + 1) * BLOCK_SIZE]) }
}

pub unsafe fn u8_slice_as_dirents<'a>(p: &'a [u8]) -> &'a mut [dirent; BLOCK_SIZE / DIRENT_SIZE] {
    transmute(p.as_ptr())
}

pub fn extract_dirents_pointer<'a>(
    img: &'a [u8],
    block_num: usize,
) -> &'a mut [dirent; BLOCK_SIZE / DIRENT_SIZE] {
    unsafe { u8_slice_as_dirents(&img[block_num * BLOCK_SIZE..(block_num + 1) * BLOCK_SIZE]) }
}

pub unsafe fn u8_slice_as_block<'a>(p: &'a [u8]) -> &'a mut [u8; BLOCK_SIZE] {
    transmute(p.as_ptr())
}

pub fn extract_block_pointer<'a>(img: &'a [u8], block_num: usize) -> &'a mut [u8; BLOCK_SIZE] {
    unsafe { u8_slice_as_block(&img[block_num * BLOCK_SIZE..(block_num + 1) * BLOCK_SIZE]) }
}

// for indirect reference
pub const U32_PER_BLOCK: usize = BLOCK_SIZE / std::mem::size_of::<u32>();

pub unsafe fn u8_slice_as_u32_slice_im<'a>(p: &'a [u8]) -> &'a [u32; U32_PER_BLOCK] {
    transmute(p.as_ptr())
}
pub fn extract_indirect_reference_block_pointer_im<'a>(
    m: &'a [u8],
    block_num: usize,
) -> &'a [u32; U32_PER_BLOCK] {
    unsafe { u8_slice_as_u32_slice_im(&m[block_num * BLOCK_SIZE..(block_num + 1) * BLOCK_SIZE]) }
}

pub unsafe fn u8_slice_as_u32_slice<'a>(p: &'a [u8]) -> &'a mut [u32; U32_PER_BLOCK] {
    transmute(p.as_ptr())
}

pub fn extract_indirect_reference_block_pointer<'a>(
    img: &'a [u8],
    block_num: usize,
) -> &'a mut [u32; U32_PER_BLOCK] {
    unsafe { u8_slice_as_u32_slice(&img[block_num * BLOCK_SIZE..(block_num + 1) * BLOCK_SIZE]) }
}

pub fn search_for_available_inode(
    img: &memmap::MmapMut,
    sblock: &superblock,
) -> Result<usize, &'static str> {
    let inodestart_addr = sblock.inodestart as usize * BLOCK_SIZE;
    for i in 1..sblock.ninodes as usize {
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
    let datastart = (sblock.bmapstart + sblock.size / 8) as usize; // start block number of data block
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
