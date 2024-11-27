mod schema;

use rusqlite::Connection;

// TODO does this really need to be a dashmap?

/// Holds all of Tempo's open SQLite connections
pub struct Dbs(
    dashmap::DashMap<std::path::PathBuf, std::sync::Arc<tokio::sync::Mutex<Connection>>>,
);

impl Dbs {
    fn get<P>(&self, path: P) -> anyhow::Result<std::sync::Arc<tokio::sync::Mutex<Connection>>>
    where
        P: AsRef<std::path::Path>,
    {
        Ok(self
            .0
            .entry(path.as_ref().to_path_buf())
            .or_insert_with(|| {
                std::sync::Arc::new(tokio::sync::Mutex::new(
                    rusqlite::Connection::open(path).unwrap(),
                ))
            })
            .clone())
    }
}

pub struct Db(std::sync::Arc<tokio::sync::Mutex<Connection>>);

impl Db {
    pub async fn get_store(&self) -> anyhow::Result<String> {
        Ok(self
            .0
            .lock()
            .await
            .query_row("SELECT store FROM misc", [], |row| row.get(0))?)
    }

    pub async fn set_store(&self, json: &str) -> anyhow::Result<()> {
        self.0.lock().await.execute("INSERT INTO misc", rusqlite::params![json])?;
        Ok(())
    }
}
