pub mod schema;
pub mod settings_repo;
pub mod picklist_repo;
pub mod log_repo;
pub mod project_repo;
pub mod project_link_repo;

use rusqlite::{Connection, Result};

pub fn open(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
    schema::migrate(&conn)?;
    Ok(conn)
}
