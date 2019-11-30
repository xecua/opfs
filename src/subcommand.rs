use crate::block::sblock::superblock;
use memmap::MmapMut;
use std::convert::TryInto;

const ROOT_INODE: u32 = 1; // inode number of root directory("/")

pub fn ls(m: &MmapMut, path: &str, super_block: &superblock) {
    use crate::block::inode::*;
    use std::str::from_utf8;

    let mut current_inode = u8_slice_as_dinode(&m, ROOT_INODE, &super_block);
    if path != "/" {
        'outer: for file_name in path.split("/").skip(1) {
            // TODO: addrsのループ(再帰にしたほうが良い?)
            let d = u8_slice_as_dirents(&m, current_inode.addrs[0].try_into().unwrap());
            'inner: for entry in d.into_iter() {
                let name = from_utf8(&entry.name).unwrap().trim_matches(char::from(0));
                if name.is_empty() {
                    break;
                }
                if name == file_name {
                    current_inode = u8_slice_as_dinode(&m, entry.inum.into(), &super_block);
                    continue 'outer;
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
