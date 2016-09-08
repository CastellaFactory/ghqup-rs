extern crate num_cpus;

use Args;

use scoped_pool::Pool;

use std::ffi::OsStr;
use std::io::{self, BufReader, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Output};

#[derive(Clone)]
pub struct Ghqup {
    root: PathBuf,
    args: Args,
}

impl Ghqup {
    pub fn new<S: AsRef<OsStr> + ?Sized>(path: &S, args: Args) -> Ghqup {
        Ghqup {
            root: PathBuf::from(path),
            args: args,
        }
    }

    pub fn run(&self) {
        let res = process::Command::new("ghq")
            .arg("list")
            .output()
            .expect("failed to execute process 'ghq list'");

        let reader = BufReader::new(&res.stdout[..]);

        let pool = Pool::new(num_cpus::get());

        pool.scoped(move |scoped| {
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        let g = self.clone();
                        scoped.execute(move || {
                            g.exec_repo_loop(line);
                        });

                    }
                    Err(_) => continue,
                }
            }
        });
    }

    fn exec_repo_loop(&self, repo: String) {
        self.update(repo, 0);
    }

    fn update(&self, repo: String, count: u32) {
        let res = process::Command::new("git")
            .arg("pull")
            .current_dir(self.root.join(&repo))
            .output()
            .expect("failed to execute process 'git pull'");

        if res.status.success() {
            self.print_done_result(res, repo);
        } else if count < self.args.retry {
            self.update(repo, count + 1);
        } else {
            self.print_error_result(res, repo);
        }
    }

    fn print_done_result(&self, res: Output, repo: String) {
        let stdout = io::stdout();
        let mut lock = stdout.lock();
        let _ = writeln!(lock, "{}: \x1b[33mDone\x1b[0m", Path::new(&repo).display());
        self.print_output(&mut lock, res.stdout);
    }

    fn print_error_result(&self, res: Output, repo: String) {
        let stderr = io::stderr();
        let mut lock = stderr.lock();
        let _ = writeln!(lock, "{}: \x1b[31mError\x1b[0m", Path::new(&repo).display());
        self.print_output(&mut lock, res.stderr);
    }

    fn print_output<T: Write>(&self, lock: &mut T, v: Vec<u8>) {
        if !self.args.quiet {
            match String::from_utf8(v) {
                Ok(s) => writeln!(lock, "{}", s).unwrap(),
                _ => {}
            };
        }
    }
}
