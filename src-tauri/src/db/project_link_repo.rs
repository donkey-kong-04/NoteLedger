use rusqlite::{Connection, Result, params};
use crate::models::project_link::ProjectLink;

pub fn list_for_project(conn: &Connection, project_id: i64) -> Result<Vec<ProjectLink>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, label, url FROM project_links WHERE project_id = ?1 ORDER BY id"
    )?;
    let rows: Result<Vec<ProjectLink>> = stmt.query_map(params![project_id], |r| {
        Ok(ProjectLink { id: r.get(0)?, project_id: r.get(1)?, label: r.get(2)?, url: r.get(3)? })
    })?.collect();
    rows
}

pub fn list_all(conn: &Connection) -> Result<Vec<ProjectLink>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, label, url FROM project_links ORDER BY project_id, id"
    )?;
    let rows: Result<Vec<ProjectLink>> = stmt.query_map([], |r| {
        Ok(ProjectLink { id: r.get(0)?, project_id: r.get(1)?, label: r.get(2)?, url: r.get(3)? })
    })?.collect();
    rows
}

pub fn insert(conn: &Connection, link: &ProjectLink) -> Result<i64> {
    conn.execute(
        "INSERT INTO project_links (project_id, label, url) VALUES (?1, ?2, ?3)",
        params![link.project_id, link.label, link.url],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update(conn: &Connection, link: &ProjectLink) -> Result<()> {
    conn.execute(
        "UPDATE project_links SET label = ?1, url = ?2 WHERE id = ?3",
        params![link.label, link.url, link.id],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM project_links WHERE id = ?1", params![id])?;
    Ok(())
}
