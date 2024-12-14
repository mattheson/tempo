use rusqlite::{Connection, OptionalExtension};

// pub const SQL_DB_NAME: &str = "tempo.sqlite";
const SQL_SCHEMA: u32 = 0;

/// Sets up Tempo's SQLite database, given a connection to the database.
/// Performs initial setup/migrations if needed.
pub fn setup_tempo_db(conn: &mut Connection) -> anyhow::Result<()> {
    if let Some(schema) = get_schema(conn)? {
        if schema != SQL_SCHEMA {
            log::error!("SQL schema does not match! Expected {SQL_SCHEMA}, found {schema}");
            // TODO migrations and better error to user here
            panic!("SQL migrations unimplemented! Expected {SQL_SCHEMA}, found {schema}");
        }
    } else {
        setup_schema_0(conn)?;
    }
    Ok(())
}

pub fn get_schema(conn: &mut Connection) -> anyhow::Result<Option<u32>> {
    if conn
        .query_row(
            "SELECT name FROM sqlite_master WHERE name='misc'",
            [],
            |_| Ok(()),
        )
        .optional()?
        .is_none()
    {
        Ok(None)
    } else {
        Ok(conn
            .query_row("SELECT schema FROM misc", [], |row| row.get(0))
            .optional()?)
    }
}

pub fn setup_schema_0(conn: &mut Connection) -> anyhow::Result<()> {
    conn.execute(
        r#"
 
CREATE TABLE IF NOT EXISTS misc (
    id INTEGER PRIMARY KEY CHECK (id = 0),
    schema INTEGER NOT NULL,
    uuid TEXT NOT NULL,

    -- json store for frontend
    store TEXT
);

CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    provider_ns TEXT NOT NULL,
    provider_id TEXT NOT NULL,

    name TEXT NOT NULL,

    UNIQUE (provider_ns, provider_id)
);

CREATE TABLE IF NOT EXISTS objects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    session_id INTEGER NOT NULL,

    unix_time INTEGER NOT NULL,
    creator_uuid TEXT NOT NULL,

    -- note id of latest edit of this note
    latest INTEGER,

    -- note id of channel note
    in_channel INTEGER,

    -- whether notification should be displayed for this note
    new BOOLEAN NOT NULL,

    FOREIGN KEY (session_id) REFERENCES sessions(id),
    FOREIGN KEY (latest) REFERENCES notes(id),
    FOREIGN KEY (in_channel) REFERENCES notes(id)
);

CREATE TABLE IF NOT EXISTS fs_sessions (
    session_id INTEGER NOT NULL PRIMARY KEY,

    last_scan INTEGER NOT NULL,
    msg_num INTEGER NOT NULL,

    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

CREATE TABLE IF NOT EXISTS fs_users (
    session_id INTEGER NOT NULL,
    uuid TEXT NOT NULL,

    missing JSON NOT NULL,
    total_num INTEGER NOT NULL,
    last_msg INTEGER NOT NULL,

    UNIQUE (session_id, uuid)
);

CREATE TABLE IF NOT EXISTS fs_notes (
    note_id INTEGER NOT NULL PRIMARY KEY,

    msg_num INTEGER NOT NULL,
    lamport_time INTEGER NOT NULL,

    FOREIGN KEY (note_id) REFERENCES notes(id)
);

CREATE TABLE IF NOT EXISTS ancestry (
    parent INTEGER NOT NULL,
    child INTEGER NOT NULL,

    FOREIGN KEY (parent) REFERENCES notes(id),
    FOREIGN KEY (child) REFERENCES notes(id),
    UNIQUE (parent, child),
    CHECK (parent != child)
);

CREATE TRIGGER IF NOT EXISTS check_no_reverse_entries
BEFORE INSERT ON ancestry
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'Reverse relationship exists')
    WHERE EXISTS (
        SELECT 1 FROM ancestry
        WHERE parent = NEW.child AND child = NEW.parent
    );
END;

CREATE TABLE IF NOT EXISTS plugin_ids (
    id INTEGER AUTOINCREMENT,

    type TEXT NOT NULL,
    plugin_id BLOB NOT NULL,
    name TEXT NOT NULL,
    vendor TEXT NOT NULL
);

/*
CREATE TRIGGER IF NOT EXISTS check_no_cycles 
AFTER INSERT ON ancestry
BEGIN
    WITH RECURSIVE

    cycle_check(original_parent, current_node, depth) AS (
        SELECT parent, child, 1
        FROM ancestry
        WHERE parent = NEW.parent AND child = NEW.child
        
        UNION ALL
        
        SELECT cc.original_parent, a.child, cc.depth + 1
        FROM cycle_check cc
        JOIN ancestry a ON a.parent = cc.current_node
        WHERE depth < (SELECT COUNT(*) FROM notes)
    )

    SELECT RAISE(ROLLBACK, 'Cycle detected in ancestry')
    WHERE EXISTS (
        SELECT 1 
        FROM cycle_check 
        WHERE current_node = original_parent
    );
END;
*/

    "#,
        [],
    )?;

    let mut stmt = conn.prepare(
        r#"
INSERT INTO misc (id, schema, uuid) VALUES (0, ?1, ?2);
    "#,
    )?;

    // stmt.execute(rusqlite::params![0, tempo_id::Uuid::new().to_string()])?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test]
    pub fn create_and_load_db() {
        let dir = tempo_test::get_temp_dir("create_and_load_db").unwrap();
        let f = || {
            let mut conn = rusqlite::Connection::open(dir.path().join("tempo.sqlite")).unwrap();

            setup_tempo_db(&mut conn).unwrap();
        };
        f();
        f();
    }
}
