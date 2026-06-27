use rusqlite::{Connection, Result, params};
use crate::models::log::Log;

// The four category junction tables. Table names are compile-time constants
// (never user input), so interpolating them into SQL is safe.
const CAT_TABLES: [&str; 4] = ["log_category_1", "log_category_2", "log_category_3", "log_category_4"];

fn sync_cat(conn: &Connection, table: &str, log_id: i64, ids: &[i64]) -> Result<()> {
    let del = format!("DELETE FROM {} WHERE log_id = ?1", table);
    conn.execute(&del, params![log_id])?;
    let ins = format!("INSERT OR IGNORE INTO {} (log_id, value_id) VALUES (?1, ?2)", table);
    for &vid in ids {
        conn.execute(&ins, params![log_id, vid])?;
    }
    Ok(())
}

pub fn list(conn: &Connection) -> Result<Vec<Log>> {
    let mut stmt = conn.prepare(
        "SELECT id, type_id, title, description, start_date, due_date,
                is_closed, closed_date, project_id
         FROM logs ORDER BY start_date DESC"
    )?;
    let logs: Vec<Log> = stmt.query_map([], |row| {
        Ok(Log {
            id:           row.get(0)?,
            type_id:      row.get(1)?,
            title:        row.get(2)?,
            description:  row.get::<_, Option<String>>(3)?.unwrap_or_default(),
            start_date:   row.get(4)?,
            due_date:     row.get(5)?,
            is_closed:    row.get(6)?,
            closed_date:  row.get(7)?,
            project_id:   row.get(8)?,
            category1_ids: vec![],
            category2_ids: vec![],
            category3_ids: vec![],
            category4_ids: vec![],
        })
    })?.collect::<Result<Vec<_>>>()?;

    if logs.is_empty() { return Ok(logs); }

    // We need every log's categories, so load each junction table in full and
    // group by log_id — no need to build an `IN (...)` list of ids.
    let load_cats = |table: &str| -> Result<std::collections::HashMap<i64, Vec<i64>>> {
        let sql = format!("SELECT log_id, value_id FROM {} ORDER BY log_id, value_id", table);
        let mut stmt = conn.prepare(&sql)?;
        let mut map: std::collections::HashMap<i64, Vec<i64>> = std::collections::HashMap::new();
        let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))?;
        for row in rows { let (lid, vid) = row?; map.entry(lid).or_default().push(vid); }
        Ok(map)
    };

    let c1 = load_cats(CAT_TABLES[0])?;
    let c2 = load_cats(CAT_TABLES[1])?;
    let c3 = load_cats(CAT_TABLES[2])?;
    let c4 = load_cats(CAT_TABLES[3])?;

    Ok(logs.into_iter().map(|mut l| {
        l.category1_ids = c1.get(&l.id).cloned().unwrap_or_default();
        l.category2_ids = c2.get(&l.id).cloned().unwrap_or_default();
        l.category3_ids = c3.get(&l.id).cloned().unwrap_or_default();
        l.category4_ids = c4.get(&l.id).cloned().unwrap_or_default();
        l
    }).collect())
}

pub fn insert(conn: &Connection, log: &Log) -> Result<i64> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO logs (type_id, title, description, start_date, due_date,
                           is_closed, closed_date, project_id)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![
            log.type_id, log.title, log.description, log.start_date,
            log.due_date, log.is_closed, log.closed_date, log.project_id,
        ],
    )?;
    let id = tx.last_insert_rowid();
    let cat_ids = [&log.category1_ids, &log.category2_ids, &log.category3_ids, &log.category4_ids];
    for (table, ids) in CAT_TABLES.iter().zip(cat_ids) {
        sync_cat(&tx, table, id, ids)?;
    }
    tx.commit()?;
    Ok(id)
}

pub fn update(conn: &Connection, log: &Log) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE logs SET
            type_id=?1, title=?2, description=?3, due_date=?4,
            is_closed=?5, closed_date=?6, project_id=?7
         WHERE id=?8",
        params![
            log.type_id, log.title, log.description, log.due_date,
            log.is_closed, log.closed_date, log.project_id,
            log.id,
        ],
    )?;
    let cat_ids = [&log.category1_ids, &log.category2_ids, &log.category3_ids, &log.category4_ids];
    for (table, ids) in CAT_TABLES.iter().zip(cat_ids) {
        sync_cat(&tx, table, log.id, ids)?;
    }
    tx.commit()?;
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM logs WHERE id = ?1", params![id])?;
    Ok(())
}
