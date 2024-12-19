use core::str;
use std::path::Path;

use git2::{
    Error, ErrorCode, IndexAddOption, Oid, Repository, Signature, StatusOptions, SubmoduleIgnore,
};

use super::{GIT_AUTHOR, GIT_EMAIL, get_git_author, get_git_email};

///
///
/// Code Sourced from https://github.com/rust-lang/git2-rs/blob/master/examples/status.rs.
///
///
#[derive(Eq, PartialEq)]
enum Format {
    Long,
    Short,
    Porcelain,
}

fn show_branch(repo: &Repository, format: &Format) -> Result<(), Error> {
    let head = match repo.head() {
        Ok(head) => Some(head),
        Err(ref e) if e.code() == ErrorCode::UnbornBranch || e.code() == ErrorCode::NotFound => {
            None
        }
        Err(e) => return Err(e),
    };
    let head = head.as_ref().and_then(|h| h.shorthand());

    if format == &Format::Long {
        println!(
            "# On branch {}",
            head.unwrap_or("Not currently on any branch")
        );
    } else {
        println!("## {}", head.unwrap_or("HEAD (no branch)"));
    }
    Ok(())
}

fn print_submodules(repo: &Repository) -> Result<(), Error> {
    let modules = repo.submodules()?;
    println!("# Submodules");
    for sm in &modules {
        println!(
            "# - submodule '{}' at {}",
            sm.name().unwrap(),
            sm.path().display()
        );
    }
    Ok(())
}

// This function print out an output similar to git's status command in long
// form, including the command-line hints.
fn print_long(statuses: &git2::Statuses) {
    let mut header = false;
    let mut rm_in_workdir = false;
    let mut changes_in_index = false;
    let mut changed_in_workdir = false;

    // Print index changes
    for entry in statuses
        .iter()
        .filter(|e| e.status() != git2::Status::CURRENT)
    {
        if entry.status().contains(git2::Status::WT_DELETED) {
            rm_in_workdir = true;
        }
        let istatus = match entry.status() {
            s if s.contains(git2::Status::INDEX_NEW) => "new file: ",
            s if s.contains(git2::Status::INDEX_MODIFIED) => "modified: ",
            s if s.contains(git2::Status::INDEX_DELETED) => "deleted: ",
            s if s.contains(git2::Status::INDEX_RENAMED) => "renamed: ",
            s if s.contains(git2::Status::INDEX_TYPECHANGE) => "typechange:",
            _ => continue,
        };
        if !header {
            println!(
                "\
# Changes to be committed:
#   (use \"git reset HEAD <file>...\" to unstage)
#"
            );
            header = true;
        }

        let old_path = entry.head_to_index().unwrap().old_file().path();
        let new_path = entry.head_to_index().unwrap().new_file().path();
        match (old_path, new_path) {
            (Some(old), Some(new)) if old != new => {
                println!("#\t{}  {} -> {}", istatus, old.display(), new.display());
            }
            (old, new) => {
                println!("#\t{}  {}", istatus, old.or(new).unwrap().display());
            }
        }
    }

    if header {
        changes_in_index = true;
        println!("#");
    }
    header = false;

    // Print workdir changes to tracked files
    for entry in statuses.iter() {
        // With `Status::OPT_INCLUDE_UNMODIFIED` (not used in this example)
        // `index_to_workdir` may not be `None` even if there are no differences,
        // in which case it will be a `Delta::Unmodified`.
        if entry.status() == git2::Status::CURRENT || entry.index_to_workdir().is_none() {
            continue;
        }

        let istatus = match entry.status() {
            s if s.contains(git2::Status::WT_MODIFIED) => "modified: ",
            s if s.contains(git2::Status::WT_DELETED) => "deleted: ",
            s if s.contains(git2::Status::WT_RENAMED) => "renamed: ",
            s if s.contains(git2::Status::WT_TYPECHANGE) => "typechange:",
            _ => continue,
        };

        if !header {
            println!(
                "\
# Changes not staged for commit:
#   (use \"git add{} <file>...\" to update what will be committed)
#   (use \"git checkout -- <file>...\" to discard changes in working directory)
#\
                ",
                if rm_in_workdir { "/rm" } else { "" }
            );
            header = true;
        }

        let old_path = entry.index_to_workdir().unwrap().old_file().path();
        let new_path = entry.index_to_workdir().unwrap().new_file().path();
        match (old_path, new_path) {
            (Some(old), Some(new)) if old != new => {
                println!("#\t{}  {} -> {}", istatus, old.display(), new.display());
            }
            (old, new) => {
                println!("#\t{}  {}", istatus, old.or(new).unwrap().display());
            }
        }
    }

    if header {
        changed_in_workdir = true;
        println!("#");
    }
    header = false;

    // Print untracked files
    for entry in statuses
        .iter()
        .filter(|e| e.status() == git2::Status::WT_NEW)
    {
        if !header {
            println!(
                "\
# Untracked files
#   (use \"git add <file>...\" to include in what will be committed)
#"
            );
            header = true;
        }
        let file = entry.index_to_workdir().unwrap().old_file().path().unwrap();
        println!("#\t{}", file.display());
    }
    header = false;

    // Print ignored files
    for entry in statuses
        .iter()
        .filter(|e| e.status() == git2::Status::IGNORED)
    {
        if !header {
            println!(
                "\
# Ignored files
#   (use \"git add -f <file>...\" to include in what will be committed)
#"
            );
            header = true;
        }
        let file = entry.index_to_workdir().unwrap().old_file().path().unwrap();
        println!("#\t{}", file.display());
    }

    if !changes_in_index && changed_in_workdir {
        println!(
            "no changes added to commit (use \"git add\" and/or \
             \"git commit -a\")"
        );
    }
}

