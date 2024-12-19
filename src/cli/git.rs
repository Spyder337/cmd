use std::{fs, io::Write, path::Path}; // Add this line to import the crate

use clap::{Subcommand, ValueHint, command};
use git2::{Repository, Status, StatusOptions};

use crate::{create_commit, message_short, time_now};

use super::{GIT_AUTHOR, get_git_dir};

/// # Summary
/// Git related commands for the CLI.
///
/// ## Commands
/// Contains the following functions:
/// - Update : Stage changes and commit them.
/// - List : List the repos in the user git directory.
/// - SetDir : Set the current repo directory.
/// - GetDir : Get the current repo directory.
/// - FetchIgnore : Fetches a .gitignore file.  
/// - FetchIgnores : Fetches a list of valid ignore files.
/// - CheatSheet : Git Cheat Sheet
/// - ReadMe : Generate a README.md file.
#[derive(Subcommand, Debug)]
pub enum GitCommands {
    #[command(
        about = "Stage changes and commit them.",
        long_about = r###"Add changes in the current directory or an optional one.
Then generate a change list or take one from the user.
Generate a generic commit message header with timestamps and then combine
that with the optional change list.

If a path is not provided then '.' is used as the path.

If the remove flag is provided then the files matching the path are unstaged."###
    )]
    Update {
        #[arg(short = 'p', long)]
        paths: Option<Vec<String>>,
        #[arg(short = 'r', long)]
        remove: Option<bool>,
        #[arg(short = 'c', long, value_delimiter = ',', num_args = 0..)]
        changes: Option<Vec<String>>,
    },
    #[command(
        about = "List repos in the user git directory.",
        long_about = "List the user repos and the pulled returned as a json object."
    )]
    List {},
    #[command(about = "Set the repo directory.")]
    SetDir {
        #[arg(value_hint = ValueHint::DirPath)]
        path: String,
    },
    #[command(about = "Get the repo directory.")]
    GetDir {},
    #[command(
        about = "Fetch a .gitignore file.",
        long_about = r#"Fetches a .gitignore file from https://github.com/github/gitignore
Common Ignores: 
    - Rust
    - VisualSudio
    
If multiple ignore files are provided then they are appended to the same file.

To get a list of all of the valid ignore names use the fetch-ignores command."#
    )]
    FetchIgnore {
        #[arg(help = "Ignore file names", num_args= 1..,)]
        files: Vec<String>,
        write_file: bool,
    },
    #[command(
        about = "Display valid ignore files.",
        long_about = r###"Fetches ignore files from https://github.com/github/gitignore"###
    )]
    FetchIgnores {
        #[arg(
            short = 's',
            long = "search",
            help = "Search for a specific ignore file."
        )]
        name: Option<String>,
    },
    #[command(
        about = "Create a new repo.",
        long_about = r###"Create a new repo in the user's git directory."###
    )]
    New {
        name: String,
        #[arg(short = 'i', long)]
        ignores: Option<Vec<String>>,
    },
}

pub enum LicenseType {
    MIT,
    Apache,
    GPL,
    LGPL,
    BSD,
    MPL,
    AGPL,
    EPL,
    ISC,
    WTFPL,
    Unlicense,
    Custom,
}

/// Takes in a GitCommand and executes it's function.
pub(crate) fn handle_commands(command: &GitCommands) {
    match command {
        GitCommands::Update {
            paths,
            remove,
            changes,
        } => update_exec(paths, remove, changes),
        GitCommands::List {} => list_exec(),
        GitCommands::SetDir { path } => set_dir_exec(path),
        GitCommands::GetDir {} => {
            let dir = get_dir_exec();
            println!("Path: {}", dir.to_str().unwrap());
        }
        GitCommands::FetchIgnore { files, write_file } => fetch_ignore_exec(&files, *write_file),
        GitCommands::FetchIgnores { name } => fetch_ignores_exec(name),
        GitCommands::New { name, ignores } => {
            let mut ignore_list = Vec::new();
            if ignores.is_some() {
                ignore_list = ignores.clone().unwrap();
            }
            new_repo(name, &ignore_list);
        }
    }
}

