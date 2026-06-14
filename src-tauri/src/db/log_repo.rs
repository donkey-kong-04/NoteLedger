use rusqlite::{Connection, Result, params};
use crate::models::log::Log;

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

    let ids_str = logs.iter().map(|l| l.id.to_string()).collect::<Vec<_>>().join(",");

    let load_cats = |table: &str| -> Result<std::collections::HashMap<i64, Vec<i64>>> {
        let sql = format!("SELECT log_id, value_id FROM {} WHERE log_id IN ({}) ORDER BY log_id, value_id", table, ids_str);
        let mut stmt = conn.prepare(&sql)?;
        let mut map: std::collections::HashMap<i64, Vec<i64>> = std::collections::HashMap::new();
        let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))?;
        for row in rows { let (lid, vid) = row?; map.entry(lid).or_default().push(vid); }
        Ok(map)
    };

    let c1 = load_cats("log_category_1")?;
    let c2 = load_cats("log_category_2")?;
    let c3 = load_cats("log_category_3")?;
    let c4 = load_cats("log_category_4")?;

    Ok(logs.into_iter().map(|mut l| {
        l.category1_ids = c1.get(&l.id).cloned().unwrap_or_default();
        l.category2_ids = c2.get(&l.id).cloned().unwrap_or_default();
        l.category3_ids = c3.get(&l.id).cloned().unwrap_or_default();
        l.category4_ids = c4.get(&l.id).cloned().unwrap_or_default();
        l
    }).collect())
}

pub fn insert(conn: &Connection, log: &Log) -> Result<i64> {
    conn.execute(
        "INSERT INTO logs (type_id, title, description, start_date, due_date,
                           is_closed, closed_date, project_id)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![
            log.type_id, log.title, log.description, log.start_date,
            log.due_date, log.is_closed, log.closed_date, log.project_id,
        ],
    )?;
    let id = conn.last_insert_rowid();
    sync_cat(conn, "log_category_1", id, &log.category1_ids)?;
    sync_cat(conn, "log_category_2", id, &log.category2_ids)?;
    sync_cat(conn, "log_category_3", id, &log.category3_ids)?;
    sync_cat(conn, "log_category_4", id, &log.category4_ids)?;
    Ok(id)
}

pub fn update(conn: &Connection, log: &Log) -> Result<()> {
    conn.execute(
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
    sync_cat(conn, "log_category_1", log.id, &log.category1_ids)?;
    sync_cat(conn, "log_category_2", log.id, &log.category2_ids)?;
    sync_cat(conn, "log_category_3", log.id, &log.category3_ids)?;
    sync_cat(conn, "log_category_4", log.id, &log.category4_ids)?;
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM logs WHERE id = ?1", params![id])?;
    Ok(())
}
