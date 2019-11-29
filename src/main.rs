#[macro_use]
extern crate clap;

use memmap::MmapOptions;
use std::fs::{metadata, OpenOptions};

#[derive(Debug)]
struct superblock {
    magic: u32,      // Must be FSMAGIC
    size: u32,       // Size of file system image (blocks)
    nblocks: u32,    // Number of data blocks
    ninodes: u32,    // Number of inodes.
    nlog: u32,       // Number of log blocks
    logstart: u32,   // Block number of first log block
    inodestart: u32, // Block number of first inode block
    bmapstart: u32,  // Block number of first free map block
}

fn u8_slice_as_superblock(s: &[u8]) -> superblock {
    let p = s.as_ptr() as *const [u8; std::mem::size_of::<superblock>()];
    unsafe { std::mem::transmute(*p) }
}

fn main() {
    // let _app = app_from_crate!().get_matches();
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("./fs.img")
        .unwrap();
    let mmap = unsafe { MmapOptions::new().len(file_size).map_mut(&file).unwrap() };
    println!("{:?}", u8_slice_as_superblock(&*mmap));
}
