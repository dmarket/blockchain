extern crate clap;

use clap::{App, Arg};
use super::VERSION;

pub struct Flags {}

pub fn parse() -> Option<Flags> {
    let matches = App::new("DMarket blockchain")
        .arg(Arg::with_name("version")
            .help("show version")
            .short("v")
            .long("version")
            .multiple(false)
        )
        .get_matches();

    match matches.occurrences_of("version") {
        1 => {
            println!("DMarket Blockchain v{}", VERSION);
            None
        }
        _ => return Some(Flags {}),
    }
}
