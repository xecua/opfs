#[macro_use]
extern crate clap;

use clap::{AppSettings, Arg, SubCommand};
use opfs::block::sblock;
use opfs::file::*;
use opfs::subcommand;

fn main() {
    let matches = app_from_crate!()
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("img_file")
                .help("path to image file to manipulate")
                .required(true)
                .index(1),
        )
        .subcommand(
            SubCommand::with_name("ls")
                .about("list directory contents")
                .arg(
                    Arg::with_name("path")
                        .help("path to look up")
                        .required(true)
                        .index(1),
                ),
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
    let super_block = sblock::u8_slice_as_superblock(&m);
    sblock::check_magic_number(&super_block);

    if let Some(ref matches) = matches.subcommand_matches("ls") {
        let path = matches.value_of("path").unwrap();
        subcommand::ls(&m, &path, &super_block);
    }
}