fn new_repo(name: &String, ignores: &Vec<String>) {
    let path = get_git_dir()
        .join(Path::new(GIT_AUTHOR.lock().unwrap().as_str()))
        .join(name);
    println!("Creating new repo: {:?}", path);
    //  Create the directory.
    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    let ignore_path = path.join(".gitignore");
    let ignore_txt = fetch_ignores(ignores.as_slice());

    let repo = Repository::init(&path);

    if repo.is_err() {
        println!("Failed to create the repo.\n{:?}", repo.err());
        return;
    }

    let path = repo.unwrap().workdir().unwrap().to_path_buf();

    println!("Repo created at: {:#?}", path);

    let mut ignore_file = fs::File::create(&ignore_path).unwrap();
    let write_res = ignore_file.write(ignore_txt.as_bytes());
    if write_res.is_err() {
        println!("Failed to write to the ignore file.\n{:?}", write_res.err());
    }
}

///  # Example Message
///  ```shell
/// $ cmd git update -p "path/to/stage" -c "Change 1", "Change 2", "Change 3"
///
/// <Main Change>
///
/// <Time Stamp>
///
/// Changes:
/// - Change 1
/// - Change 2
/// - Change 3
///
/// <Files Changed>
/// ```
fn update_exec(
    paths: &Option<Vec<String>>,
    remove: &Option<bool>,
    changes: &Option<Vec<String>>,
) -> () {
    let repo = Repository::open(".");
    if repo.is_err() {
        println!("Was not able to open the repo.\n{:?}", repo.err());
        return;
    }

    let path_specs = paths.clone().unwrap_or(vec![".".to_string()]);

    let r = &mut repo.unwrap();
    let mut _removing = false;
    if let Some(val) = remove {
        _removing = *val;
        if _removing {
            println!("Deleting files from being staged.");
            //  Git Command: git restore --staged $path
        }
        todo!("Implement unstaging files.")
    } else {
        // println!("Staging new files...");

        //  Equivalent to git add --update
        crate::git::core::add_files(r, &path_specs, None);
        //  Git Command: git stage $path
        //  Use `git status -s` to generate an organized change list.
        let mut commit_msg = String::with_capacity(1536);
        let mut status_msg = String::with_capacity(512);
        let mut change_msg = String::with_capacity(512);
        //  If we're staging files then we should generate a commit for the
        //  module too.
        let main_change: String;
        let time = time_now();
        let time_str = time.format("%Y-%m-%d %H:%M:%S");

        //  If the changes are not provided then we generate a timestamp for the
        // first line of the commit message.
        if changes.is_none() {
            main_change = format!("Updated: {}", time_str).to_string();
        } else {
            main_change = changes.as_ref().unwrap().first().unwrap().to_string();
            let change_list = changes.as_ref().unwrap();
            if change_list.len() > 1 {
                for i in 1..change_list.len() {
                    if change_list[i].is_empty() {
                        continue;
                    }
                    change_msg.push_str(&format!("- {}\n", change_list[i]));
                }
            } else {
                change_msg = String::new();
            }
        }

        //  Generate the first line.
        commit_msg.push_str(&main_change);
        commit_msg.push_str("\n\n");

        //  If there are changes then append them.
        if changes.is_some() {
            commit_msg.push_str(format!("Updated: {}\n", time_str).as_str());
            if !change_msg.is_empty() {
                commit_msg.push_str("\nChanges:\n");
                commit_msg.push_str(&change_msg);
            }
        }

        // Generate the status message
        let statuses = r.statuses(Some(&mut StatusOptions::new()));

        for s in statuses.unwrap().iter().map(|a| a) {
            if let Some(p) = s.path() {
                for path in path_specs.clone() {
                    if !p.contains(&path) {
                        continue;
                    }
                }
            }
            match s.status() {
                Status::WT_NEW | Status::INDEX_NEW => {
                    status_msg.push_str(format!("A {}\n", s.path().unwrap()).as_str());
                }
                Status::WT_MODIFIED | Status::INDEX_MODIFIED => {
                    status_msg.push_str(format!("M {}\n", s.path().unwrap()).as_str());
                }
                Status::WT_DELETED | Status::INDEX_DELETED => {
                    status_msg.push_str(format!("D {}\n", s.path().unwrap()).as_str());
                }
                _ => (),
            }
        }

        //  Append the status message.
        commit_msg.push_str("\nFiles Changed:\n");
        commit_msg.push_str(&status_msg);
        println!("Commit Message Generated: \n\n{}", commit_msg);

        let res = create_commit(r, commit_msg);

        if let Ok(_) = res {
            println!("Commit was successful.");
        } else {
            println!("Commit failed.\n{:?}", res.err());
        }
    }
}