// This version of the output prefixes each path with two status columns and
// shows submodule status information.
pub fn print_short(repo: &Repository, statuses: &git2::Statuses) {
    for entry in statuses
        .iter()
        .filter(|e| e.status() != git2::Status::CURRENT)
    {
        let mut istatus = match entry.status() {
            s if s.contains(git2::Status::INDEX_NEW) => 'A',
            s if s.contains(git2::Status::INDEX_MODIFIED) => 'M',
            s if s.contains(git2::Status::INDEX_DELETED) => 'D',
            s if s.contains(git2::Status::INDEX_RENAMED) => 'R',
            s if s.contains(git2::Status::INDEX_TYPECHANGE) => 'T',
            _ => ' ',
        };
        let mut wstatus = match entry.status() {
            s if s.contains(git2::Status::WT_NEW) => {
                if istatus == ' ' {
                    istatus = '?';
                }
                '?'
            }
            s if s.contains(git2::Status::WT_MODIFIED) => 'M',
            s if s.contains(git2::Status::WT_DELETED) => 'D',
            s if s.contains(git2::Status::WT_RENAMED) => 'R',
            s if s.contains(git2::Status::WT_TYPECHANGE) => 'T',
            _ => ' ',
        };

        if entry.status().contains(git2::Status::IGNORED) {
            istatus = '!';
            wstatus = '!';
        }
        if istatus == '?' && wstatus == '?' {
            continue;
        }
        let mut extra = "";

        // A commit in a tree is how submodules are stored, so let's go take a
        // look at its status.
        //
        // TODO: check for GIT_FILEMODE_COMMIT
        let status = entry.index_to_workdir().and_then(|diff| {
            let ignore = SubmoduleIgnore::Unspecified;
            diff.new_file()
                .path_bytes()
                .and_then(|s| str::from_utf8(s).ok())
                .and_then(|name| repo.submodule_status(name, ignore).ok())
        });
        if let Some(status) = status {
            if status.contains(git2::SubmoduleStatus::WD_MODIFIED) {
                extra = " (new commits)";
            } else if status.contains(git2::SubmoduleStatus::WD_INDEX_MODIFIED)
                || status.contains(git2::SubmoduleStatus::WD_WD_MODIFIED)
            {
                extra = " (modified content)";
            } else if status.contains(git2::SubmoduleStatus::WD_UNTRACKED) {
                extra = " (untracked content)";
            }
        }

        let (mut a, mut b, mut c) = (None, None, None);
        if let Some(diff) = entry.head_to_index() {
            a = diff.old_file().path();
            b = diff.new_file().path();
        }
        if let Some(diff) = entry.index_to_workdir() {
            a = a.or_else(|| diff.old_file().path());
            b = b.or_else(|| diff.old_file().path());
            c = diff.new_file().path();
        }

        match (istatus, wstatus) {
            ('R', 'R') => println!(
                "RR {} {} {}{}",
                a.unwrap().display(),
                b.unwrap().display(),
                c.unwrap().display(),
                extra
            ),
            ('R', w) => println!(
                "R{} {} {}{}",
                w,
                a.unwrap().display(),
                b.unwrap().display(),
                extra
            ),
            (i, 'R') => println!(
                "{}R {} {}{}",
                i,
                a.unwrap().display(),
                c.unwrap().display(),
                extra
            ),
            (i, w) => println!("{}{} {}{}", i, w, a.unwrap().display(), extra),
        }
    }

    for entry in statuses
        .iter()
        .filter(|e| e.status() == git2::Status::WT_NEW)
    {
        println!(
            "?? {}",
            entry
                .index_to_workdir()
                .unwrap()
                .old_file()
                .path()
                .unwrap()
                .display()
        );
    }
}
// This version of the output prefixes each path with two status columns and
// shows submodule status information.
pub fn message_short(repo: &Repository, statuses: &git2::Statuses) -> String {
    let mut msg: String = String::with_capacity(256);
    for entry in statuses
        .iter()
        .filter(|e| e.status() != git2::Status::CURRENT)
    {
        let mut istatus = match entry.status() {
            s if s.contains(git2::Status::INDEX_NEW) => 'A',
            s if s.contains(git2::Status::INDEX_MODIFIED) => 'M',
            s if s.contains(git2::Status::INDEX_DELETED) => 'D',
            s if s.contains(git2::Status::INDEX_RENAMED) => 'R',
            s if s.contains(git2::Status::INDEX_TYPECHANGE) => 'T',
            _ => ' ',
        };
        let mut wstatus = match entry.status() {
            s if s.contains(git2::Status::WT_NEW) => {
                if istatus == ' ' {
                    istatus = '?';
                }
                '?'
            }
            s if s.contains(git2::Status::WT_MODIFIED) => 'M',
            s if s.contains(git2::Status::WT_DELETED) => 'D',
            s if s.contains(git2::Status::WT_RENAMED) => 'R',
            s if s.contains(git2::Status::WT_TYPECHANGE) => 'T',
            _ => ' ',
        };

        if entry.status().contains(git2::Status::IGNORED) {
            istatus = '!';
            wstatus = '!';
        }
        if istatus == '?' && wstatus == '?' {
            continue;
        }
        let mut extra = "";

        // A commit in a tree is how submodules are stored, so let's go take a
        // look at its status.
        //
        // TODO: check for GIT_FILEMODE_COMMIT
        let status = entry.index_to_workdir().and_then(|diff| {
            let ignore = SubmoduleIgnore::Unspecified;
            diff.new_file()
                .path_bytes()
                .and_then(|s| str::from_utf8(s).ok())
                .and_then(|name| repo.submodule_status(name, ignore).ok())
        });
        if let Some(status) = status {
            if status.contains(git2::SubmoduleStatus::WD_MODIFIED) {
                extra = " (new commits)";
            } else if status.contains(git2::SubmoduleStatus::WD_INDEX_MODIFIED)
                || status.contains(git2::SubmoduleStatus::WD_WD_MODIFIED)
            {
                extra = " (modified content)";
            } else if status.contains(git2::SubmoduleStatus::WD_UNTRACKED) {
                extra = " (untracked content)";
            }
        }

        let (mut a, mut b, mut c) = (None, None, None);
        if let Some(diff) = entry.head_to_index() {
            a = diff.old_file().path();
            b = diff.new_file().path();
        }
        if let Some(diff) = entry.index_to_workdir() {
            a = a.or_else(|| diff.old_file().path());
            b = b.or_else(|| diff.old_file().path());
            c = diff.new_file().path();
        }

        let m = match (istatus, wstatus) {
            ('R', 'R') => format!(
                "RR {} {} {}{}",
                a.unwrap().display(),
                b.unwrap().display(),
                c.unwrap().display(),
                extra
            ),
            ('R', w) => {
                format!(
                    "R{} {} {}{}",
                    w,
                    a.unwrap().display(),
                    b.unwrap().display(),
                    extra
                )
            }
            (i, 'R') => {
                format!(
                    "{}R {} {}{}",
                    i,
                    a.unwrap().display(),
                    c.unwrap().display(),
                    extra
                )
            }
            (i, w) => format!("{}{} {}{}", i, w, a.unwrap().display(), extra),
        };

        msg.push_str(&format!("{}\n", m));
    }

    for entry in statuses
        .iter()
        .filter(|e| e.status() == git2::Status::WT_NEW)
    {
        msg.push_str(&format!(
            "?? {}",
            entry
                .index_to_workdir()
                .unwrap()
                .old_file()
                .path()
                .unwrap()
                .display()
        ));
    }
    msg
}
///
///
/// End of Sourced code https://github.com/rust-lang/git2-rs/blob/master/examples/status.rs
///
///

