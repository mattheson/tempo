use anyhow::Context;

/// Connection to Tempo's database.
pub struct Db<R>
where
    R: tauri::Runtime,
{
    pub(crate) conn: super::DbConnection,
    pub(crate) notify: super::DbNotifier<R>,
}

impl<R: tauri::Runtime> Db<R> {
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
            .context("invalid JSON provided to set_store()")?;

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

    struct DbTest {
        pub app: tauri::App<tauri::test::MockRuntime>,
        pub handle: tauri::AppHandle<tauri::test::MockRuntime>,
        pub dir: tempo_misc::TempDir,
        pub dbs: crate::Dbs<tauri::test::MockRuntime>,
    }

    impl DbTest {
        fn new(prefix: &str) -> DbTest {
            let (app, handle) = tempo_misc::tauri_test();
            let dir = tempo_test::get_temp_dir(prefix).unwrap();
            let dbs = crate::Dbs::new(dir.path(), handle.clone()).unwrap();
            Self {
                app,
                handle,
                dir,
                dbs,
            }
        }
    }

    #[test(tokio::test)]
    pub async fn test_store() {
        let t = DbTest::new("test_store");

        t.dbs.get().set_store("{ \"hi\": 2 }").await.unwrap();
        log::info!("got json: {}", t.dbs.get().get_store().await.unwrap());
    }

    // #[test(tokio::test)]
}
