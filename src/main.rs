#[macro_use]
extern crate clap;

use clap::{AppSettings, Arg, SubCommand};
use opfs::initializer::initialize;
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
    let (img, sblock) = initialize(path);

    if let Some(ref matches) = matches.subcommand_matches("ls") {
        let path = matches.value_of("path").unwrap();
        subcommand::ls(img, path, sblock);
    }
}
