use rusqlite::{Connection, Result, params};
use crate::models::settings::UserSettings;

pub fn load(conn: &Connection) -> Result<UserSettings> {
    conn.query_row(
        "SELECT category1_label, category2_label, category3_label, category4_label, dark_mode
         FROM user_settings WHERE id = 1",
        [],
        |row| Ok(UserSettings {
            category1_label: row.get(0)?,
            category2_label: row.get(1)?,
            category3_label: row.get(2)?,
            category4_label: row.get(3)?,
            dark_mode: row.get(4)?,
        }),
    )
}

pub fn save(conn: &Connection, s: &UserSettings) -> Result<()> {
    conn.execute(
        "UPDATE user_settings SET
            category1_label = ?1, category2_label = ?2,
            category3_label = ?3, category4_label = ?4,
            dark_mode = ?5
         WHERE id = 1",
        params![s.category1_label, s.category2_label, s.category3_label, s.category4_label, s.dark_mode],
    )?;
    Ok(())
}
