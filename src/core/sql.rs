use std::fs;

use rusqlite::{Connection, params};

pub const SQL_FILE: &str = "rsrc/database.sql";
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn box_error<T>(err: Option<T>) -> Box<dyn std::error::Error>
where
    T: std::error::Error + 'static,
{
    Box::new(err.unwrap())
}

pub struct Item {
    pub id: i32,
    pub var: String,
    pub val: Option<String>,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Database { conn })
    }

    pub fn create_table(&self) -> Result<()> {
        let text_res = fs::read_to_string(SQL_FILE);

        if let Ok(text) = text_res {
            self.conn.execute_batch(&text)?;
            return Ok(());
        }

        Err(box_error(text_res.err()))
    }

    pub fn insert_item(&self, var: &str, val: Option<&str>) -> Result<usize> {
        let query = self.conn.execute(
            "INSERT OR IGNORE INTO ShellCommanders (VAR, VAL) VALUES (?1, ?2)",
            params![var, val],
        );

        if query.is_err() {
            return Err(box_error(query.err()));
        } else {
            return Ok(query.unwrap());
        }
    }

    pub fn insert_or_update_item(&self, var: &str, val: Option<&str>) -> Result<usize> {
        let query = self.conn.execute(
            "INSERT INTO ShellCommanders (VAR, VAL) VALUES (?1, ?2)
            ON CONFLICT(VAR) DO UPDATE SET VAL = excluded.VAL",
            params![var, val],
        );

        if query.is_err() {
            return Err(box_error(query.err()));
        } else {
            return Ok(query.unwrap());
        }
    }

    pub fn get_item_by_id(&self, id: i32) -> Result<Item> {
        let query = self.conn.query_row(
            "SELECT ID, VAR, VAL FROM ShellCommanders WHERE ID = ?1",
            params![id],
            |row| {
                Ok(Item {
                    id: row.get(0)?,
                    var: row.get(1)?,
                    val: row.get(2)?,
                })
            },
        );

        if query.is_err() {
            return Err(box_error(query.err()));
        } else {
            return Ok(query.unwrap());
        }
    }

    pub fn get_item_by_var(&self, var: &str) -> Result<Item> {
        let query = self.conn.query_row(
            "SELECT ID, VAR, VAL FROM ShellCommanders WHERE VAR = ?1",
            params![var],
            |row| {
                Ok(Item {
                    id: row.get(0)?,
                    var: row.get(1)?,
                    val: row.get(2)?,
                })
            },
        );

        if query.is_err() {
            return Err(box_error(query.err()));
        } else {
            return Ok(query.unwrap());
        }
    }

    pub fn get_all_items(&self) -> Result<Vec<Item>> {
        let mut stmt = self
            .conn
            .prepare("SELECT ID, VAR, VAL FROM ShellCommanders")?;
        let items = stmt.query_map([], |row| {
            Ok(Item {
                id: row.get(0)?,
                var: row.get(1)?,
                val: row.get(2)?,
            })
        })?;

        let mut results = Vec::new();
        for item in items {
            results.push(item?);
        }
        Ok(results)
    }

    pub fn update_item(&self, id: i32, var: &str, val: Option<&str>) -> Result<usize> {
        let query = self.conn.execute(
            "UPDATE ShellCommanders SET VAR = ?1, VAL = ?2 WHERE ID = ?3",
            params![var, val, id],
        );

        if query.is_err() {
            return Err(box_error(query.err()));
        } else {
            return Ok(query.unwrap());
        }
    }

    pub fn update_item_by_var(&self, var: &str, val: Option<&str>) -> Result<usize> {
        let query = self.conn.execute(
            "UPDATE ShellCommanders SET VAL = ?1 WHERE VAR = ?2",
            params![val, var],
        );

        if query.is_err() {
            return Err(box_error(query.err()));
        } else {
            return Ok(query.unwrap());
        }
    }

    pub fn delete_item(&self, id: i32) -> Result<usize> {
        let query = self
            .conn
            .execute("DELETE FROM ShellCommanders WHERE ID = ?1", params![id]);

        if query.is_err() {
            return Err(box_error(query.err()));
        } else {
            return Ok(query.unwrap());
        }
    }

    pub fn item_exists(&self, id: i32) -> Result<bool> {
        let exists: bool = self.conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM ShellCommanders WHERE ID = ?1)",
            params![id],
            |row| row.get(0),
        )?;
        Ok(exists)
    }

    pub fn item_exists_by_var(&self, var: &str) -> Result<bool> {
        let exists: bool = self.conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM ShellCommanders WHERE VAR = ?1)",
            params![var],
            |row| row.get(0),
        )?;
        Ok(exists)
    }

    pub fn update_database_file(&self) -> Result<usize> {
        let statement = self.conn.execute("VACUUM", []);

        if statement.is_err() {
            return Err(box_error(statement.err()));
        } else {
            return Ok(statement.unwrap());
        }
    }
}
