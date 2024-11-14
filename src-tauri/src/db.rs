use crate::misc::path_to_str;
use anyhow::Result;
use dashmap::DashMap;
use sqlx::{pool::PoolConnection, Sqlite, SqlitePool};
use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

struct DbManager(LazyLock<DashMap<PathBuf, SqlitePool>>);

impl DbManager {
    pub fn new() -> Self {
        Self(LazyLock::new(|| DashMap::new()))
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

static DBS: LazyLock<DbManager> = LazyLock::new(|| DbManager::new());

/// Handle to Tempo's sqlite database stored in the data directory.
pub struct TempoDb {
    conn: PoolConnection<Sqlite>,
}

impl TempoDb {
    pub async fn new(db_path: &Path) -> Result<Self> {
        Ok(Self {
            conn: DBS.get(db_path).await?.acquire().await?,
        })
    }
}
