use crate::block::inode::dinode;
use crate::block::inode::*;
use crate::block::sblock::superblock;
use crate::BLOCK_SIZE;
use memmap::MmapMut;
use std::process::exit;
use std::str::from_utf8;

const ROOT_INODE: usize = 1; // inode number of root directory("/")

// explore the given path, and return its inode
fn explore_path<'a>(
    img: &'a MmapMut,
    path: &str,
    sblock: &superblock,
) -> Result<&'a dinode, String> {
    let mut current_inode: &dinode = extract_inode_pointer_im(&img, ROOT_INODE, &sblock);
    if path != "/" {
        'directory: for file_name in path.split("/").skip(1) {
            for i in 0..NDIRECT {
                if current_inode.addrs[i] == 0 {
                    break;
                }
                for entry in u8_slice_as_dirents(&img, current_inode.addrs[i] as usize).into_iter()
                {
                    if file_name == from_utf8(&entry.name).unwrap().trim_matches(char::from(0)) {
                        current_inode = extract_inode_pointer_im(&img, entry.inum.into(), &sblock);
                        continue 'directory;
                    }
                }
            }
            // indirect reference block
            for i in u8_slice_as_u32_slice(&img, current_inode.addrs[NDIRECT] as usize).into_iter()
            {
                if *i == 0u32 {
                    continue;
                }
                for entry in u8_slice_as_dirents(&img, (*i) as usize).into_iter() {
                    if file_name == from_utf8(&entry.name).unwrap().trim_matches(char::from(0)) {
                        current_inode = extract_inode_pointer_im(&img, entry.inum.into(), &sblock);
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
    let inode = explore_path(&img, path, sblock);
    if inode.is_err() {
        eprintln!("ls: {}", inode.unwrap_err());
        exit(1);
    }
    let inode: &dinode = inode.unwrap();
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
                        continue;
                    }
                    let inode = extract_inode_pointer_im(&img, entry.inum.into(), &sblock);
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
                        continue;
                    }
                    let inode = extract_inode_pointer_im(&img, entry.inum.into(), &sblock);
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
        InodeType::ZERO => {
            panic!("get: type field is not set.");
        }
    }
}

