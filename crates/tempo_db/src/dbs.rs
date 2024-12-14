/// Holds all of Tempo's open SQLite connections.
pub struct Dbs<R: tauri::Runtime> {
    tempo_db_path: std::path::PathBuf,
    tempo_db: super::DbConnection,

    notifier: super::DbNotifier<R>,

    // TODO drop connections when they're not being used
    other: dashmap::DashMap<std::path::PathBuf, super::DbConnection>,
}

impl<R: tauri::Runtime> Dbs<R> {
    pub fn new<P>(data_dir: P, handle: tauri::AppHandle<R>) -> anyhow::Result<Self>
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
            notifier: super::DbNotifier::new(handle),
            other: dashmap::DashMap::new(),
        })
    }

    pub fn get(&self) -> crate::Db<R> {
        crate::Db {
            conn: self.tempo_db.clone(),
            notify: self.notifier.clone()
        }
    }

    pub fn get_other<P>(&self, path: P) -> anyhow::Result<super::DbConnection>
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
        let (_app, handle) = tempo_misc::tauri_test();
        let dir = tempo_test::get_temp_dir("create_and_load_db").unwrap();
        let f = || {
            let _dbs = Dbs::new(dir.path(), handle.clone()).unwrap();
        };
        f();
        f();
    }
}
