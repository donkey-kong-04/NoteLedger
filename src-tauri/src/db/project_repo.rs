use chrono::{Duration, Local, NaiveDate};
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

fn load_project(conn: &Connection, id: i64, title: String, description: String, parent_id: Option<i64>, is_closed: bool, is_template: bool, start_date: Option<String>, end_date: Option<String>) -> Result<Project> {
    Ok(Project {
        id,
        title,
        description,
        parent_id,
        is_closed,
        is_template,
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
        "SELECT id, title, description, parent_id, is_closed, is_template, start_date, end_date FROM projects ORDER BY id"
    )?;
    let rows: Result<Vec<(i64, String, String, Option<i64>, bool, bool, Option<String>, Option<String>)>> = stmt.query_map([], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get::<_, Option<String>>(2)?.unwrap_or_default(), r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?))
    })?.collect();
    rows?.into_iter()
        .map(|(id, title, desc, parent_id, is_closed, is_template, start_date, end_date)| load_project(conn, id, title, desc, parent_id, is_closed, is_template, start_date, end_date))
        .collect()
}

pub fn insert(conn: &Connection, project: &Project) -> Result<i64> {
    // A sub-project always inherits its parent's template flag so a template
    // tree stays consistently marked.
    let is_template = match project.parent_id {
        Some(pid) => conn.query_row(
            "SELECT is_template FROM projects WHERE id = ?1", params![pid], |r| r.get(0)
        )?,
        None => project.is_template,
    };
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO projects (title, description, parent_id, is_closed, is_template, start_date, end_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![project.title, project.description, project.parent_id, project.is_closed, is_template, project.start_date, project.end_date],
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
    // is_template is intentionally not updatable — it is fixed at creation/clone time.
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

/// Deep-copies a project tree: the project itself (with `new_title`), its
/// categories, links, logs (with their categories), and all sub-projects
/// recursively. The copies are fully independent of the source and are never
/// templates.
///
/// Dates are re-anchored to today:
/// - every project start date becomes today; its end date is shifted by the
///   same amount so the planned duration is preserved;
/// - log due dates shift as a group: the earliest due date in the whole tree
///   lands on today and all others keep their relative distance to it;
/// - log start dates ("open since") are reset to today.
///
/// Returns the id of the new root project.
pub fn clone_tree(conn: &Connection, src_id: i64, new_title: &str) -> Result<i64> {
    clone_tree_at(conn, src_id, new_title, Local::now().date_naive())
}

/// Testable variant of `clone_tree` with an injectable "today".
pub fn clone_tree_at(conn: &Connection, src_id: i64, new_title: &str, today: NaiveDate) -> Result<i64> {
    // Global log shift: earliest due date anywhere in the tree → today.
    let mut subtree = vec![src_id];
    collect_subtree_ids(conn, src_id, &mut subtree)?;
    let mut min_due: Option<NaiveDate> = None;
    for pid in &subtree {
        let mut stmt = conn.prepare("SELECT due_date FROM logs WHERE project_id = ?1 AND due_date IS NOT NULL")?;
        let dates: Result<Vec<String>> = stmt.query_map(params![pid], |r| r.get(0))?.collect();
        for d in dates? {
            if let Ok(parsed) = NaiveDate::parse_from_str(&d, "%Y-%m-%d") {
                min_due = Some(min_due.map_or(parsed, |m: NaiveDate| m.min(parsed)));
            }
        }
    }
    let log_shift_days = min_due.map(|d| (today - d).num_days()).unwrap_or(0);

    let tx = conn.unchecked_transaction()?;
    let new_id = clone_node(&tx, src_id, None, Some(new_title), today, log_shift_days)?;
    tx.commit()?;
    Ok(new_id)
}

fn collect_subtree_ids(conn: &Connection, root: i64, acc: &mut Vec<i64>) -> Result<()> {
    let children: Vec<i64> = {
        let mut stmt = conn.prepare("SELECT id FROM projects WHERE parent_id = ?1")?;
        let ids: Result<Vec<i64>> = stmt.query_map(params![root], |r| r.get(0))?.collect();
        ids?
    };
    for c in children {
        acc.push(c);
        collect_subtree_ids(conn, c, acc)?;
    }
    Ok(())
}

/// Shift an ISO date string by `days`; unparseable dates pass through unchanged.
fn shift_date(date: &str, days: i64) -> String {
    match NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        Ok(d) => (d + Duration::days(days)).to_string(),
        Err(_) => date.to_string(),
    }
}

fn clone_node(conn: &Connection, src_id: i64, new_parent: Option<i64>, title_override: Option<&str>, today: NaiveDate, log_shift_days: i64) -> Result<i64> {
    let (title, description, is_closed, start_date, end_date): (String, Option<String>, bool, Option<String>, Option<String>) = conn.query_row(
        "SELECT title, description, is_closed, start_date, end_date FROM projects WHERE id = ?1",
        params![src_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
    )?;
    let title = title_override.map(str::to_string).unwrap_or(title);

    // Project start → today; end keeps its distance from start (falls back to
    // the global log shift when there is no start to measure from).
    let project_shift_days = start_date
        .as_deref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .map(|s| (today - s).num_days())
        .unwrap_or(log_shift_days);
    let new_start = start_date.as_ref().map(|_| today.to_string());
    let new_end = end_date.as_deref().map(|e| shift_date(e, project_shift_days));

    conn.execute(
        "INSERT INTO projects (title, description, parent_id, is_closed, is_template, start_date, end_date) VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6)",
        params![title, description, new_parent, is_closed, new_start, new_end],
    )?;
    let new_id = conn.last_insert_rowid();

    for table in CAT_TABLES {
        let sql = format!("INSERT INTO {t} (project_id, value_id) SELECT ?1, value_id FROM {t} WHERE project_id = ?2", t = table);
        conn.execute(&sql, params![new_id, src_id])?;
    }

    conn.execute(
        "INSERT INTO project_links (project_id, label, url) SELECT ?1, label, url FROM project_links WHERE project_id = ?2",
        params![new_id, src_id],
    )?;

    let log_rows: Vec<(i64, Option<String>)> = {
        let mut stmt = conn.prepare("SELECT id, due_date FROM logs WHERE project_id = ?1")?;
        let rows: Result<Vec<(i64, Option<String>)>> = stmt.query_map(params![src_id], |r| Ok((r.get(0)?, r.get(1)?)))?.collect();
        rows?
    };
    for (lid, due_date) in log_rows {
        let new_due = due_date.as_deref().map(|d| shift_date(d, log_shift_days));
        conn.execute(
            "INSERT INTO logs (type_id, title, description, start_date, due_date, is_closed, closed_date, project_id, closed_description)
             SELECT type_id, title, description, ?1, ?2, is_closed, closed_date, ?3, closed_description FROM logs WHERE id = ?4",
            params![today.to_string(), new_due, new_id, lid],
        )?;
        let new_log_id = conn.last_insert_rowid();
        for table in ["log_category_1", "log_category_2", "log_category_3", "log_category_4"] {
            let sql = format!("INSERT INTO {t} (log_id, value_id) SELECT ?1, value_id FROM {t} WHERE log_id = ?2", t = table);
            conn.execute(&sql, params![new_log_id, lid])?;
        }
    }

    let child_ids: Vec<i64> = {
        let mut stmt = conn.prepare("SELECT id FROM projects WHERE parent_id = ?1")?;
        let ids: Result<Vec<i64>> = stmt.query_map(params![src_id], |r| r.get(0))?.collect();
        ids?
    };
    for cid in child_ids {
        clone_node(conn, cid, Some(new_id), None, today, log_shift_days)?;
    }

    Ok(new_id)
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
