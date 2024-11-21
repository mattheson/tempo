// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::fs::create_dir_all;
use std::path::Path;

use indexmap::IndexMap;
use serde_json::Value as JsonValue;
use sqlx::{/*migrate::MigrateDatabase*/ Column, Executor, Pool, Row};
use tauri::{AppHandle, Runtime};

use sqlx::Sqlite;

use crate::LastInsertId;

pub struct DbPool(Pool<Sqlite>);

// private methods
impl DbPool {
    pub(crate) async fn connect<R: Runtime, P>(
        data_dir: P,
        _app: &AppHandle<R>,
    ) -> Result<Self, crate::Error>
    where
        P: AsRef<Path>,
    {
        create_dir_all(&data_dir).expect("Couldn't create data directory");

        let conn_url = &format!(
            "sqlite:{}",
            data_dir.as_ref().join("tempo.sqlite").to_string_lossy()
        );

        // if !Sqlite::database_exists(conn_url).await.unwrap_or(false) {
        //     Sqlite::create_database(conn_url).await?;
        // }

        Ok(Self(Pool::connect(conn_url).await?))
    }

    // pub(crate) async fn migrate(
    //     &self,
    //     _migrator: &sqlx::migrate::Migrator,
    // ) -> Result<(), crate::Error> {
    //     match self {
    //         #[cfg(feature = "sqlite")]
    //         DbPool::Sqlite(pool) => _migrator.run(pool).await?,
    //         #[cfg(feature = "mysql")]
    //         DbPool::MySql(pool) => _migrator.run(pool).await?,
    //         #[cfg(feature = "postgres")]
    //         DbPool::Postgres(pool) => _migrator.run(pool).await?,
    //         #[cfg(not(any(feature = "sqlite", feature = "mysql", feature = "postgres")))]
    //         DbPool::None => (),
    //     }
    //     Ok(())
    // }

    pub(crate) async fn close(&self) {
        self.0.close().await
    }

    pub(crate) async fn execute(
        &self,
        _query: String,
        _values: Vec<JsonValue>,
    ) -> Result<(u64, LastInsertId), crate::Error> {
        let mut query = sqlx::query(&_query);
        for value in _values {
            if value.is_null() {
                query = query.bind(None::<JsonValue>);
            } else if value.is_string() {
                query = query.bind(value.as_str().unwrap().to_owned())
            } else if let Some(number) = value.as_number() {
                query = query.bind(number.as_f64().unwrap_or_default())
            } else {
                query = query.bind(value);
            }
        }
        let result = self.0.execute(query).await?;
        Ok((result.rows_affected(), result.last_insert_rowid()))
    }

    pub(crate) async fn select(
        &self,
        _query: String,
        _values: Vec<JsonValue>,
    ) -> Result<Vec<IndexMap<String, JsonValue>>, crate::Error> {
        let mut query = sqlx::query(&_query);

        for value in _values {
            if value.is_null() {
                query = query.bind(None::<JsonValue>);
            } else if value.is_string() {
                query = query.bind(value.as_str().unwrap().to_owned())
            } else if let Some(number) = value.as_number() {
                query = query.bind(number.as_f64().unwrap_or_default())
            } else {
                query = query.bind(value);
            }
        }

        let rows = self.0.fetch_all(query).await?;
        let mut values = Vec::new();
        for row in rows {
            let mut value = IndexMap::default();
            for (i, column) in row.columns().iter().enumerate() {
                let v = row.try_get_raw(i)?;

                let v = crate::decode::to_json(v)?;

                value.insert(column.name().to_string(), v);
            }

            values.push(value);
        }
        Ok(values)
    }
}
