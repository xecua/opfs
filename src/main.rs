#[macro_use]
extern crate clap;

use clap::Arg;
use opfs::block::func::u8_slice_as_superblock;
use opfs::file::*;

fn main() {
    let matches = app_from_crate!()
        .arg(
            Arg::with_name("PATH")
                .help("image file path to manipulate")
                .required(true)
                .index(1),
        )
        .get_matches();
    let path = matches.value_of("PATH").unwrap();
    let file_size = match get_file_size(path) {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    };
    let file = match open_readable_and_writable_file(path) {
        Ok(f) => f,
        Err(e) => panic!("{}", e),
    };
    let m;
    unsafe {
        m = match get_memory_mapped_file(&file, file_size) {
            Ok(m) => m,
            Err(e) => panic!("{}", e),
        };
    };
    println!("{:?}", m);
    println!("{:?}", u8_slice_as_superblock(&*m));
}
