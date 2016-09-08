#[macro_use]
extern crate clap;
extern crate scoped_pool;

use clap::{Arg, App};

use std::error::Error;
use std::process;

mod ghqup;

#[derive(Debug, Clone)]
pub struct Args {
    quiet: bool,
    retry: u32,
}

fn main() {
    let res = process::Command::new("ghq")
        .arg("root")
        .output()
        .expect("failed to execute ghq root");

    let m = App::new("ghqup")
        .version(crate_version!())
        .version_short("v")
        .arg(Arg::with_name("QUIUT")
            .long("quiet")
            .short("q")
            .help("Quiet mode"))
        .arg(Arg::with_name("RETRY")
            .long("retry")
            .short("r")
            .takes_value(true)
            .empty_values(false)
            .value_name("RETRY")
            .help("Retry count [default: 3]"))
        .get_matches();

    let args = Args {
        quiet: m.is_present("QUIET"),
        retry: value_t!(m, "RETRY", u32).unwrap_or(3),
    };

    match String::from_utf8(res.stdout) {
        Ok(root) => {
            ghqup::Ghqup::new(root.trim_right(), args).run();
        }
        Err(err) => panic!("{}", err.description()),
    }
}
