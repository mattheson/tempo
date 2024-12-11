use anyhow::Context;

/// Connection to Tempo's database.
pub struct Db {
    pub(crate) conn: super::DbConnection,
    pub(crate) notify: super::DbNotifier
}

impl Db {
    async fn lock(&self) -> tokio::sync::MutexGuard<'_, rusqlite::Connection> {
        self.conn.lock().await
    }

    pub async fn get_store(&self) -> anyhow::Result<String> {
        Ok(self
            .lock()
            .await
            .query_row("SELECT store FROM misc", [], |row| row.get(0))?)
    }

    pub async fn set_store(&self, json: &str) -> anyhow::Result<()> {
        let _ = serde_json::from_str::<serde_json::Value>(json)
            .context("Invalid JSON provided to set_store()")?;

        self.conn.lock().await.execute(
            "UPDATE misc SET store = ?1 WHERE id = 0",
            rusqlite::params![json],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    use test_log::test;

    fn get_dbs(prefix: &str) -> (tempo_misc::TempDir, crate::Dbs) {
        let dir = tempo_test::get_temp_dir(prefix).unwrap();
        let dbs = crate::Dbs::new(dir.path()).unwrap();
        (dir, dbs)
    }

    #[test(tokio::test)]
    pub async fn test_store() {
        let (_dir, dbs) = get_dbs("test_store");

        dbs.get().set_store("{ \"hi\": 2 }").await.unwrap();
        log::info!("got json: {}", dbs.get().get_store().await.unwrap());
    }

    // #[test(tokio::test)]
}
