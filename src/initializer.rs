use crate::types::superblock;
use crate::utils::*;
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

pub fn initialize(path: &str) -> (MmapMut, superblock) {
    use std::process::exit;
    let len = match get_file_size(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };
    let file = match open_readable_and_writable_file(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };
    let img = match get_memory_mapped_file(&file, len) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };
    let sblock = extract_superblock(&img);
    check_magic_number(&sblock);
    (img, sblock)
}
