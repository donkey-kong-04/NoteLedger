mod db;
mod models;

use std::sync::Mutex;
use rusqlite::Connection;
use tauri::State;
use chrono::Local;

use models::settings::UserSettings;
use models::picklist::PicklistValue;
use models::log::Log;
use models::project::Project;
use models::project_link::ProjectLink;

pub struct AppState(Mutex<Connection>);

// ── Settings ──────────────────────────────────────────────────────────────────

#[tauri::command]
fn get_settings(state: State<AppState>) -> Result<UserSettings, String> {
    let conn = state.0.lock().unwrap();
    db::settings_repo::load(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_settings(state: State<AppState>, settings: UserSettings) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::settings_repo::save(&conn, &settings).map_err(|e| e.to_string())
}

// ── Picklists ─────────────────────────────────────────────────────────────────

#[tauri::command]
fn get_all_picklists(state: State<AppState>) -> Result<Vec<PicklistValue>, String> {
    let conn = state.0.lock().unwrap();
    db::picklist_repo::list_all(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_picklist_value(state: State<AppState>, picklist_type: String, label: String) -> Result<PicklistValue, String> {
    let conn = state.0.lock().unwrap();
    let id = db::picklist_repo::insert(&conn, &picklist_type, &label).map_err(|e| e.to_string())?;
    Ok(PicklistValue { id, picklist_type, label })
}

#[tauri::command]
fn update_picklist_value(state: State<AppState>, id: i64, label: String) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::picklist_repo::update(&conn, id, &label).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_picklist_value(state: State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::picklist_repo::delete(&conn, id).map_err(|e| e.to_string())
}

// ── Projects ──────────────────────────────────────────────────────────────

#[tauri::command]
fn get_projects(state: State<AppState>) -> Result<Vec<Project>, String> {
    let conn = state.0.lock().unwrap();
    db::project_repo::list(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_project(state: State<AppState>, mut project: Project) -> Result<Project, String> {
    let conn = state.0.lock().unwrap();
    let id = db::project_repo::insert(&conn, &project).map_err(|e| e.to_string())?;
    project.id = id;
    Ok(project)
}

#[tauri::command]
fn update_project(state: State<AppState>, project: Project) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::project_repo::update(&conn, &project).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_project(state: State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::project_repo::delete(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn clone_project(state: State<AppState>, id: i64, new_title: String) -> Result<i64, String> {
    let conn = state.0.lock().unwrap();
    db::project_repo::clone_tree(&conn, id, &new_title).map_err(|e| e.to_string())
}

// ── Project Links ─────────────────────────────────────────────────────────────

#[tauri::command]
fn get_project_links(state: State<AppState>) -> Result<Vec<ProjectLink>, String> {
    let conn = state.0.lock().unwrap();
    db::project_link_repo::list_all(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_project_link(state: State<AppState>, mut link: ProjectLink) -> Result<ProjectLink, String> {
    let conn = state.0.lock().unwrap();
    let id = db::project_link_repo::insert(&conn, &link).map_err(|e| e.to_string())?;
    link.id = id;
    Ok(link)
}

#[tauri::command]
fn update_project_link(state: State<AppState>, link: ProjectLink) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::project_link_repo::update(&conn, &link).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_project_link(state: State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::project_link_repo::delete(&conn, id).map_err(|e| e.to_string())
}

// ── Logs ──────────────────────────────────────────────────────────────────────

#[tauri::command]
fn get_logs(state: State<AppState>) -> Result<Vec<Log>, String> {
    let conn = state.0.lock().unwrap();
    db::log_repo::list(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_log(state: State<AppState>, mut log: Log) -> Result<Log, String> {
    let conn = state.0.lock().unwrap();
    log.start_date = Local::now().date_naive().to_string();
    let id = db::log_repo::insert(&conn, &log).map_err(|e| e.to_string())?;
    log.id = id;
    Ok(log)
}

#[tauri::command]
fn update_log(state: State<AppState>, mut log: Log) -> Result<Log, String> {
    let conn = state.0.lock().unwrap();
    // Auto-manage closed_date
    if log.is_closed && log.closed_date.is_none() {
        log.closed_date = Some(Local::now().date_naive().to_string());
    } else if !log.is_closed {
        log.closed_date = None;
    }
    db::log_repo::update(&conn, &log).map_err(|e| e.to_string())?;
    Ok(log)
}

#[tauri::command]
fn delete_log(state: State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::log_repo::delete(&conn, id).map_err(|e| e.to_string())
}

// ── App setup ─────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let data_dir = dirs_next::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("note-ledger");
    std::fs::create_dir_all(&data_dir).ok();
    let db_path = data_dir.join("note_ledger.db");

    let conn = db::open(db_path.to_str().unwrap()).expect("Failed to open database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState(Mutex::new(conn)))
        .invoke_handler(tauri::generate_handler![
            get_settings, save_settings,
            get_all_picklists, create_picklist_value, update_picklist_value, delete_picklist_value,
            get_projects, create_project, update_project, delete_project, clone_project,
            get_project_links, create_project_link, update_project_link, delete_project_link,
            get_logs, create_log, update_log, delete_log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