pub struct GitRepo {
    pub name: String,
    pub path: String,
    pub owned: bool,
}

fn traverse_dir(dir: Box<Path>, acc: &mut Vec<Box<Path>>) {
    let entries = fs::read_dir(dir).unwrap();
    let git_dir = get_git_dir();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            if path.components().into_iter().any(|x| {
                x.as_os_str() == "target"
                    || x.as_os_str() == "obj"
                    || x.as_os_str() == ".git"
                    || x.as_os_str() == "bin"
            }) {
                continue;
            }

            if path.parent().unwrap().parent().unwrap() == git_dir.to_path_buf() {
                acc.push(entry.path().into_boxed_path());
            }

            traverse_dir(path.into_boxed_path(), acc);
        }
    }
}

fn list_exec() -> () {
    let dir = crate::git::get_git_dir();
    let exists = dir.exists();
    if !exists {
        println!("Directory does not exist.");
        return;
    }
    let mut paths: Vec<Box<Path>> = Vec::new();
    let buff = dir.into_path_buf();
    traverse_dir(buff.into_boxed_path(), &mut paths);
    println!("Listing repos in: {:#?}", get_git_dir());

    println!("Directories:");
    for path in paths {
        println!("{:?}", path);
    }
}

fn set_dir_exec(path: &str) -> () {
    crate::git::set_git_dir(path);
}

fn get_dir_exec() -> Box<Path> {
    crate::git::get_git_dir()
}

fn fetch_ignore_exec(ignores: &[String], write_file: bool) -> () {
    if !write_file {
        fetch_ignores(ignores);
    } else {
        let ignore_txt = fetch_ignores(ignores);
        let path = Path::new(".gitignore");
        let mut ignore_file = fs::File::create(path).unwrap();
        let write_res = ignore_file.write(ignore_txt.as_bytes());
        if write_res.is_err() {
            println!("Failed to write to the ignore file.\n{:?}", write_res.err());
        }
    }
}

fn fetch_ignores(ignores: &[String]) -> String {
    if ignores.is_empty() {
        println!("No ignore files provided.");
        return String::new();
    }
    let url = "https://www.toptal.com/developers/gitignore/api/";
    let full_url = format!("{}{}", url, ignores.join(","));
    let res = reqwest::blocking::get(full_url).unwrap();
    let body = res.text().unwrap();
    body
}

fn get_ignore_list(name: &Option<String>) -> Vec<String> {
    let url = "https://www.toptal.com/developers/gitignore/api/list?format=lines";
    // println!("Url: {}", url);
    let res = reqwest::blocking::get(url).unwrap();
    // println!("{:#?}", res);
    let body = res.text().unwrap();
    if name.is_none() {
        return body.lines().map(|x| x.to_string()).collect();
    } else {
        return body
            .lines()
            .filter(|x| x.contains(name.clone().unwrap().as_str()))
            .map(|x| x.to_string())
            .collect();
    }
}

fn fetch_ignores_exec(name: &Option<String>) -> () {
    let ignores = get_ignore_list(name);
    let mut cnt = 1;
    for ignore in ignores {
        println!("{}: {}", cnt, ignore);
        cnt += 1;
    }
}
