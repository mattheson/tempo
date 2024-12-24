use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

/// Holds all of Tempo's open SQLite connections.
pub struct Dbs<R: tauri::Runtime> {
    tempo_db_path: std::path::PathBuf,

    tempo_db: super::RawConnection,

    notifier: super::DbNotifier<R>,

    other: Arc<Mutex<HashMap<String, (super::RawConnection, Instant)>>>,
}

impl<R: tauri::Runtime> Dbs<R> {
    pub async fn new<P>(data_dir: P, handle: tauri::AppHandle<R>) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        let tempo_db_path = data_dir.as_ref().join("tempo.sqlite3");
        let conn = crate::Connection::open(&tempo_db_path).await?;

        conn.call(|c| Ok(crate::schema::setup_root(c)?)).await?;

        let tempo_db = std::sync::Arc::new(conn);

        let s = Self {
            tempo_db_path,
            tempo_db,
            notifier: super::DbNotifier::new(handle),
            other: Arc::new(Mutex::new(HashMap::new())),
        };

        tokio::runtime::Handle::current().spawn(cleanup_task(Arc::downgrade(&s.other)));

        Ok(s)
    }

    pub fn get(&self) -> crate::Db<R> {
        crate::Db {
            conn: self.tempo_db.clone(),
            notify: self.notifier.clone(),
        }
    }

    pub async fn get_other<P>(&self, path: P) -> anyhow::Result<super::RawConnection>
    where
        P: AsRef<std::path::Path>,
    {
        let p = path.as_ref().to_string_lossy().to_string();

        Ok(self
            .other
            .lock()
            .unwrap()
            .entry(p)
            .and_modify(|ent| ent.1 = Instant::now())
            .or_insert_with(|| {
                (
                    Arc::new(tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current()
                            .block_on(crate::Connection::open(path))
                            .unwrap()
                    })),
                    Instant::now(),
                )
            })
            .0
            .clone())
    }
}

async fn cleanup_task(r: Weak<Mutex<HashMap<String, (super::RawConnection, Instant)>>>) {
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;

        if let Some(m) = r.upgrade() {
            let mut lock = m.lock().unwrap();

            lock.retain(|k, (c, t)| {
                if t.elapsed() > Duration::from_secs(30) {
                    if Arc::strong_count(c) > 1 {
                        log::warn!(
                            "a connection to {k} has been held for {} seconds, ref count: {}",
                            t.elapsed().as_secs(),
                            Arc::strong_count(c)
                        );
                        true
                    } else {
                        // correctness: we're locking the entire map, no other threads could be acquiring connections at this time
                        false
                    }
                } else {
                    true
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test(tokio::test)]
    async fn create_and_load_db() {
        let (_app, handle) = tempo_misc::tauri_test();
        let dir = tempo_test::get_temp_dir("create_and_load_db").unwrap();

        let _dbs = Dbs::new(dir.path(), handle.clone()).await.unwrap();
        let _dbs = Dbs::new(dir.path(), handle.clone()).await.unwrap();
    }
}
