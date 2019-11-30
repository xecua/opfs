use crate::block::inode::dinode;
use crate::block::sblock::superblock;
use memmap::MmapMut;
use std::convert::TryInto;

const ROOT_INODE: u32 = 1; // inode number of root directory("/")

// explore the given path, and return its inode
fn explore_path(m: &MmapMut, path: &str, super_block: &superblock) -> Result<dinode, String> {
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
            return Err(format!("{}: no such file or directory", path));
        }
    }
    Ok(current_inode)
}
pub fn ls(m: &MmapMut, path: &str, super_block: &superblock) {
    use crate::block::inode::*;
    use std::str::from_utf8;

    let inode = explore_path(m, path, super_block);
    if inode.is_err() {
        eprintln!("ls: {}", inode.unwrap_err());
        return;
    }
    let inode = inode.unwrap();
    match inode.r#type {
        InodeType::T_DIR => {
            let d = u8_slice_as_dirents(&m, inode.addrs[0].try_into().unwrap());
            for entry in d.into_iter() {
                let name = from_utf8(&entry.name).unwrap().trim_matches(char::from(0));
                if name.is_empty() {
                    continue;
                }
                let inode = u8_slice_as_dinode(&m, entry.inum.into(), &super_block);
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
        InodeType::T_FILE | InodeType::T_DEV => println!("{}", path),
    }
}
