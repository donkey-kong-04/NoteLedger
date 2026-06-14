use rusqlite::{Connection, Result, params, OptionalExtension};
use crate::models::project::Project;

fn row_to_project(row: &rusqlite::Row) -> rusqlite::Result<Project> {
    Ok(Project {
        id:          row.get(0)?,
        title:       row.get(1)?,
        description: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
        parent_id:   row.get(3)?,
    })
}

pub fn list(conn: &Connection) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, description, parent_id FROM projects ORDER BY id"
    )?;
    let x: Result<Vec<Project>> = stmt.query_map([], row_to_project)?.collect();
    x
}

pub fn insert(conn: &Connection, project: &Project) -> Result<i64> {
    conn.execute(
        "INSERT INTO projects (title, description, parent_id) VALUES (?1, ?2, ?3)",
        params![project.title, project.description, project.parent_id],
    )?;
    Ok(conn.last_insert_rowid())
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
    // Unlink logs before deleting
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
