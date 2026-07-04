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
    backup_if_outdated(path, &conn)?;
    schema::migrate(&conn)?;
    Ok(conn)
}

/// Before running migrations on an existing database, take a timestamped copy of
/// the file and verify it. A failure REFUSES to proceed: the migration is not
/// run and the app does not start, leaving the database untouched. A brand-new
/// database (version 0) or an already-current one is skipped.
fn backup_if_outdated(path: &str, conn: &Connection) -> Result<()> {
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap_or(0);
    if version == 0 || version >= schema::LATEST_VERSION {
        return Ok(());
    }
    // Flush the WAL into the main file so the copy is complete. This must
    // succeed, otherwise the copied file could miss recent writes.
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;

    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let backup = format!("{}.backup-v{}-{}", path, version, ts);
    let fail = |msg: String| {
        rusqlite::Error::InvalidParameterName(format!(
            "Refusing to migrate DB v{}\u{2192}v{}: {}",
            version,
            schema::LATEST_VERSION,
            msg
        ))
    };

    std::fs::copy(path, &backup).map_err(|e| fail(format!("backup copy failed: {}", e)))?;

    // Verify the copy: same size as the source, and openable as a SQLite DB
    // at the expected schema version.
    let src_len = std::fs::metadata(path).map_err(|e| fail(format!("cannot stat source DB: {}", e)))?.len();
    let bak_len = std::fs::metadata(&backup).map_err(|e| fail(format!("cannot stat backup: {}", e)))?.len();
    if src_len != bak_len {
        return Err(fail(format!("backup size mismatch ({} vs {} bytes)", bak_len, src_len)));
    }
    let bak_version: i64 = Connection::open(&backup)
        .and_then(|c| c.query_row("PRAGMA user_version", [], |r| r.get(0)))
        .map_err(|e| fail(format!("backup is not a readable SQLite DB: {}", e)))?;
    if bak_version != version {
        return Err(fail(format!("backup has wrong schema version ({} vs {})", bak_version, version)));
    }

    eprintln!(
        "[note-ledger] migrating DB v{}\u{2192}v{}: verified backup saved to {}",
        version, schema::LATEST_VERSION, backup
    );
    Ok(())
}

#[cfg(test)]
mod tests;