/// Add files to the git repository.
/// paths: Path specs.
pub fn add_files(repo: &mut Repository, paths: &Vec<String>, update: Option<bool>) {
    let mut index = repo.index().unwrap();
    let is_update = update.unwrap_or(false);

    //  Paths are passed into update_all or add_all and those paths
    //  are filtered by this function.
    //  0 is returned if the path is added to the index.
    //  1 is returned if the path is not added to the index.
    //  0< is returned if an error occurs.
    let cb = &mut |path: &Path, _matched_spec: &[u8]| -> i32 {
        let status = repo.status_file(path).unwrap();
        let ret = if status.contains(git2::Status::WT_MODIFIED)
            || status.contains(git2::Status::WT_NEW)
        {
            0
        } else {
            1
        };
        ret
    };

    let cb = if !is_update {
        None
    } else {
        Some(cb as &mut git2::IndexMatchedPath)
    };

    if is_update {
        index.update_all(paths, cb).unwrap();
    } else {
        //  TODO: Implement Error handling for adding files.
        let _ = index.add_all(paths, git2::IndexAddOption::DEFAULT, cb);
    }

    index.write().unwrap();
}

pub fn create_commit(repo: &Repository, commit_msg: String) -> Result<Oid, git2::Error> {
    // Get the index and write it as a tree
    let mut index = repo.index()?;
    // index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
    // index.write()?;
    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;

    // Get the HEAD reference and its commit
    let head = repo.head()?;
    let parent_commit = head.peel_to_commit()?;

    // Create a signature
    let sig = Signature::now(&get_git_author(), &get_git_email())?;

    // Create the commit
    let commit_oid = repo.commit(
        Some("HEAD"),      // the name of the reference to update
        &sig,              // the author signature
        &sig,              // the committer signature
        &commit_msg,       // the commit message
        &tree,             // the tree object this commit points to
        &[&parent_commit], // parents of the commit
    )?;

    Ok(commit_oid)
}
