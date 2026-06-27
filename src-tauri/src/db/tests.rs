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
