use crate::block::inode::dinode;
use crate::block::sblock::superblock;
use crate::BLOCK_SIZE;
use memmap::MmapMut;

const ROOT_INODE: u32 = 1; // inode number of root directory("/")

// explore the given path, and return its inode
fn explore_path(img: &MmapMut, path: &str, sblock: &superblock) -> Result<dinode, String> {
    use crate::block::inode::*;
    use std::str::from_utf8;

    let mut current_inode = u8_slice_as_dinode(&img, ROOT_INODE, &sblock);
    if path != "/" {
        'directory: for file_name in path.split("/").skip(1) {
            for i in 0..NDIRECT {
                if current_inode.addrs[i] == 0 {
                    break;
                }
                let d = u8_slice_as_dirents(&img, current_inode.addrs[i] as usize);
                for entry in d.into_iter() {
                    let name = from_utf8(&entry.name).unwrap().trim_matches(char::from(0));
                    if name.is_empty() {
                        break;
                    }
                    if name == file_name {
                        current_inode = u8_slice_as_dinode(&img, entry.inum.into(), &sblock);
                        continue 'directory;
                    }
                }
            }
            // indirect reference block
            for i in u8_slice_as_u32_slice(&img, current_inode.addrs[NDIRECT] as usize).into_iter()
            {
                if *i == 0u32 {
                    break;
                }
                let d = u8_slice_as_dirents(&img, (*i) as usize);
                for entry in d.into_iter() {
                    let name = from_utf8(&entry.name).unwrap().trim_matches(char::from(0));
                    if name.is_empty() {
                        break;
                    }
                    if name == file_name {
                        current_inode = u8_slice_as_dinode(&img, entry.inum.into(), &sblock);
                        continue 'directory;
                    }
                }
            }
            // coming here means file does not exist.
            return Err(format!("{}: no such file or directory", path));
        }
    }
    Ok(current_inode)
}
pub fn ls(img: &MmapMut, path: &str, sblock: &superblock) {
    use crate::block::inode::*;
    use std::str::from_utf8;

    let inode = explore_path(img, path, sblock);
    if inode.is_err() {
        eprintln!("ls: {}", inode.unwrap_err());
        return;
    }
    let inode = inode.unwrap();
    match inode.r#type {
        InodeType::T_DIR => {
            for i in 0..NDIRECT {
                if inode.addrs[i] == 0 {
                    break;
                }
                let d = u8_slice_as_dirents(&img, inode.addrs[i] as usize);
                for entry in d.into_iter() {
                    let name = from_utf8(&entry.name).unwrap().trim_matches(char::from(0));
                    if name.is_empty() {
                        break;
                    }
                    let inode = u8_slice_as_dinode(&img, entry.inum.into(), &sblock);
                    println!(
                        "{:<width$}: {}, No.{}, {} Bytes",
                        name,
                        inode.r#type,
                        entry.inum,
                        inode.size,
                        width = DIRSIZ
                    );
                }
            }
            // indirect reference block
            for i in u8_slice_as_u32_slice(&img, inode.addrs[NDIRECT] as usize).into_iter() {
                if *i == 0u32 {
                    break;
                }
                let d = u8_slice_as_dirents(&img, (*i) as usize);
                for entry in d.into_iter() {
                    let name = from_utf8(&entry.name).unwrap().trim_matches(char::from(0));
                    if name.is_empty() {
                        break;
                    }
                    let inode = u8_slice_as_dinode(&img, entry.inum.into(), &sblock);
                    println!(
                        "{:<width$}: {}, No.{}, {} Bytes",
                        name,
                        inode.r#type,
                        entry.inum,
                        inode.size,
                        width = DIRSIZ
                    );
                }
            }
        }
        InodeType::T_FILE | InodeType::T_DEV => println!("{}", path),
    }
}

pub fn get(img: &MmapMut, src: &str, dst: &str, sblock: &superblock) {
    use crate::block::inode::*;
    use std::io::prelude::*;

    let inode = explore_path(&img, src, &sblock);
    if inode.is_err() {
        eprintln!("ls: {}", inode.unwrap_err());
        return;
    }
    let inode = inode.unwrap();
    match inode.r#type {
        InodeType::T_DIR => {
            eprintln!("get: {} is a directory.", src);
            return;
        }
        InodeType::T_DEV => {
            eprintln!("get: {} is a device file.", src);
            return;
        }
        InodeType::T_FILE => {}
    }
    let dst_file = crate::file::open_new_file(dst);
    if dst_file.is_err() {
        eprintln!("get: cannot open file: {}", dst);
    }
    let mut dst_file = dst_file.unwrap();
    let mut written_size: usize = 0;
    let file_size: usize = inode.size as usize;
    for i in 0..NDIRECT {
        if inode.addrs[i] == 0 {
            break;
        }
        match dst_file.write(
            &img[(inode.addrs[i] as usize) * BLOCK_SIZE
                ..(inode.addrs[i] as usize) * BLOCK_SIZE
                    + BLOCK_SIZE.min(file_size - written_size)],
        ) {
            Ok(s) => written_size += s,
            Err(e) => {
                eprintln!("get: {}", e);
                return;
            }
        }
    }
    // indirect reference block
    for i in u8_slice_as_u32_slice(&img, inode.addrs[NDIRECT] as usize).into_iter() {
        if *i == 0u32 {
            break;
        }
        match dst_file.write(
            &img[(inode.addrs[*i as usize] as usize) * BLOCK_SIZE
                ..(inode.addrs[*i as usize] as usize) * BLOCK_SIZE
                    + BLOCK_SIZE.min(file_size - written_size)],
        ) {
            Ok(s) => written_size += s,
            Err(e) => {
                eprintln!("get: {}", e);
                return;
            }
        }
    }
    if written_size != (inode.size as usize) {
        eprintln!(
            "get: written size does not match. expected: {}, actual: {}",
            inode.size, written_size
        );
    }
}
