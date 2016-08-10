extern crate wait_group;
use self::wait_group::WaitGroup;

use std::ffi::OsStr;
use std::fs;
use std::io::StdoutLock;
use std::io::Write;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::str;
use std::thread;
use std::vec::Vec;
use super::Args;

#[derive(Clone)]
pub struct Ghqup {
    root: PathBuf,
    args: Args,
    wg: WaitGroup,
}

impl Ghqup {
    pub fn new<S: AsRef<OsStr> + ?Sized>(path: &S, args: Args) -> Ghqup {
        Ghqup {
            root: PathBuf::from(path),
            args: args,
            wg: WaitGroup::new(),
        }
    }

    pub fn run(&self) {
        if self.args.flag_github || self.args.flag_all {
            self.exec_user_name_loop("github.com");
        }
        if self.args.flag_bitbucket || self.args.flag_all {
            self.exec_user_name_loop("bitbucket.org");
        }
    }

    fn exec_user_name_loop(&self, repo_type: &str) {
        match fs::read_dir(self.root.join(repo_type)) {
            Ok(files) => {
                for file in files.into_iter().filter_map(|e| e.ok()) {
                    match file.metadata() {
                        Ok(metadata) => {
                            if metadata.is_dir() {
                                match file.path()
                                    .file_name()
                                    .and_then(|name| name.to_str()) {
                                    Some(username) => {
                                        let n_self = self.clone();
                                        let repo_type = repo_type.to_owned();
                                        let username = username.to_owned();
                                        self.wg.add(1);
                                        thread::spawn(move || {
                                            n_self.exec_repo_loop(repo_type, username);
                                        });
                                    }
                                    None => return,
                                };
                            }
                        }
                        Err(err) => {
                            println!("{}", err.to_string());
                            return;
                        }
                    }
                }
            }
            Err(err) => {
                println!("{}: {}",
                         self.root.join(repo_type).to_str().unwrap_or(""),
                         err.to_string());
            }
        }
        self.wg.wait();
    }

    fn exec_repo_loop(&self, repo_type: String, username: String) {
        defer!(self.wg.done());
        match fs::read_dir(self.root.join(&repo_type).join(&username)) {
            Ok(files) => {
                for file in files.into_iter().filter_map(|e| e.ok()) {
                    match file.metadata() {
                        Ok(metadata) => {
                            if metadata.is_dir() {
                                match file.path()
                                    .file_name()
                                    .and_then(|name| name.to_str()) {
                                    Some(repo) => {
                                        let n_self = self.clone();
                                        let repo_type = repo_type.clone();
                                        let username = username.clone();
                                        let repo = repo.to_owned();
                                        self.wg.add(1);
                                        thread::spawn(move || {
                                            n_self.update(repo_type, username, repo, 0);
                                        });
                                    }
                                    None => {
                                        return;
                                    }
                                };
                            }
                        }
                        Err(err) => {
                            println!("{}", err.to_string());
                            return;
                        }
                    };
                }
            }
            Err(err) => {
                println!("{}: {}",
                         self.root.join(repo_type).to_str().unwrap_or(""),
                         err.to_string());
            }
        }
    }


    fn update(&self, repo_type: String, username: String, repo: String, count: u32) {
        defer!(self.wg.done());
        let res = process::Command::new("git")
            .arg("pull")
            .current_dir(self.root.join(&repo_type).join(&username).join(&repo))
            .output()
            .expect("failed to execute process 'git pull'");

        if res.status.success() {
            let stdout = io::stdout();
            let mut lock = stdout.lock();
            writeln!(lock,
                     "{}: \x1b[33mDone\x1b[0m",
                     Path::new(&repo_type).join(&username).join(&repo).display())
                .unwrap();
            if !self.args.flag_quiet {
                print_output(&mut lock, res.stdout);
            }
        } else if count < self.args.flag_retry {
            self.update(repo_type, username, repo, count + 1);
            return;
        } else {
            let stdout = io::stdout();
            let mut lock = stdout.lock();
            writeln!(lock,
                     "{}: \x1b[31mError\x1b[0m",
                     Path::new(&repo_type).join(&username).join(&repo).display())
                .unwrap();
            if !self.args.flag_quiet {
                print_output(&mut lock, res.stdout);
            }
        }
    }
}

fn print_output(lock: &mut StdoutLock, v: Vec<u8>) {
    match String::from_utf8(v) {
        Ok(s) => writeln!(lock, "{}", s).unwrap(),
        _ => {}
    };
}
