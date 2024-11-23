use rusqlite::Connection;

/*

misc
schema number | uuid | frontend kv store

sessions
id | provider | provider id | name

notes
id | latest (id of latest edit) | folder id | provider note id | creator uuid | channel note id (optional) | new (bool)

ancestry
parent id | child id

plugin_ids
id | format-specific id | name | vendor

 */

pub fn load(conn: &mut Connection) -> anyhow::Result<()> {
    Ok(())
}

const SQL_SCHEMA: u32 = 0;

pub fn get_schema(conn: &mut Connection) -> anyhow::Result<u32> {
    Ok(conn.query_row("SELECT schema FROM misc", [], |row| row.get(0))?)
}

pub fn schema_0(conn: &mut Connection) -> anyhow::Result<()> {
    conn.execute(
        r#"

CREATE TABLE IF NOT EXISTS misc (
    schema INTEGER NOT NULL,
    uuid TEXT NOT NULL,

    -- json store for frontend
    store BLOB
);

CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER AUTOINCREMENT,
    provider_id BLOB NOT NULL,

    provider_ns TEXT NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS notes (
    id INTEGER AUTOINCREMENT,
    provider_id BLOB NOT NULL,
    session_id INTEGER NOT NULL,

    -- unix time
    time INTEGER NOT NULL,
    creator_uuid BLOB NOT NULL,

    -- note id of latest edit of this note
    latest INTEGER,

    -- note id of channel note
    in_channel INTEGER,

    channel BOOLEAN NOT NULL,
    new BOOLEAN NOT NULL,

    FOREIGN KEY (session_id) REFERENCES sessions(id),
    FOREIGN KEY (latest) REFERENCES notes(id),
    FOREIGN KEY (in_channel) REFERENCES notes(id)
);

CREATE TABLE IF NOT EXISTS ancestry (
    parent INTEGER NOT NULL,
    child INTEGER NOT NULL,

    FOREIGN KEY (parent) REFERENCES notes(id),
    FOREIGN KEY (child) REFERENCES notes(id),
    UNIQUE (parent, child),
    CHECK (parent != child)
);

CREATE TABLE IF NOT EXISTS plugin_ids (
    id INTEGER AUTOINCREMENT,

    type TEXT NOT NULL,
    plugin_id BLOB NOT NULL,
    name TEXT NOT NULL,
    vendor TEXT NOT NULL
);

CREATE TRIGGER IF NOT EXISTS check_no_cycles 
AFTER INSERT ON ancestry
BEGIN
    WITH RECURSIVE

    cycle_check(original_parent, current_node, depth) AS (
        SELECT parent, child, 1
        FROM ancestry
        WHERE parent = NEW.parent AND child = NEW.child
        
        UNION ALL
        
        -- Recursively follow the chain
        SELECT cc.original_parent, a.child, cc.depth + 1
        FROM cycle_check cc
        JOIN ancestry a ON a.parent = cc.current_node
        WHERE depth < (SELECT COUNT(*) FROM notes) -- Prevent infinite recursion
    )

    SELECT RAISE(ROLLBACK, 'Cycle detected in ancestry')
    WHERE EXISTS (
        SELECT 1 
        FROM cycle_check 
        WHERE current_node = original_parent
    );
END;

    "#,
        [],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;
    use tempo_test::get_temp_dir;

    #[test]
    pub fn create_db() {
        let dir = get_temp_dir("create_db").unwrap();
    }
}