use super::*;
use crate::models::log::Log;
use crate::models::project::Project;
use rusqlite::Connection;

/// Fresh in-memory DB, fully migrated — mirrors what `open()` produces.
fn mem() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
    schema::migrate(&conn).unwrap();
    conn
}

fn project(title: &str) -> Project {
    Project {
        id: 0,
        title: title.into(),
        description: String::new(),
        parent_id: None,
        is_closed: false,
        is_template: false,
        start_date: None,
        end_date: None,
        category1_ids: vec![],
        category2_ids: vec![],
        category3_ids: vec![],
        category4_ids: vec![],
    }
}

fn log(project_id: i64) -> Log {
    Log {
        id: 0,
        type_id: 1, // seeded "Task" log type
        title: "T".into(),
        description: "d".into(),
        closed_description: String::new(),
        start_date: "2026-01-01".into(),
        due_date: None,
        is_closed: false,
        closed_date: None,
        project_id,
        category1_ids: vec![],
        category2_ids: vec![],
        category3_ids: vec![],
        category4_ids: vec![],
    }
}

#[test]
fn fresh_migration_reaches_latest_with_seeds() {
    let conn = mem();
    let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap();
    assert_eq!(version, schema::LATEST_VERSION);

    let log_types = picklist_repo::list(&conn, "log_type").unwrap();
    let labels: Vec<&str> = log_types.iter().map(|v| v.label.as_str()).collect();
    assert_eq!(labels, vec!["Task", "Decision", "Event"]);
}

#[test]
fn log_categories_round_trip_and_replace_on_update() {
    let conn = mem();
    let pid = project_repo::insert(&conn, &project("P")).unwrap();
    let a = picklist_repo::insert(&conn, "category_1", "A").unwrap();
    let b = picklist_repo::insert(&conn, "category_1", "B").unwrap();
    let c = picklist_repo::insert(&conn, "category_1", "C").unwrap();

    let mut l = log(pid);
    l.category1_ids = vec![a];
    let id = log_repo::insert(&conn, &l).unwrap();

    let loaded = log_repo::list(&conn).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].category1_ids, vec![a]);

    // Update replaces the category set rather than appending.
    let mut updated = loaded[0].clone();
    updated.id = id;
    updated.category1_ids = vec![b, c];
    log_repo::update(&conn, &updated).unwrap();

    let reloaded = log_repo::list(&conn).unwrap();
    assert_eq!(reloaded[0].category1_ids, vec![b, c]);
}

#[test]
fn deleting_project_with_logs_is_blocked() {
    let conn = mem();
    let pid = project_repo::insert(&conn, &project("Work")).unwrap();
    log_repo::insert(&conn, &log(pid)).unwrap();

    assert!(
        project_repo::delete(&conn, pid).is_err(),
        "a project that still contains logs must not be deletable"
    );

    // Both the project and its log are untouched.
    let still_there: i64 = conn
        .query_row("SELECT COUNT(*) FROM projects WHERE id = ?1", [pid], |r| r.get(0))
        .unwrap();
    assert_eq!(still_there, 1);
    assert_eq!(log_repo::list(&conn).unwrap().len(), 1);
}

