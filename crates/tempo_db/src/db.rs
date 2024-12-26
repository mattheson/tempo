use crate::{schema::DbTypes, Connection, Result};
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct DbInner<R: tauri::Runtime> {
    pub(crate) notify: super::DbNotifier<R>,
    pub(crate) conn: Connection,
}

#[derive(Debug)]
pub struct RootDb<R: tauri::Runtime>(DbInner<R>);

impl<R: tauri::Runtime> RootDb<R> {
    pub async fn new(
        path: impl AsRef<std::path::Path>,
        notify: super::DbNotifier<R>,
    ) -> Result<Self> {
        let conn = Connection::open(&path).await?;

        conn.call(|conn| Ok(crate::schema::load(conn, DbTypes::Root)?))
            .await?;

        Ok(Self(DbInner { notify, conn }))
    }

    /// This will immediately start indexing.
    /// `tree_root` is the root directory of the tree.
    pub async fn add_tree(
        &self,
        tree_root: impl AsRef<std::path::Path>,
    ) -> Result<()> {
        todo!()
    }

    pub async fn remove_tree(
        &self,
        tree_root: impl AsRef<std::path::Path>,
    ) -> Result<()> {
        todo!()
    }
}

/// Tree database. Stores info about files in the provided directory. Local-only for now.
#[derive(Debug)]
pub struct TreeDb<R: tauri::Runtime> {
    path: std::path::PathBuf,
    db: DbInner<R>,
}

impl<R: tauri::Runtime> TreeDb<R> {
    /// `root_dir` is the root directory of this file tree.
    /// Fails if the directory is missing.
    pub async fn new(
        root_dir: impl AsRef<std::path::Path>,
        notify: super::DbNotifier<R>,
    ) -> Result<Self> {
        if !std::fs::exists(root_dir.as_ref())? {
            return Err(crate::Error::MissingTree(root_dir.as_ref().to_path_buf()));
        }

        let conn = Connection::open(&root_dir.as_ref().join("tempo-db.sqlite3")).await?;

        conn.call(|conn| Ok(crate::schema::load(conn, DbTypes::Tree)?))
            .await?;

        Ok(Self {
            db: DbInner { notify, conn },
            path: root_dir.as_ref().to_path_buf(),
        })
    }

    pub async fn reindex(&self) -> Result<()> {
        todo!()
    }
}

// /// Connection to Tempo's database.
// pub struct Db<R>
// where
//     R: tauri::Runtime,
// {
//     pub(crate) conn: super::RawConnection,
//     pub(crate) notify: super::DbNotifier<R>,
// }

// impl<R: tauri::Runtime> Db<R> {
//     pub async fn call<F, T>(&self, function: F) -> anyhow::Result<T>
//     where
//         F: FnOnce(&mut rusqlite::Connection) -> Result<T, crate::ConnectionError> + 'static + Send,
//         T: Send + 'static,
//     {
//         Ok(self.conn.call(function).await?)
//     }

//     pub async fn get_store(&self) -> anyhow::Result<String> {
//         self.call(|c| Ok(c.query_row("SELECT store FROM misc", [], |row| row.get(0))?))
//             .await
//     }

//     pub async fn set_store(&self, json: &str) -> anyhow::Result<()> {
//         let json = json.to_string();

//         let _ = serde_json::from_str::<serde_json::Value>(&json)
//             .context("invalid JSON provided to set_store()")?;

//         let _ = self.call(move |c| {
//             Ok(c.execute(
//                 "UPDATE misc SET store = ?1 WHERE id = 0",
//                 rusqlite::params![json],
//             ))
//         })
//         .await?;

//         Ok(())
//     }
// }

// #[cfg(test)]
// mod tests {
//     // use super::*;
//     use test_log::test;

//     struct DbTest {
//         pub app: tauri::App<tauri::test::MockRuntime>,
//         pub handle: tauri::AppHandle<tauri::test::MockRuntime>,
//         pub dir: tempo_misc::TempDir,
//         pub dbs: crate::Dbs<tauri::test::MockRuntime>,
//     }

//     impl DbTest {
//         async fn new(prefix: &str) -> DbTest {
//             let (app, handle) = tempo_misc::tauri_test();
//             let dir = tempo_test::get_temp_dir(prefix).unwrap();
//             let dbs = crate::Dbs::new(dir.path(), handle.clone()).await.unwrap();
//             Self {
//                 app,
//                 handle,
//                 dir,
//                 dbs,
//             }
//         }
//     }

//     #[test(tokio::test)]
//     pub async fn test_store() {
//         let t = DbTest::new("test_store").await;

//         t.dbs.get().set_store("{ \"hi\": 2 }").await.unwrap();
//         log::info!("got json: {}", t.dbs.get().get_store().await.unwrap());
//     }

//     // #[test(tokio::test)]
// }
