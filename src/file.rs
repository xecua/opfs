use memmap::{MmapMut, MmapOptions};
use std::fs::{metadata, File, OpenOptions};
use std::io;

pub fn get_file_size(path: &str) -> io::Result<usize> {
    Ok(metadata(path)?.len() as usize)
}

pub fn open_readable_and_writable_file(path: &str) -> io::Result<File> {
    OpenOptions::new().read(true).write(true).open(path)
}

pub fn get_memory_mapped_file(file: &File, len: usize) -> io::Result<MmapMut> {
    unsafe { MmapOptions::new().len(len).map_mut(&file) }
}

pub fn open_new_file(path: &str) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
}
