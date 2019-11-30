#[macro_use]
extern crate clap;

use clap::{AppSettings, Arg, SubCommand};
use opfs::block::sblock;
use opfs::file::*;
use opfs::subcommand;
use std::process::exit;

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
                        .help("path to file to look up")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("extract file")
                .arg(
                    Arg::with_name("source")
                        .help("path to file to extract")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("destination")
                        .help("destination path of host")
                        .required(true)
                        .index(2),
                ),
        )
        .get_matches();
    let path = matches.value_of("img_file").unwrap();
    let file_size = match get_file_size(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };
    let file = match open_readable_and_writable_file(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };
    let m = match get_memory_mapped_file(&file, file_size) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };
    let sblock = sblock::u8_slice_as_superblock(&m);
    sblock::check_magic_number(&sblock);

    if let Some(ref matches) = matches.subcommand_matches("ls") {
        let path = matches.value_of("path").unwrap();
        subcommand::ls(&m, &path, &sblock);
    } else if let Some(ref matches) = matches.subcommand_matches("get") {
        let src = matches.value_of("source").unwrap();
        let dst = matches.value_of("destination").unwrap();
        subcommand::get(&m, &src, &dst, &sblock);
    }
}
