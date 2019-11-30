use crate::converter::*;
use crate::types::*;

pub fn extract_superblock(img: &[u8]) -> superblock {
    let p =
        img[block_num_to_addr_range(1)].as_ptr() as *const [u8; std::mem::size_of::<superblock>()];
    unsafe { std::mem::transmute(*p) }
}

pub fn extract_inode(img: &[u8], inode_num: usize, sblock: &superblock) -> dinode {
    if inode_num >= sblock.ninodes as usize {
        eprintln!(
            "inode access number limit exceeded: must be less than {}, given {}",
            sblock.ninodes, inode_num
        );
        std::process::exit(1);
    }
    u8_slice_as_dinode(&img[inode_num_to_addr_range(inode_num, &sblock)])
}

pub fn block_as_dirents(slice: &[u8]) -> Vec<dirent> {
    let mut dirents = Vec::new();
    for i in 0..BLOCK_SIZE / DIRENT_SIZE {
        dirents.push(u8_slice_as_dirent(
            &slice[i * DIRENT_SIZE..(i + 1) * DIRENT_SIZE],
        ));
    }
    dirents
}

pub fn extract_dirents(img: &[u8], block_num: usize) -> Vec<dirent> {
    block_as_dirents(&img[block_num_to_addr_range(block_num)])
}

// for indirect reference
pub fn extract_indirect_reference_block(img: &[u8], block_num: usize) -> [u32; U32_PER_BLOCK] {
    let p = img[block_num_to_addr_range(block_num)].as_ptr() as *const [u8; BLOCK_SIZE];
    unsafe { std::mem::transmute(*p) }
}
