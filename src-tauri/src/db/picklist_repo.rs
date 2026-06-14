use rusqlite::{Connection, Result, params};
use crate::models::picklist::PicklistValue;

pub fn list(conn: &Connection, picklist_type: &str) -> Result<Vec<PicklistValue>> {
    let mut stmt = conn.prepare(
        "SELECT id, type, label FROM user_customizable_input WHERE type = ?1 ORDER BY id"
    )?;
    let rows = stmt.query_map(params![picklist_type], |row| {
        Ok(PicklistValue {
            id: row.get(0)?,
            picklist_type: row.get(1)?,
            label: row.get(2)?,
        })
    })?;
    rows.collect()
}

pub fn list_all(conn: &Connection) -> Result<Vec<PicklistValue>> {
    let mut stmt = conn.prepare(
        "SELECT id, type, label FROM user_customizable_input ORDER BY type, id"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(PicklistValue {
            id: row.get(0)?,
            picklist_type: row.get(1)?,
            label: row.get(2)?,
        })
    })?;
    rows.collect()
}

pub fn insert(conn: &Connection, picklist_type: &str, label: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO user_customizable_input (type, label) VALUES (?1, ?2)",
        params![picklist_type, label],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update(conn: &Connection, id: i64, label: &str) -> Result<()> {
    conn.execute(
        "UPDATE user_customizable_input SET label = ?1 WHERE id = ?2",
        params![label, id],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    let in_use: i64 = conn.query_row(
        "SELECT COUNT(*) FROM (
            SELECT id FROM logs WHERE type_id = ?1
            UNION SELECT log_id FROM log_category_1 WHERE value_id = ?1
            UNION SELECT log_id FROM log_category_2 WHERE value_id = ?1
            UNION SELECT log_id FROM log_category_3 WHERE value_id = ?1
            UNION SELECT log_id FROM log_category_4 WHERE value_id = ?1
         )",
        params![id],
        |row| row.get(0),
    )?;
    if in_use > 0 {
        return Err(rusqlite::Error::InvalidParameterName(
            format!("This value is used by {} log(s) and cannot be deleted.", in_use)
        ));
    }
    conn.execute("DELETE FROM user_customizable_input WHERE id = ?1", params![id])?;
    Ok(())
}
