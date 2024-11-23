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

const SQL_SCHEMA: u32 = 0;

pub fn get_schema(conn: &mut Connection) -> anyhow::Result<u32> {
    Ok(conn.query_row("SELECT...", [], |row| row.get(0))?)
}

pub fn schema_0(conn: &mut Connection) -> anyhow::Result<()> {
    conn.execute(
        r#"

CREATE TABLE IF NOT EXISTS misc (
    schema INTEGER NOT NULL,
    uuid TEXT NOT NULL,
    store TEXT
);

CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER AUTOINCREMENT,
    provider TEXT NOT NULL,
    provider_id BLOB NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS notes (
    id INTEGER AUTOINCREMENT,

    creator_uuid BLOB NOT NULL,
    session_id INTEGER NOT NULL,

    latest INTEGER,

    channel_id INTEGER,
    new BOOLEAN NOT NULL,

    FOREIGN KEY (session_id) REFERENCES sessions(id),
    FOREIGN KEY ()
);

CREATE TABLE IF NOT EXISTS ancestry (
    
);

CREATE TABLE IF NOT EXISTS ableton_plugin_ids (

);

CREATE TABLE IF NOT EXISTS ableton_plugins (
);

CREATE TABLE IF NOT EXISTS audio_unit_ids (
);

CREATE TABLE IF NOT EXISTS audio_units (
);

    "#,
        [],
    )?;
    Ok(())
}
