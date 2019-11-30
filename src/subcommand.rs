use crate::block::sblock::superblock;
use crate::BLOCK_SIZE;
use memmap::MmapMut;
use std::convert::TryInto;

const ROOT_INODE: u32 = 1; // inode number of root directory("/")

pub fn ls(m: &MmapMut, path: &str, super_block: &superblock) {
    use crate::block::inode::*;
    use std::str::from_utf8;

    let mut current_inode = u8_slice_as_dinode(&m, ROOT_INODE, &super_block);
    if path != "/" {
        'directory: for file_name in path.split("/").skip(1) {
            for i in 0..NDIRECT {
                let d = u8_slice_as_dirents(&m, current_inode.addrs[i].try_into().unwrap());
                for entry in d.into_iter() {
                    let name = from_utf8(&entry.name).unwrap().trim_matches(char::from(0));
                    if name.is_empty() {
                        break;
                    }
                    if name == file_name {
                        current_inode = u8_slice_as_dinode(&m, entry.inum.into(), &super_block);
                        continue 'directory;
                    }
                }
            }
            // indirect reference block
            for i in u8_slice_as_u32_slice(&m, current_inode.addrs[NDIRECT].try_into().unwrap())
                .into_iter()
            {
                if *i == 0u32 {
                    break;
                }
                let d = u8_slice_as_dirents(&m, (*i).try_into().unwrap());
                for entry in d.into_iter() {
                    let name = from_utf8(&entry.name).unwrap().trim_matches(char::from(0));
                    if name.is_empty() {
                        break;
                    }
                    if name == file_name {
                        current_inode = u8_slice_as_dinode(&m, entry.inum.into(), &super_block);
                        continue 'directory;
                    }
                }
            }
            // coming here means file does not exist.
            eprintln!("ls: {}: no such file or directory", path);
            std::process::exit(1);
        }
    }

    match current_inode.r#type {
        InodeType::T_DIR => {
            let d = u8_slice_as_dirents(&m, current_inode.addrs[0].try_into().unwrap());
            for entry in d.into_iter() {
                let name = from_utf8(&entry.name).unwrap().trim_matches(char::from(0));
                if name.is_empty() {
                    continue;
                }
                println!("{}", name);
            }
        }
        InodeType::T_FILE | InodeType::T_DEV => println!("{}", path),
    }
}
