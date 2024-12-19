#![allow(unused_imports, dead_code)]
use std::{
    fs,
    path::{Path, PathBuf},
};

use directories::BaseDirs;
use lazy_static::lazy_static;
use std::sync::Mutex;

use crate::{
    Database,
    core::sql::{SQL_DATABASE, SQL_FILE},
};

pub mod core;

lazy_static! {
    pub static ref GIT_DIR: Mutex<String> = Mutex::new(String::from("/home/Code/"));
    pub static ref GIT_AUTHOR: Mutex<String> = Mutex::new(String::from("Spyder337"));
    pub static ref GIT_EMAIL: Mutex<String> = Mutex::new(String::from("owsley.wood@gmail.com"));
}

/// # Summary
/// Returns the git directory, author, and email stored in the database.
pub fn get_git_info(db: &Database) -> (Box<Path>, String, String) {
    let g_path = get_git_dir_db(db);
    let author = get_git_author_db(db);
    let email = get_git_email_db(db);
    return (g_path, author, email);
}

/// # Summary
/// Prints the git_dir, author, and email to the console.
pub fn display_git_info() {
    let db = crate::Database::new(SQL_DATABASE.to_str().unwrap()).unwrap();
    let _ = db.create_table();
    let (g_path, author, email) = get_git_info(&db);
    println!(
        "Path: {}\nAuthor: {}\nEmail: {}",
        g_path.display(),
        author,
        email
    );
}

fn get_git_dir_db(db: &Database) -> Box<Path> {
    let dir = db
        .get_item_by_var("GIT_DIR")
        .unwrap()
        .val
        .unwrap_or(String::from("~/Code"));
    return Box::from(Path::new(&dir));
}

/// # Summary
/// Returns the git directory stored in the database.
/// The path is returned in a cannonicalized form.
///
/// If there is no entry in the database then a default value of
/// `~/user/Code` is returned.
pub fn get_git_dir() -> Box<Path> {
    let db = crate::Database::new(SQL_DATABASE.to_str().unwrap()).unwrap();
    let mut dir = get_git_dir_db(&db);
    if dir.starts_with("~") {
        let home = BaseDirs::new().unwrap();
        home.home_dir();
        dir = PathBuf::from(
            dir.to_str()
                .unwrap()
                .replace("~", home.home_dir().to_str().unwrap()),
        )
        .into_boxed_path();
    }
    // let dir = fs::canonicalize(dir).unwrap().into_boxed_path();
    // println!("Dir: {:#?}", dir);
    dir
}

/// # Summary
/// If the directory is successfully set then the function returns true.
pub fn set_git_dir(path: &str) -> bool {
    let db = crate::Database::new(SQL_DATABASE.to_str().unwrap()).unwrap();
    let res = db.insert_or_update_item("GIT_DIR", Some(path));
    if res.is_ok() {
        *GIT_DIR.lock().unwrap() = path.to_string();
        let _ = db.update_database_file();
        return true;
    }
    false
}

fn get_git_author_db(db: &Database) -> String {
    return db
        .get_item_by_var("GIT_AUTHOR")
        .unwrap()
        .val
        .unwrap_or(String::from("Author"));
}

/// # Summary
/// Returns the git author stored in the database.
///
/// If there is no entry in the database then a default value of
/// `Author` is returned.
pub fn get_git_author() -> String {
    let db = crate::Database::new(SQL_DATABASE.to_str().unwrap()).unwrap();
    let author = get_git_author_db(&db);
    author
}

/// # Summary
/// If the author is successfully set then the function returns true.
pub fn set_git_author(author: &str) -> bool {
    let db = crate::Database::new(SQL_DATABASE.to_str().unwrap()).unwrap();
    let res = db.insert_or_update_item("GIT_AUTHOR", Some(author));
    if res.is_ok() {
        *GIT_AUTHOR.lock().unwrap() = author.to_string();
        let _ = db.update_database_file();
        return true;
    }
    false
}

fn get_git_email_db(db: &Database) -> String {
    return db
        .get_item_by_var("GIT_EMAIL")
        .unwrap()
        .val
        .unwrap_or(String::from("user.name@email.com"));
}

/// # Summary
/// Returns the git author stored in the database.
///
/// If there is no entry in the database then a default value of
/// `Author` is returned.
pub fn get_git_email() -> String {
    let db = crate::Database::new(SQL_DATABASE.to_str().unwrap()).unwrap();
    let email = get_git_email_db(&db);
    email
}

/// # Summary
/// If the email is successfully set then the function returns true.
pub fn set_git_email(email: &str) -> bool {
    let db = crate::Database::new(SQL_DATABASE.to_str().unwrap()).unwrap();
    let res = db.insert_or_update_item("GIT_EMAIL", Some(email));

    if let Ok(_updated) = res {
        *GIT_EMAIL.lock().unwrap() = email.to_string();
        let _ = db.update_database_file();
        println!("Email set to: {}", GIT_EMAIL.lock().unwrap());
        true
    } else {
        false
    }
}

fn get_git_ignore_url_db(db: &Database) -> String {
    return db
        .get_item_by_var("GIT_IGNORE_URL")
        .unwrap()
        .val
        .unwrap_or(String::from("https://www.toptal.com/developers/gitignore"));
}

pub fn get_git_ignore_url() -> String {
    let db = Database::new(SQL_DATABASE.to_str().unwrap()).unwrap();
    get_git_author_db(&db)
}
