use rusqlite::{Connection, Result, params, OptionalExtension};
use crate::models::project::Project;

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

fn load_project(conn: &Connection, id: i64, title: String, description: String, parent_id: Option<i64>) -> Result<Project> {
    Ok(Project {
        id,
        title,
        description,
        parent_id,
        category1_ids: load_category_ids(conn, "project_category_1", id)?,
        category2_ids: load_category_ids(conn, "project_category_2", id)?,
        category3_ids: load_category_ids(conn, "project_category_3", id)?,
        category4_ids: load_category_ids(conn, "project_category_4", id)?,
    })
}

pub fn list(conn: &Connection) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, description, parent_id FROM projects ORDER BY id"
    )?;
    let rows: Result<Vec<(i64, String, String, Option<i64>)>> = stmt.query_map([], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get::<_, Option<String>>(2)?.unwrap_or_default(), r.get(3)?))
    })?.collect();
    rows?.into_iter()
        .map(|(id, title, desc, parent_id)| load_project(conn, id, title, desc, parent_id))
        .collect()
}

pub fn insert(conn: &Connection, project: &Project) -> Result<i64> {
    conn.execute(
        "INSERT INTO projects (title, description, parent_id) VALUES (?1, ?2, ?3)",
        params![project.title, project.description, project.parent_id],
    )?;
    let id = conn.last_insert_rowid();
    save_categories(conn, "project_category_1", id, &project.category1_ids)?;
    save_categories(conn, "project_category_2", id, &project.category2_ids)?;
    save_categories(conn, "project_category_3", id, &project.category3_ids)?;
    save_categories(conn, "project_category_4", id, &project.category4_ids)?;
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
    conn.execute(
        "UPDATE projects SET title=?1, description=?2, parent_id=?3 WHERE id=?4",
        params![project.title, project.description, project.parent_id, project.id],
    )?;
    save_categories(conn, "project_category_1", project.id, &project.category1_ids)?;
    save_categories(conn, "project_category_2", project.id, &project.category2_ids)?;
    save_categories(conn, "project_category_3", project.id, &project.category3_ids)?;
    save_categories(conn, "project_category_4", project.id, &project.category4_ids)?;
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
    conn.execute("UPDATE logs SET project_id = NULL WHERE project_id = ?1", params![id])?;
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