#[test]
fn deleting_project_with_no_logs_succeeds() {
    let conn = mem();
    let pid = project_repo::insert(&conn, &project("Empty")).unwrap();
    project_repo::delete(&conn, pid).unwrap();
    // No stray "Others" project created when there was nothing to reassign.
    let others: i64 = conn
        .query_row("SELECT COUNT(*) FROM projects WHERE title = 'Others'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(others, 0);
}

#[test]
fn picklist_value_used_by_project_cannot_be_deleted() {
    let conn = mem();
    let cat = picklist_repo::insert(&conn, "category_1", "X").unwrap();
    let mut p = project("P");
    p.category1_ids = vec![cat];
    project_repo::insert(&conn, &p).unwrap();

    assert!(
        picklist_repo::delete(&conn, cat).is_err(),
        "value attached to a project must be protected from deletion"
    );
}

#[test]
fn log_type_assigned_to_a_log_cannot_be_deleted() {
    let conn = mem();
    let pid = project_repo::insert(&conn, &project("P")).unwrap();
    let mut l = log(pid);
    l.type_id = 1; // seeded "Task" log type
    log_repo::insert(&conn, &l).unwrap();

    assert!(
        picklist_repo::delete(&conn, 1).is_err(),
        "a log type in use by a log must be protected from deletion"
    );
}

#[test]
fn unused_picklist_value_can_be_deleted() {
    let conn = mem();
    let cat = picklist_repo::insert(&conn, "category_2", "Unused").unwrap();
    assert!(picklist_repo::delete(&conn, cat).is_ok());
}

#[test]
fn clone_tree_deep_copies_everything_independently() {
    let conn = mem();

    // Template root with a category, a link, a log, and a child with a log
    let mut root = project("Template");
    root.is_template = true;
    root.category1_ids = vec![picklist_repo::insert(&conn, "category_1", "Cat").unwrap()];
    let root_id = project_repo::insert(&conn, &root).unwrap();

    let mut child = project("Child");
    child.parent_id = Some(root_id);
    let child_id = project_repo::insert(&conn, &child).unwrap();

    log_repo::insert(&conn, &log(root_id)).unwrap();
    log_repo::insert(&conn, &log(child_id)).unwrap();
    conn.execute(
        "INSERT INTO project_links (project_id, label, url) VALUES (?1, 'L', 'https://x')",
        rusqlite::params![root_id],
    ).unwrap();

    let new_id = project_repo::clone_tree(&conn, root_id, "Cloned").unwrap();
    let projects = project_repo::list(&conn).unwrap();

    let new_root = projects.iter().find(|p| p.id == new_id).unwrap();
    assert_eq!(new_root.title, "Cloned");
    assert!(!new_root.is_template, "clones must not be templates");
    assert_eq!(new_root.category1_ids.len(), 1, "categories are copied");

    let new_child = projects.iter().find(|p| p.parent_id == Some(new_id)).unwrap();
    assert_eq!(new_child.title, "Child");
    assert!(!new_child.is_template);

    let logs = log_repo::list(&conn).unwrap();
    assert_eq!(logs.iter().filter(|l| l.project_id == new_id).count(), 1, "root log copied");
    assert_eq!(logs.iter().filter(|l| l.project_id == new_child.id).count(), 1, "child log copied");

    let links: i64 = conn.query_row(
        "SELECT COUNT(*) FROM project_links WHERE project_id = ?1",
        rusqlite::params![new_id], |r| r.get(0),
    ).unwrap();
    assert_eq!(links, 1, "links are copied");

    // Independence: editing the template does not touch the clone
    let mut tpl = projects.iter().find(|p| p.id == root_id).unwrap().clone();
    tpl.title = "Template renamed".into();
    project_repo::update(&conn, &tpl).unwrap();
    let after = project_repo::list(&conn).unwrap();
    assert_eq!(after.iter().find(|p| p.id == new_id).unwrap().title, "Cloned");
}

#[test]
fn sub_project_inherits_template_flag() {
    let conn = mem();
    let mut root = project("Tpl");
    root.is_template = true;
    let root_id = project_repo::insert(&conn, &root).unwrap();

    let mut child = project("Sub");
    child.parent_id = Some(root_id);
    child.is_template = false; // frontend may not set it — must inherit anyway
    let child_id = project_repo::insert(&conn, &child).unwrap();

    let projects = project_repo::list(&conn).unwrap();
    assert!(projects.iter().find(|p| p.id == child_id).unwrap().is_template);
}

#[test]
fn clone_tree_shifts_dates_to_today() {
    let conn = mem();
    let today = chrono::NaiveDate::from_ymd_opt(2026, 7, 4).unwrap();

    // Template: project started 2026-05-01, ends 2026-08-01 (3 months planned).
    // Logs due 2026-06-01 (earliest, in a child), 2026-06-04, 2026-06-07.
    let mut root = project("Tpl");
    root.is_template = true;
    root.start_date = Some("2026-05-01".into());
    root.end_date = Some("2026-08-01".into());
    let root_id = project_repo::insert(&conn, &root).unwrap();

    let mut child = project("Child");
    child.parent_id = Some(root_id);
    let child_id = project_repo::insert(&conn, &child).unwrap();

    let mut l1 = log(child_id); l1.due_date = Some("2026-06-01".into());
    let mut l2 = log(root_id);  l2.due_date = Some("2026-06-04".into());
    let mut l3 = log(root_id);  l3.due_date = Some("2026-06-07".into());
    let mut l4 = log(root_id);  l4.due_date = None; // no due date stays empty
    for l in [&l1, &l2, &l3, &l4] { log_repo::insert(&conn, l).unwrap(); }

    let new_id = project_repo::clone_tree_at(&conn, root_id, "Cloned", today).unwrap();

    let projects = project_repo::list(&conn).unwrap();
    let new_root = projects.iter().find(|p| p.id == new_id).unwrap();
    // Start moves to today; end keeps the original 92-day distance from start.
    assert_eq!(new_root.start_date.as_deref(), Some("2026-07-04"));
    assert_eq!(new_root.end_date.as_deref(), Some("2026-10-04"));

    let new_child = projects.iter().find(|p| p.parent_id == Some(new_id)).unwrap();
    let logs = log_repo::list(&conn).unwrap();
    let cloned: Vec<&crate::models::log::Log> = logs.iter()
        .filter(|l| l.project_id == new_id || l.project_id == new_child.id)
        .collect();

    // Earliest (06-01) → today (07-04); the others keep their +3/+6 day distance.
    let mut dues: Vec<Option<&str>> = cloned.iter().map(|l| l.due_date.as_deref()).collect();
    dues.sort();
    assert_eq!(dues, vec![None, Some("2026-07-04"), Some("2026-07-07"), Some("2026-07-10")]);

    // "Open since" resets: every cloned log starts today.
    assert!(cloned.iter().all(|l| l.start_date == "2026-07-04"));

    // Template dates untouched.
    let tpl = projects.iter().find(|p| p.id == root_id).unwrap();
    assert_eq!(tpl.start_date.as_deref(), Some("2026-05-01"));
}
