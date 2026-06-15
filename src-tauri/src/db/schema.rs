use rusqlite::{Connection, Result};

pub fn migrate(conn: &Connection) -> Result<()> {
    let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;

    if version < 1 {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS user_settings (
                id              INTEGER PRIMARY KEY CHECK (id = 1),
                category1_label TEXT    NOT NULL DEFAULT 'Category 1',
                category2_label TEXT    NOT NULL DEFAULT 'Category 2',
                category3_label TEXT    NOT NULL DEFAULT 'Category 3',
                category4_label TEXT    NOT NULL DEFAULT 'Category 4',
                dark_mode       BOOLEAN NOT NULL DEFAULT 0
            );

            INSERT OR IGNORE INTO user_settings (id) VALUES (1);

            CREATE TABLE IF NOT EXISTS user_customizable_input (
                id    INTEGER PRIMARY KEY AUTOINCREMENT,
                type  TEXT    NOT NULL CHECK (type IN ('category_1','category_2','category_3','category_4','log_type')),
                label TEXT    NOT NULL
            );

            INSERT INTO user_customizable_input (type, label)
            SELECT 'log_type', 'Task' WHERE NOT EXISTS (SELECT 1 FROM user_customizable_input WHERE type = 'log_type');
            INSERT INTO user_customizable_input (type, label)
            SELECT 'log_type', 'Decision' WHERE NOT EXISTS (SELECT 1 FROM user_customizable_input WHERE type = 'log_type' AND label = 'Decision');
            INSERT INTO user_customizable_input (type, label)
            SELECT 'log_type', 'Event' WHERE NOT EXISTS (SELECT 1 FROM user_customizable_input WHERE type = 'log_type' AND label = 'Event');

            CREATE TABLE IF NOT EXISTS logs (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                type_id      INTEGER NOT NULL REFERENCES user_customizable_input(id),
                title        TEXT    NOT NULL,
                description  TEXT,
                start_date   TEXT    NOT NULL,
                due_date     TEXT,
                is_closed    BOOLEAN NOT NULL DEFAULT 0,
                closed_date  TEXT,
                parent_id    INTEGER REFERENCES logs(id),
                category1_id INTEGER REFERENCES user_customizable_input(id),
                category2_id INTEGER REFERENCES user_customizable_input(id),
                category3_id INTEGER REFERENCES user_customizable_input(id),
                category4_id INTEGER REFERENCES user_customizable_input(id)
            );

            PRAGMA user_version = 1;
        ")?;
    }

    if version < 2 {
        conn.execute_batch("
            PRAGMA foreign_keys = OFF;

            CREATE TABLE IF NOT EXISTS log_category_1 (
                log_id   INTEGER NOT NULL REFERENCES logs(id) ON DELETE CASCADE,
                value_id INTEGER NOT NULL REFERENCES user_customizable_input(id),
                PRIMARY KEY (log_id, value_id)
            );
            CREATE TABLE IF NOT EXISTS log_category_2 (
                log_id   INTEGER NOT NULL REFERENCES logs(id) ON DELETE CASCADE,
                value_id INTEGER NOT NULL REFERENCES user_customizable_input(id),
                PRIMARY KEY (log_id, value_id)
            );
            CREATE TABLE IF NOT EXISTS log_category_3 (
                log_id   INTEGER NOT NULL REFERENCES logs(id) ON DELETE CASCADE,
                value_id INTEGER NOT NULL REFERENCES user_customizable_input(id),
                PRIMARY KEY (log_id, value_id)
            );
            CREATE TABLE IF NOT EXISTS log_category_4 (
                log_id   INTEGER NOT NULL REFERENCES logs(id) ON DELETE CASCADE,
                value_id INTEGER NOT NULL REFERENCES user_customizable_input(id),
                PRIMARY KEY (log_id, value_id)
            );
        ")?;

        // Migrate existing single-value data only if the old columns still exist
        let cols: Vec<String> = {
            let mut stmt = conn.prepare("PRAGMA table_info(logs)")?;
            let x: Result<Vec<String>> = stmt.query_map([], |r| r.get::<_, String>(1))?.collect();
            x?
        };
        if cols.iter().any(|c| c == "category1_id") {
            conn.execute_batch("
                INSERT OR IGNORE INTO log_category_1 (log_id, value_id)
                    SELECT id, category1_id FROM logs WHERE category1_id IS NOT NULL;
                INSERT OR IGNORE INTO log_category_2 (log_id, value_id)
                    SELECT id, category2_id FROM logs WHERE category2_id IS NOT NULL;
                INSERT OR IGNORE INTO log_category_3 (log_id, value_id)
                    SELECT id, category3_id FROM logs WHERE category3_id IS NOT NULL;
                INSERT OR IGNORE INTO log_category_4 (log_id, value_id)
                    SELECT id, category4_id FROM logs WHERE category4_id IS NOT NULL;

                ALTER TABLE logs DROP COLUMN category1_id;
                ALTER TABLE logs DROP COLUMN category2_id;
                ALTER TABLE logs DROP COLUMN category3_id;
                ALTER TABLE logs DROP COLUMN category4_id;
            ")?;
        }

        conn.execute_batch("
            PRAGMA foreign_keys = ON;
            PRAGMA user_version = 2;
        ")?;
    }

    if version < 3 {
        conn.execute_batch("
            PRAGMA foreign_keys = OFF;

            CREATE TABLE IF NOT EXISTS projects (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                title       TEXT    NOT NULL,
                description TEXT,
                parent_id   INTEGER REFERENCES projects(id)
            );

            ALTER TABLE logs ADD COLUMN project_id INTEGER REFERENCES projects(id);
        ")?;

        // Migrate parent_id from logs → projects if column still exists
        let cols: Vec<String> = {
            let mut stmt = conn.prepare("PRAGMA table_info(logs)")?;
            let x: Result<Vec<String>> = stmt.query_map([], |r| r.get::<_, String>(1))?.collect();
            x?
        };
        if cols.iter().any(|c| c == "parent_id") {
            conn.execute_batch("
                INSERT INTO projects (title)
                    SELECT DISTINCT l2.title FROM logs l1
                    JOIN logs l2 ON l1.parent_id = l2.id;

                UPDATE logs SET project_id = (
                    SELECT p.id FROM projects p
                    JOIN logs parent ON parent.title = p.title
                    WHERE parent.id = logs.parent_id
                ) WHERE parent_id IS NOT NULL;

                ALTER TABLE logs DROP COLUMN parent_id;
            ")?;
        }

        conn.execute_batch("
            PRAGMA foreign_keys = ON;
            PRAGMA user_version = 3;
        ")?;
    }

    if version < 4 {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS project_category_1 (
                project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                value_id   INTEGER NOT NULL REFERENCES user_customizable_input(id),
                PRIMARY KEY (project_id, value_id)
            );
            CREATE TABLE IF NOT EXISTS project_category_2 (
                project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                value_id   INTEGER NOT NULL REFERENCES user_customizable_input(id),
                PRIMARY KEY (project_id, value_id)
            );
            CREATE TABLE IF NOT EXISTS project_category_3 (
                project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                value_id   INTEGER NOT NULL REFERENCES user_customizable_input(id),
                PRIMARY KEY (project_id, value_id)
            );
            CREATE TABLE IF NOT EXISTS project_category_4 (
                project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                value_id   INTEGER NOT NULL REFERENCES user_customizable_input(id),
                PRIMARY KEY (project_id, value_id)
            );
            PRAGMA user_version = 4;
        ")?;
    }

    Ok(())
}
