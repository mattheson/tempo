use super::DbConnection;

/// Holds all of Tempo's open SQLite connections.
pub struct Dbs {
    tempo_db_path: std::path::PathBuf,
    tempo_db: DbConnection,

    other: dashmap::DashMap<std::path::PathBuf, DbConnection>,
}

impl Dbs {
    pub fn new<P>(data_dir: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        let tempo_db_path = data_dir.as_ref().join("tempo.sqlite");
        let mut conn = rusqlite::Connection::open(&tempo_db_path)?;

        crate::schema::setup_tempo_db(&mut conn)?;

        let tempo_db = std::sync::Arc::new(tokio::sync::Mutex::new(conn));

        Ok(Self {
            tempo_db_path,
            tempo_db,
            other: dashmap::DashMap::new(),
        })
    }

    pub fn get(&self) -> crate::Db {
        crate::Db(self.tempo_db.clone())
    }

    pub fn get_other<P>(&self, path: P) -> anyhow::Result<DbConnection>
    where
        P: AsRef<std::path::Path>,
    {
        Ok(self
            .other
            .entry(path.as_ref().to_path_buf())
            .or_insert_with(|| {
                std::sync::Arc::new(tokio::sync::Mutex::new(
                    rusqlite::Connection::open(path).unwrap(),
                ))
            })
            .clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test]
    pub fn create_and_load_db() {
        let dir = tempo_test::get_temp_dir("create_and_load_db").unwrap();
        let f = || {
            let _dbs = Dbs::new(dir.path()).unwrap();
        };
        f();
        f();
    }
}
