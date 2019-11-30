#[allow(non_camel_case_types)]
// Super Block
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct superblock {
    pub magic: u32,      // Must be FSMAGIC
    pub size: u32,       // Size of file system image (blocks)
    pub nblocks: u32,    // Number of data blocks
    pub ninodes: u32,    // Number of inodes.
    nlog: u32,           // Number of log blocks
    logstart: u32,       // Block number of first log block
    pub inodestart: u32, // Block number of first inode block
    bmapstart: u32,      // Block number of first free map block
}
// dinode.type
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
#[repr(i16)]
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
pub struct dinode {
    pub r#type: InodeType,         // file type
    major: i16,                    // device id
    minor: i16,                    // device id
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

pub const BLOCK_SIZE: usize = 1024;
pub const NDIRECT: usize = 12;
pub const DIRSIZ: usize = 14;
pub const ROOT_INODE: usize = 1; // inode number of root directory("/")
pub const DINODE_SIZE: usize = std::mem::size_of::<dinode>();
pub const DIRENT_SIZE: usize = std::mem::size_of::<dirent>();
pub const U32_PER_BLOCK: usize = BLOCK_SIZE / std::mem::size_of::<u32>();
