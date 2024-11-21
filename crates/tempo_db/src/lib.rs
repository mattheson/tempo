use anyhow::Result;
use dashmap::DashMap;
use sqlx::{pool::PoolConnection, Sqlite, SqlitePool};
use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};
use tempo_misc::path_to_str;

struct DbManager(DashMap<PathBuf, SqlitePool>);

impl DbManager {
    pub fn new() -> Self {
        Self(DashMap::new())
    }

    pub async fn get(&self, path: &Path) -> Result<SqlitePool> {
        Ok(self
            .0
            .entry(path.to_path_buf())
            .or_insert(SqlitePool::connect(&format!("sqlite:{}", path_to_str(path))).await?)
            .value()
            .clone())
    }

    // TODO auto clean up of pools with no connections
}

static DBS: LazyLock<DbManager> = LazyLock::new(DbManager::new);

/// Handle to Tempo's sqlite database stored in the data directory.
pub struct DbHandle {
    conn: PoolConnection<Sqlite>,
}

impl DbHandle {
    pub async fn new(db_path: &Path, namespace: &str) -> Result<Self> {
        Ok(Self {
            conn: DBS.get(db_path).await?.acquire().await?,
        })
    }
}

/*

info -----
install ulid | maybe other stuff

folders -----
id | ns | last scan time? | json meta of doc | last meta doc heads

notes -----
id | folder id | json | last doc hash | last head

notifications ----
type | folder | channel | note | comment

users ----

latter 3 optional

types
- new note
- new comment

i need the last known head to generate patchlog for notifications. think it should work

 */
