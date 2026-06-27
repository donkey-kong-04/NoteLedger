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
    backup_if_outdated(path, &conn);
    schema::migrate(&conn)?;
    Ok(conn)
}

/// Before running migrations on an existing database, take a timestamped copy of
/// the file. Best-effort: a failure here logs a warning but does not block the
/// app. A brand-new database (version 0) or an already-current one is skipped.
fn backup_if_outdated(path: &str, conn: &Connection) {
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap_or(0);
    if version == 0 || version >= schema::LATEST_VERSION {
        return;
    }
    // Flush the WAL into the main file so the copy is complete.
    let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let backup = format!("{}.backup-v{}-{}", path, version, ts);
    match std::fs::copy(path, &backup) {
        Ok(_) => eprintln!(
            "[note-ledger] migrating DB v{}\u{2192}v{}: backup saved to {}",
            version, schema::LATEST_VERSION, backup
        ),
        Err(e) => eprintln!(
            "[note-ledger] WARNING: could not back up DB before migration: {}",
            e
        ),
    }
}

#[cfg(test)]
mod tests;
