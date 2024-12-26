/// General purpose function for loading databases. Automatically creates tables. Migrates.
pub fn load(conn: &mut rusqlite::Connection, expected: DbTypes) -> crate::Result<()> {
    let tx = conn.transaction()?;

    match DbMeta::load(&tx)? {
        None => {
            DbMeta {
                typ: expected,
                schema: MIGRATIONS[expected as usize].len() - 1,
            }
            .init(&tx)?;
        }

        Some(mut meta) => {
            if meta.typ != expected {
                return Err(crate::Error::InvalidDb(format!(
                    "expected {expected} database, found {meta}"
                )));
            }

            meta.migrate(&tx)?;
        }
    }

    tx.commit()?;
    Ok(())
}

impl std::fmt::Display for DbTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DbTypes::*;

        f.write_str(match self {
            Root => "root",
            Tree => "tree",
            Note => "note",
            Shared => "shared",
        })
    }
}

impl TryFrom<&str> for DbTypes {
    type Error = crate::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use DbTypes::*;

        Ok(match value {
            "root" => Root,
            "tree" => Tree,
            "note" => Note,
            "shared" => Shared,
            _ => return Err(crate::Error::UnknownDbType(value.to_string())),
        })
    }
}

impl rusqlite::types::ToSql for DbTypes {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Owned(
            rusqlite::types::Value::Text(self.to_string()),
        ))
    }
}

impl rusqlite::types::FromSql for DbTypes {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        Self::try_from(value.as_str()?)
            .map_err(|e| rusqlite::types::FromSqlError::Other(Box::new(e)))
    }
}

#[repr(usize)]
#[derive(PartialEq, Clone, Copy)]
pub enum DbTypes {
    Root = 0,
    Tree = 1,
    Note = 2,
    Shared = 3,
}

type Migrations = &'static [&'static [fn(&rusqlite::Connection) -> crate::Result<()>]];

// schema number = idx of function last ran in here
// everything past 0 should be a migration
const MIGRATIONS: Migrations = &[&[root::init]];

pub(crate) struct DbMeta {
    schema: usize,
    typ: DbTypes,
}

impl std::fmt::Display for DbMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tempo {} database, schema {}", self.typ, self.schema)
    }
}

impl DbMeta {
    pub(crate) fn load(conn: &rusqlite::Connection) -> crate::Result<Option<Self>> {
        use rusqlite::OptionalExtension;
        if (conn
            .query_row(
                "SELECT name FROM sqlite_master WHERE name='tempo_info'",
                [],
                |_| Ok(()),
            )
            .optional()?)
        .is_none()
        {
            return Ok(None);
        }

        match conn.query_row("SELECT schema, typ FROM tempo_info", [], |row| {
            Ok((row.get(0)?, row.get(1)?))
        }) {
            Ok((schema, typ)) => Ok(Some(Self { schema, typ })),
            Err(e) => Err(crate::Error::InvalidDb(e.to_string())),
        }
    }

    pub(crate) fn init(&self, conn: &rusqlite::Connection) -> crate::Result<()> {
        conn.execute(
            r#"
            CREATE TABLE tempo_info (
                id INTEGER PRIMARY KEY CHECK (id = 0),
                schema INTEGER NOT NULL,
                typ STRING NOT NULL
            );
        "#,
            [],
        )?;

        conn.execute(
            r#"
INSERT INTO tempo_info (id, schema, typ) VALUES (0, ?1, ?2);
        "#,
            rusqlite::params![self.schema, self.typ],
        )?;

        for m in MIGRATIONS[self.typ as usize][0..=self.schema].iter() {
            m(conn)?;
        }

        Ok(())
    }

    pub(crate) fn migrate(&mut self, conn: &rusqlite::Connection) -> crate::Result<()> {
        let migrations = MIGRATIONS[self.typ as usize];

        if self.schema == migrations.len() - 1 {
            return Ok(());
        }

        if self.schema > migrations.len() - 1 {
            return Err(crate::Error::InvalidDb(format!(
                "unexpected {self}, only know up to migration {}",
                MIGRATIONS[self.typ as usize].len()
            )));
        }

        for m in migrations[self.schema..=migrations.len() - 1].iter() {
            m(conn)?;
        }

        self.schema = migrations.len() - 1;

        Ok(())
    }
}

pub mod root {
    pub fn init(conn: &rusqlite::Connection) -> crate::Result<()> {
        conn.execute(
            r#"
            CREATE TABLE misc (
                id INTEGER PRIMARY KEY CHECK (id = 0),
                schema INTEGER NOT NULL,
                uuid TEXT NOT NULL,
                notes_dir TEXT,
            
                -- json store for frontend
                store TEXT
            );


            "#,
            [],
        )?;

        conn.execute(
            r#"
            CREATE TABLE trees (
                id INTEGER PRIMARY KEY AUTOINCREMENT,

                path TEXT NOT NULL,

                name TEXT NOT NULL,

                UNIQUE(path)
            );
            "#,
            [],
        )?;

        Ok(())
    }

    pub fn add_tree(
        conn: &rusqlite::Connection,
        path: impl AsRef<std::path::Path>,
        name: &str,
    ) -> crate::Result<()> {
        conn.execute(
            r#"
        INSERT INTO trees (path, name) VALUES (?1, ?2);
        "#,
            rusqlite::params![path.as_ref().to_string_lossy(), name],
        )?;
        Ok(())
    }

    pub fn remove_tree(
        conn: &rusqlite::Connection,
        path: impl AsRef<std::path::Path>,
    ) -> crate::Result<()> {
        conn.execute(
            "DELETE FROM trees WHERE path = ?1",
            rusqlite::params![path.as_ref().to_string_lossy()],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test]
    pub fn test() {
        let dir = tempo_test::get_temp_dir("create_and_load_db").unwrap();
        let f = || {
            let mut conn = rusqlite::Connection::open(dir.path().join("tempo.sqlite3")).unwrap();
            load(&mut conn, DbTypes::Root).unwrap();
        };
        f();
        f();
        let mut conn = rusqlite::Connection::open(dir.path().join("tempo.sqlite3")).unwrap();
        load(&mut conn, DbTypes::Root).unwrap();
        root::add_tree(&conn, "/tmp/asdfasdf", "test").unwrap();
        root::remove_tree(&conn, "/tmp/asdfasdf").unwrap();
    }
}

// random old sql

/*
CREATE TABLE IF NOT EXISTS objects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    library_id INTEGER NOT NULL,

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
*/

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
