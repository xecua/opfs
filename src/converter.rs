use crate::types::*;
use std::ops::Range;

pub fn block_num_to_addr_range(block_num: usize) -> Range<usize> {
    BLOCK_SIZE * block_num..BLOCK_SIZE * (block_num + 1)
}

pub fn inode_num_to_addr_range(inode_num: usize, sblock: &superblock) -> Range<usize> {
    ((sblock.inodestart as usize) * BLOCK_SIZE) + inode_num * DINODE_SIZE
        ..((sblock.inodestart as usize) * BLOCK_SIZE) + (inode_num + 1) * DINODE_SIZE
}

pub fn u8_slice_as_dinode(slice: &[u8]) -> dinode {
    let p = slice.as_ptr() as *const [u8; DINODE_SIZE];
    unsafe { std::mem::transmute(*p) }
}

pub fn u8_slice_as_dirent(slice: &[u8]) -> dirent {
    unsafe { std::mem::transmute(slice) }
}

pub fn u8_slice_as_dirents(slice: &[u8], sblock: &superblock) -> Vec<dirent> {
    let mut dirents = Vec::new();
    for i in 0..BLOCK_SIZE / DIRENT_SIZE {
        dirents.push(u8_slice_as_dirent(
            &slice[inode_num_to_addr_range(i, &sblock)],
        ));
    }
    dirents
}
