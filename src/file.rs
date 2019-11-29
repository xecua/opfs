use memmap::{MmapMut, MmapOptions};
use std::fs::{metadata, File, OpenOptions};
use std::io;

pub fn get_file_size(path: &str) -> io::Result<usize> {
    Ok(metadata(path)?.len() as usize)
}

pub fn open_readable_and_writable_file(path: &str) -> io::Result<File> {
    OpenOptions::new().read(true).write(true).open(path)
}

pub unsafe fn get_memory_mapped_file(file: &File, len: usize) -> io::Result<MmapMut> {
    MmapOptions::new().len(len).map_mut(&file)
}
