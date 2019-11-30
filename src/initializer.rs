use crate::types::superblock;
use memmap::{MmapMut, MmapOptions};
use std::fs::{metadata, File, OpenOptions};
use std::io;

fn get_file_size(path: &str) -> io::Result<usize> {
    Ok(metadata(path)?.len() as usize)
}

fn open_readable_and_writable_file(path: &str) -> io::Result<File> {
    OpenOptions::new().read(true).write(true).open(path)
}

fn get_memory_mapped_file(file: &File, len: usize) -> io::Result<MmapMut> {
    unsafe { MmapOptions::new().len(len).map_mut(&file) }
}

fn check_magic_number(sblock: &superblock) {
    if sblock.magic != 0x10203040 {
        panic!("magic number is invalid: {:x}", sblock.magic);
    }
}
