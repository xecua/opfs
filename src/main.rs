#[macro_use]
extern crate clap;

use clap::Arg;
use opfs::block::sblock;
use opfs::file::*;

fn main() {
    let matches = app_from_crate!()
        .arg(
            Arg::with_name("img_file")
                .help("path to image file to manipulate")
                .required(true)
                .index(1),
        )
        .get_matches();
    let path = matches.value_of("img_file").unwrap();
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
    let _s = sblock::check_magic_number(&m);
    println!("ok");
}
