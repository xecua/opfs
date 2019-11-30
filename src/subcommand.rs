use crate::types::*;
use crate::utils::*;
use memmap::MmapMut;
use std::convert::TryInto;

// explore the given path, and return its inode
fn explore_path(m: &MmapMut, path: &str, sblock: &superblock) -> Result<dinode, String> {
    use std::str::from_utf8;

    let mut current_inode = extract_inode(&m, ROOT_INODE, &sblock);
    if path != "/" {
        'directory: for file_name in path.split("/").skip(1) {
            for i in 0..NDIRECT {
                if current_inode.addrs[i] == 0 {
                    continue;
                }
                let d = extract_dirents(&m, current_inode.addrs[i].try_into().unwrap(), &sblock);
                for entry in d.into_iter() {
                    let name = from_utf8(&entry.name).unwrap().trim_matches(char::from(0));
                    if name.is_empty() {
                        break;
                    }
                    if name == file_name {
                        current_inode = extract_inode(&m, entry.inum.into(), &sblock);
                        continue 'directory;
                    }
                }
            }
            // indirect reference block
            for i in extract_indirect_reference_block(&m, current_inode.addrs[NDIRECT] as usize)
                .into_iter()
            {
                if *i == 0u32 {
                    break;
                }
                let d = extract_dirents(&m, (*i) as usize, &sblock);
                for entry in d.into_iter() {
                    let name = from_utf8(&entry.name).unwrap().trim_matches(char::from(0));
                    if name.is_empty() {
                        break;
                    }
                    if name == file_name {
                        current_inode = extract_inode(&m, entry.inum.into(), &sblock);
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
pub fn ls(m: &MmapMut, path: &str, sblock: &superblock) {
    use std::str::from_utf8;

    let inode = explore_path(m, path, sblock);
    if inode.is_err() {
        eprintln!("ls: {}", inode.unwrap_err());
        return;
    }
    let inode = inode.unwrap();
    match inode.r#type {
        InodeType::T_DIR => {
            let d = extract_dirents(&m, inode.addrs[0] as usize, &sblock);
            for entry in d.into_iter() {
                let name = from_utf8(&entry.name).unwrap().trim_matches(char::from(0));
                if name.is_empty() {
                    continue;
                }
                let inode = extract_inode(&m, entry.inum.into(), &sblock);
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
