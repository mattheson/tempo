use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

/// Holds all of Tempo's open SQLite connections.
#[derive(Debug)]
pub struct Dbs<R: tauri::Runtime> {
    root_path: std::path::PathBuf,
    root: super::RootDb<R>,

    notifier: super::DbNotifier<R>,

    // we hold open connections to everything we know for now

    trees: Arc<Mutex<HashMap<String, super::Connection>>>,
}

impl<R: tauri::Runtime> Dbs<R> {
    pub async fn open(
        data_dir: impl AsRef<std::path::Path>,
        handle: tauri::AppHandle<R>,
    ) -> crate::Result<Self> {
        std::fs::create_dir_all(&data_dir)?;

        let notifier = super::DbNotifier::new(handle);

        let root_path = data_dir.as_ref().join("tempo.sqlite3");
        let root = super::RootDb::new(&root_path, notifier.clone()).await?;

        Ok(Self {
            root,
            root_path,
            notifier,
            trees: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}

//     pub fn get(&self) -> crate::Db<R> {
//         crate::Db {
//             conn: self.tempo_db.clone(),
//             notify: self.notifier.clone(),
//         }
//     }

//     pub async fn get_other<P>(&self, path: P) -> anyhow::Result<super::RawConnection>
//     where
//         P: AsRef<std::path::Path>,
//     {
//         let p = path.as_ref().to_string_lossy().to_string();

//         Ok(self
//             .other
//             .lock()
//             .unwrap()
//             .entry(p)
//             .and_modify(|ent| ent.1 = Instant::now())
//             .or_insert_with(|| {
//                 (
//                     Arc::new(tokio::task::block_in_place(|| {
//                         tokio::runtime::Handle::current()
//                             .block_on(crate::Connection::open(path))
//                             .unwrap()
//                     })),
//                     Instant::now(),
//                 )
//             })
//             .0
//             .clone())
//     }
// }

// async fn cleanup_task(r: Weak<Mutex<HashMap<String, (super::RawConnection, Instant)>>>) {
//     loop {
//         tokio::time::sleep(Duration::from_secs(30)).await;

//         if let Some(m) = r.upgrade() {
//             let mut lock = m.lock().unwrap();

//             lock.retain(|k, (c, t)| {
//                 if t.elapsed() > Duration::from_secs(30) {
//                     if Arc::strong_count(c) > 1 {
//                         log::warn!(
//                             "a connection to {k} has been held for {} seconds, ref count: {}",
//                             t.elapsed().as_secs(),
//                             Arc::strong_count(c)
//                         );
//                         true
//                     } else {
//                         // correctness: we're locking the entire map, no other threads could be acquiring connections at this time
//                         false
//                     }
//                 } else {
//                     true
//                 }
//             });
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use test_log::test;

//     #[test(tokio::test)]
//     async fn create_and_load_db() {
//         let (_app, handle) = tempo_misc::tauri_test();
//         let dir = tempo_test::get_temp_dir("create_and_load_db").unwrap();

//         let _dbs = Dbs::new(dir.path(), handle.clone()).await.unwrap();
//         let _dbs = Dbs::new(dir.path(), handle.clone()).await.unwrap();
//     }
// }
