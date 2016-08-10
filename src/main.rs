#[macro_use(defer)]
extern crate scopeguard;
extern crate docopt;
extern crate rustc_serialize;

use docopt::Docopt;
use std::error::Error;
use std::process;

mod ghqup;

const USAGE: &'static str = "
Usage:
  ghqup [--github] [--bitbucket] [--all] [--quiet] [--retry <count>]
  ghqup [--help]
  
Options:
  -h, --help                    Show this screen.
  -g, --github                  Update GitHub repos.
  -b, --bitbucket               Update Bitbucket repos.
  -a, --all                     Update All repos.
  -q, --quiet                   Quiet mode.
  -r <count>, --retry <count>   Retry count [default: 5].
";

#[derive(Debug, RustcDecodable, Clone)]
pub struct Args {
    flag_github: bool,
    flag_bitbucket: bool,
    flag_all: bool,
    flag_quiet: bool,
    flag_retry: u32,
}


fn main() {
    let res = process::Command::new("ghq")
        .arg("root")
        .output();

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    match res {
        Ok(output) => {
            match String::from_utf8(output.stdout) {
                Ok(root) => {
                    let ghqup = ghqup::Ghqup::new(root.trim_right(), args);
                    ghqup.run();
                }
                Err(err) => panic!("{}", err.description()),
            };

        }
        Err(_) => {
            println!("failed to run ghq.");
            std::process::exit(1);
        }
    };
}