pub fn get(img: &MmapMut, src: &str, dst: &str, sblock: &superblock) {
    use std::io::prelude::*;

    let inode = explore_path(&img, src, &sblock);
    if inode.is_err() {
        eprintln!("get: {}", inode.unwrap_err());
        exit(1);
    }
    let inode: &dinode = inode.unwrap();
    match inode.r#type {
        InodeType::T_DIR => {
            eprintln!("get: {} is a directory.", src);
            exit(1);
        }
        InodeType::T_DEV => {
            eprintln!("get: {} is a device file.", src);
            exit(1);
        }
        InodeType::T_FILE => {}
        InodeType::ZERO => {
            panic!("get: type field is not set.");
        }
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
                exit(1);
            }
        }
    }
    // indirect reference block
    for i in u8_slice_as_u32_slice(&img, inode.addrs[NDIRECT] as usize).into_iter() {
        if *i == 0u32 {
            break;
        }
        match dst_file.write(
            &img[(*i as usize) * BLOCK_SIZE
                ..(*i as usize) * BLOCK_SIZE + BLOCK_SIZE.min(file_size - written_size)],
        ) {
            Ok(s) => written_size += s,
            Err(e) => {
                eprintln!("get: {}", e);
                exit(1);
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

// for now, even if block becomes empty by deleting this file, not release the block.
pub fn rm(img: &mut MmapMut, path: &str, sblock: &superblock) {
    let mut dirent_block_number = 0;
    let mut dirent_offset = 0;
    let mut inode_num = ROOT_INODE;

    let mut data_block_nums: [u32; NDIRECT + 1] = [0; NDIRECT + 1];
    let mut del_flag = false; // delete file or not
                              // update inode information
    {
        let mut current_inode: &dinode = extract_inode_pointer_im(&img, inode_num, &sblock);
        if path != "/" {
            'directory: for file_name in path.split("/").skip(1) {
                for i in 0..NDIRECT {
                    if current_inode.addrs[i] == 0 {
                        break;
                    }
                    for (j, entry) in u8_slice_as_dirents(&img, current_inode.addrs[i] as usize)
                        .into_iter()
                        .enumerate()
                    {
                        if file_name == from_utf8(&entry.name).unwrap().trim_matches(char::from(0))
                        {
                            current_inode =
                                extract_inode_pointer_im(&img, entry.inum.into(), &sblock);
                            inode_num = entry.inum as usize;
                            dirent_block_number = current_inode.addrs[i];
                            dirent_offset = j;
                            continue 'directory;
                        }
                    }
                }
                // indirect reference block
                for i in
                    u8_slice_as_u32_slice(&img, current_inode.addrs[NDIRECT] as usize).into_iter()
                {
                    if *i == 0u32 {
                        continue;
                    }
                    for (j, entry) in u8_slice_as_dirents(&img, (*i) as usize)
                        .into_iter()
                        .enumerate()
                    {
                        if file_name == from_utf8(&entry.name).unwrap().trim_matches(char::from(0))
                        {
                            current_inode =
                                extract_inode_pointer_im(&img, entry.inum.into(), &sblock);
                            inode_num = entry.inum as usize;
                            dirent_block_number = *i;
                            dirent_offset = j;
                            continue 'directory;
                        }
                    }
                }
                // coming here means file does not exist.
                eprintln!("{}: no such file or directory", path);
                exit(1);
            }
        }

        match current_inode.r#type {
            InodeType::T_DIR => {
                eprintln!("rm: {} is a directory.", path);
                exit(1);
            }
            InodeType::T_DEV => {
                eprintln!("rm: {} is a device file.", path);
                exit(1);
            }
            InodeType::T_FILE => {}
            InodeType::ZERO => {
                panic!("get: type field is not set.");
            }
        }

        // update inode
        let current_inode: &mut dinode = extract_inode_pointer(&img, inode_num, &sblock);
        if current_inode.nlink != 1 {
            // only decrement its nlink
            (*current_inode).nlink -= 1;
        } else {
            del_flag = true;
            data_block_nums = current_inode.addrs.clone();

            *current_inode = dinode {
                r#type: InodeType::ZERO,
                major: 0,
                minor: 0,
                nlink: 0,
                size: 0,
                addrs: [0; NDIRECT + 1],
            };
        }
    }
    // delete related data block.
    if del_flag {
        for i in data_block_nums[..NDIRECT].iter() {
            if *i == 0 {
                break;
            }
            let block: &mut [u8; BLOCK_SIZE] = extract_block_pointer(&img, *i as usize);
            *block = [0u8; BLOCK_SIZE];
        }
        // indirect reference block
        if data_block_nums[NDIRECT] != 0 {
            for i in u8_slice_as_u32_slice(&img, data_block_nums[NDIRECT] as usize).iter() {
                let block: &mut [u8; BLOCK_SIZE] = extract_block_pointer(&img, *i as usize);
                *block = [0u8; BLOCK_SIZE];
            }
            let block: &mut [u8; BLOCK_SIZE] =
                extract_block_pointer(&img, data_block_nums[NDIRECT] as usize);
            *block = [0u8; BLOCK_SIZE];
        }
    }

    // finally, delete directory entry
    // put into block to limit mutable borrowing's lifetime
    {
        let dirent: &mut dirent =
            extract_dirent_pointer(&img, dirent_block_number as usize, dirent_offset);
        *dirent = dirent {
            inum: 0,
            name: [0u8; DIRSIZ],
        };
    }
}

// pub fn put(img: &mut MmapMut, src: &str, dst: &str, sblock: &superblock) {
//     use std::fs::{metadata, File};
//     use std::io::prelude::*;
//     let inodestart_addr = BLOCK_SIZE * sblock.inodestart as usize;

//     let met = match metadata(src) {
//         Ok(m) => m,
//         Err(e) => {
//             eprintln!("put: {}", e);
//             exit(1);
//         }
//     };
//     if !met.file_type().is_file() {
//         eprintln!("put: currently cannot put non-regular file.");
//         exit(1);
//     }
//     if met.len() > std::u32::MAX as u64 {
//         eprintln!(
//             "put: {} is too large (must be {} bytes or smaller)",
//             src,
//             std::u32::MAX
//         );
//         exit(1);
//     }

//     let candidate_inode_number = search_for_available_inode(&img, &sblock);
//     if candidate_inode_number.is_err() {
//         eprintln!("put: {}", candidate_inode_number.unwrap_err());
//         exit(1);
//     }
//     // for parallelism, mutual exclusion (by locking img) may be needed.
//     let candidate_inode_number = candidate_inode_number.unwrap();
//     let mut inode = dinode {
//         r#type: InodeType::T_FILE,
//         major: 0,
//         minor: 0,
//         nlink: 1,
//         size: met.len() as u32,
//         addrs: [0; 13],
//     };
//     let mut src_file = match File::open(src) {
//         Ok(f) => f,
//         Err(e) => {
//             eprintln!("put: {}", e);
//             exit(1);
//         }
//     };
//     let mut buf: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];
//     let mut block_nums: Vec<usize> = Vec::new();
//     let mut written_size: usize = 0;
//     while let Ok(_) = src_file.read_exact(&mut buf) {
//         let block_num = match search_for_available_dblock(&img, &sblock) {
//             Ok(i) => i,
//             Err(e) => {
//                 eprintln!("put: {}", e);
//                 exit(1);
//             }
//         };
//         for i in 0..BLOCK_SIZE {
//             img[block_num * BLOCK_SIZE + i] = buf[i];
//         }
//         block_nums.push(block_num);
//         written_size += BLOCK_SIZE;
//     }
//     {
//         // here buf has rest of file
//         let block_num = match search_for_available_dblock(&img, &sblock) {
//             Ok(i) => i,
//             Err(e) => {
//                 eprintln!("put: {}", e);
//                 exit(1);
//             }
//         };
//         for i in 0..met.len() as usize - written_size {
//             img[block_num * BLOCK_SIZE + i] = buf[i];
//         }
//         block_nums.push(block_num);
//     }

//     for (i, num) in block_nums[..NDIRECT].iter().enumerate() {
//         inode.addrs[i] = *num as u32;
//     }
//     if block_nums.len() > NDIRECT {
//         // allocate indirect reference block
//         let block_num = match search_for_available_dblock(&img, &sblock) {
//             Ok(i) => i,
//             Err(e) => {
//                 eprintln!("put: {}", e);
//                 exit(1);
//             }
//         };
//         inode.addrs[NDIRECT] = block_num as u32;
//         let mut ref_block: [u32; BLOCK_SIZE / 4] = [0; BLOCK_SIZE / 4];
//         for (i, num) in block_nums[NDIRECT..].iter().enumerate() {
//             ref_block[i] = *num as u32;
//         }
//         let block = u32_slice_as_u8_slice(&ref_block);
//         for i in 0..BLOCK_SIZE {
//             img[block_num * BLOCK_SIZE + i] = block[i];
//         }
//     }

//     // make new inode
//     let serialized_dinode = dinode_as_u8_slice(&inode);
//     for i in 0..DINODE_SIZE {
//         img[inodestart_addr + candidate_inode_number * DINODE_SIZE + i] = serialized_dinode[i];
//     }

//     // make dirent
//     let dirent: dirent = dirent {
//         inum: candidate_inode_number as u16,
//         name: dst.split('/').last().unwrap().as_bytes()[..DIRSIZ - 1]
//             .try_into()
//             .unwrap(),
//     };
//     let dirent_slice = dirent_as_u8_slice(&dirent);

//     let mut dirent_block_index = 0;
//     let mut dirent_offset = 0;
//     let mut parent_inode = extract_inode_pointer(&img, ROOT_INODE, &sblock);
//     let mut current_inode = extract_inode_pointer(&img, ROOT_INODE, &sblock);
//     let mut is_direct = true;
//     let dst_file_name = dst.split('/').last().unwrap();

//     // search for empty entry
//     if dst != "/" {
//         'directory: for file_name in dst.split("/").skip(1) {
//             for i in 0..NDIRECT {
//                 if current_inode.addrs[i] == 0 {
//                     break;
//                 }
//                 for (j, entry) in u8_slice_as_dirents(&img, current_inode.addrs[i] as usize)
//                     .into_iter()
//                     .enumerate()
//                 {
//                     let name = from_utf8(&entry.name).unwrap().trim_matches(char::from(0));
//                     if name.is_empty() {
//                         break;
//                     }
//                     if name == file_name {
//                         parent_inode = current_inode;
//                         current_inode = extract_inode_pointer(&img, entry.inum.into(), &sblock);
//                         dirent_block_index = i;
//                         dirent_offset = j;
//                         is_direct = true;
//                         continue 'directory;
//                     }
//                 }
//             }
//             // indirect reference block
//             for i in u8_slice_as_u32_slice(&img, current_inode.addrs[NDIRECT] as usize).into_iter()
//             {
//                 if *i == 0u32 {
//                     break;
//                 }
//                 for (j, entry) in u8_slice_as_dirents(&img, (*i) as usize)
//                     .into_iter()
//                     .enumerate()
//                 {
//                     let name = from_utf8(&entry.name).unwrap().trim_matches(char::from(0));
//                     if name.is_empty() {
//                         break;
//                     }
//                     if name == file_name {
//                         parent_inode = current_inode;
//                         current_inode = extract_inode_pointer(&img, entry.inum.into(), &sblock);
//                         dirent_block_index = *i as usize;
//                         dirent_offset = j;
//                         is_direct = false;
//                         continue 'directory;
//                     }
//                 }
//             }

//             // coming here means file does not exist.
//             if file_name == dst_file_name {
//                 // where to put file. make new dirent
//                 for i in 0..NDIRECT {
//                     if parent_inode.addrs[i] == 0u32 {
//                         // TODO: allocate new dirent block
//                         return;
//                     }
//                     let dirents = u8_slice_as_dirents(&img, parent_inode.addrs[i] as usize);
//                     for (j, dirent) in dirents.iter().enumerate() {
//                         if dirent.inum == 0 && dirent.name == [0; DIRSIZ] {
//                             for k in 0..DIRENT_SIZE {
//                                 img[parent_inode.addrs[i] as usize * BLOCK_SIZE
//                                     + j * DIRENT_SIZE
//                                     + k] = dirent_slice[k];
//                             }
//                             return;
//                         }
//                     }
//                 }
//                 // indirect reference block
//                 for i in
//                     u8_slice_as_u32_slice(&img, parent_inode.addrs[NDIRECT] as usize).into_iter()
//                 {
//                     if *i == 0u32 {
//                         // TODO: allocate new dirent block
//                         return;
//                     }
//                     for (j, entry) in u8_slice_as_dirents(&img, (*i) as usize)
//                         .into_iter()
//                         .enumerate()
//                     {
//                         if dirent.inum == 0 && dirent.name == [0; DIRSIZ] {
//                             for k in 0..DIRENT_SIZE {
//                                 img[parent_inode.addrs[*i as usize] as usize * BLOCK_SIZE
//                                     + j * DIRENT_SIZE
//                                     + k] = dirent_slice[k];
//                             }
//                             return;
//                         }
//                     }
//                 }
//             } else {
//                 // invalid path
//                 eprintln!("put: {}: no such file or directory", file_name);
//                 exit(1);
//             }
//         }
//         // already exist
//         eprintln!("put: {}: already exists.", dst);
//         exit(1);
//     } else {
//         eprintln!("put: cannot override /.");
//         exit(1);
//     }
// }
