use rusqlite::{Connection, Result, params, OptionalExtension};
use crate::models::project::Project;

// The four category junction tables for projects. Names are compile-time
// constants (never user input), so interpolating them into SQL is safe.
const CAT_TABLES: [&str; 4] = ["project_category_1", "project_category_2", "project_category_3", "project_category_4"];

fn project_cat_ids(p: &Project) -> [&Vec<i64>; 4] {
    [&p.category1_ids, &p.category2_ids, &p.category3_ids, &p.category4_ids]
}

fn load_category_ids(conn: &Connection, table: &str, project_id: i64) -> Result<Vec<i64>> {
    let sql = format!("SELECT value_id FROM {} WHERE project_id = ?1 ORDER BY value_id", table);
    let mut stmt = conn.prepare(&sql)?;
    let ids: Result<Vec<i64>> = stmt.query_map(params![project_id], |r| r.get(0))?.collect();
    ids
}

fn save_categories(conn: &Connection, table: &str, project_id: i64, ids: &[i64]) -> Result<()> {
    let del = format!("DELETE FROM {} WHERE project_id = ?1", table);
    conn.execute(&del, params![project_id])?;
    let ins = format!("INSERT OR IGNORE INTO {} (project_id, value_id) VALUES (?1, ?2)", table);
    for &vid in ids {
        conn.execute(&ins, params![project_id, vid])?;
    }
    Ok(())
}

fn load_project(conn: &Connection, id: i64, title: String, description: String, parent_id: Option<i64>, is_closed: bool, start_date: Option<String>, end_date: Option<String>) -> Result<Project> {
    Ok(Project {
        id,
        title,
        description,
        parent_id,
        is_closed,
        start_date,
        end_date,
        category1_ids: load_category_ids(conn, "project_category_1", id)?,
        category2_ids: load_category_ids(conn, "project_category_2", id)?,
        category3_ids: load_category_ids(conn, "project_category_3", id)?,
        category4_ids: load_category_ids(conn, "project_category_4", id)?,
    })
}

pub fn list(conn: &Connection) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, description, parent_id, is_closed, start_date, end_date FROM projects ORDER BY id"
    )?;
    let rows: Result<Vec<(i64, String, String, Option<i64>, bool, Option<String>, Option<String>)>> = stmt.query_map([], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get::<_, Option<String>>(2)?.unwrap_or_default(), r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?))
    })?.collect();
    rows?.into_iter()
        .map(|(id, title, desc, parent_id, is_closed, start_date, end_date)| load_project(conn, id, title, desc, parent_id, is_closed, start_date, end_date))
        .collect()
}

pub fn insert(conn: &Connection, project: &Project) -> Result<i64> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO projects (title, description, parent_id, is_closed, start_date, end_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![project.title, project.description, project.parent_id, project.is_closed, project.start_date, project.end_date],
    )?;
    let id = tx.last_insert_rowid();
    for (table, ids) in CAT_TABLES.iter().zip(project_cat_ids(project)) {
        save_categories(&tx, table, id, ids)?;
    }
    tx.commit()?;
    Ok(id)
}

pub fn update(conn: &Connection, project: &Project) -> Result<()> {
    if let Some(new_parent) = project.parent_id {
        if would_create_cycle(conn, project.id, new_parent)? {
            return Err(rusqlite::Error::InvalidParameterName(
                "Cannot set parent: would create a cycle.".into()
            ));
        }
    }
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE projects SET title=?1, description=?2, parent_id=?3, is_closed=?4, start_date=?5, end_date=?6 WHERE id=?7",
        params![project.title, project.description, project.parent_id, project.is_closed, project.start_date, project.end_date, project.id],
    )?;
    for (table, ids) in CAT_TABLES.iter().zip(project_cat_ids(project)) {
        save_categories(&tx, table, project.id, ids)?;
    }
    tx.commit()?;
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    let has_children: i64 = conn.query_row(
        "SELECT COUNT(*) FROM projects WHERE parent_id = ?1", params![id], |r| r.get(0)
    )?;
    if has_children > 0 {
        return Err(rusqlite::Error::InvalidParameterName(
            "Cannot delete a project that has sub-projects.".into()
        ));
    }

    // A project that still contains logs cannot be deleted — the user must move
    // or delete those logs first.
    let log_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM logs WHERE project_id = ?1", params![id], |r| r.get(0)
    )?;
    if log_count > 0 {
        return Err(rusqlite::Error::InvalidParameterName(
            format!("Cannot delete a project that contains {} log(s). Move or delete them first.", log_count)
        ));
    }

    // project_category_* and project_links rows cascade on delete (schema v4/v9).
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    Ok(())
}

fn would_create_cycle(conn: &Connection, project_id: i64, new_parent_id: i64) -> Result<bool> {
    let mut current = new_parent_id;
    loop {
        if current == project_id { return Ok(true); }
        let parent: Option<Option<i64>> = conn.query_row(
            "SELECT parent_id FROM projects WHERE id = ?1",
            params![current],
            |r| r.get(0),
        ).optional()?;
        match parent {
            Some(Some(p)) => current = p,
            _ => return Ok(false),
        }
    }
}
