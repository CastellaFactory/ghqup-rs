use std::sync::Mutex;
use std::thread;
use std::sync::mpsc;
use std::env;
use std::process;
use std::path::Path;
use std::fs;
use std::error::Error;

fn update(root: &Path, repo_type: &String, username: &String, repo: &String, count: u32) {
    let res = process::Command::new("git")
        .arg("pull")
        .current_dir(root.join(repo_type).join(username).join(repo))
        .output()
        .expect("failed");
    println!("{}/{}/{}: Done", repo_type, username, repo);
    println!("{:?}", String::from_utf8(res.stdout));
}

fn exec_repo_loop(root: &Path, repo_type: &String, username: &String) {
    match fs::read_dir(root.join(repo_type).join(username)) {
        Ok(paths) => {
            for path in paths {
                // println!("{}", path.unwrap().path().to_str().unwrap());
                let repo = path.unwrap().path().file_name().unwrap().to_str().unwrap().to_string();
                update(&root, &repo_type, &username, &repo, 0);
            }
        }
        Err(err) => {
            println!("{}: {}",
                     root.join(repo_type).to_str().unwrap(),
                     err.description());
        }
    }
}

fn exec_user_name_loop(root: &Path, repo_type: &String) {
    match fs::read_dir(root.join(repo_type)) {
        Ok(paths) => {
            for path in paths {
                let username =
                    path.unwrap().path().file_name().unwrap().to_str().unwrap().to_string();
                exec_repo_loop(&root, &repo_type, &username);
            }
        }
        Err(err) => {
            println!("{}: {}",
                     root.join(repo_type).to_str().unwrap(),
                     err.description());
        }
    }
}

fn main() {
    let res = process::Command::new("ghq")
        .arg("root")
        .output();

    match res {
        Ok(output) => {
            match String::from_utf8(output.stdout) {
                Ok(root) => {
                    let mut path = Path::new(root.trim_right());
                    // exec_user_name_loop(path, &"github.com".to_owned());
                    exec_user_name_loop(path, &"bitbucket.org".to_owned());
                }
                Err(err) => panic!("{:?}", err),
            };

        }
        Err(_) => {
            println!("failed to run ghq");
            std::process::exit(1);
        }
    };
}
