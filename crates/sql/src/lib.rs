// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod commands;
mod decode;
mod error;
mod wrapper;

pub use error::Error;
pub use wrapper::DbPool;

use futures_core::future::BoxFuture;
use serde::{Deserialize, Serialize};
use sqlx::{
    error::BoxDynError,
    migrate::{Migration as SqlxMigration, MigrationSource, MigrationType, Migrator},
};
use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Manager, RunEvent, Runtime,
};
use tokio::sync::{Mutex, RwLock};

use std::collections::HashMap;

#[derive(Default)]
pub struct DbInstances(pub RwLock<HashMap<String, DbPool>>);

pub type LastInsertId = i64;

// struct Migrations(Mutex<HashMap<String, MigrationList>>);

#[derive(Default, Clone, Deserialize)]
pub struct PluginConfig {
    #[serde(default)]
    preload: Vec<String>,
}

// #[derive(Debug)]
// pub enum MigrationKind {
//     Up,
//     Down,
// }

// impl From<MigrationKind> for MigrationType {
//     fn from(kind: MigrationKind) -> Self {
//         match kind {
//             MigrationKind::Up => Self::ReversibleUp,
//             MigrationKind::Down => Self::ReversibleDown,
//         }
//     }
// }

// /// A migration definition.
// #[derive(Debug)]
// pub struct Migration {
//     pub version: i64,
//     pub description: &'static str,
//     pub sql: &'static str,
//     pub kind: MigrationKind,
// }

// #[derive(Debug)]
// struct MigrationList(Vec<Migration>);

// impl MigrationSource<'static> for MigrationList {
//     fn resolve(self) -> BoxFuture<'static, std::result::Result<Vec<SqlxMigration>, BoxDynError>> {
//         Box::pin(async move {
//             let mut migrations = Vec::new();
//             for migration in self.0 {
//                 if matches!(migration.kind, MigrationKind::Up) {
//                     migrations.push(SqlxMigration::new(
//                         migration.version,
//                         migration.description.into(),
//                         migration.kind.into(),
//                         migration.sql.into(),
//                         false,
//                     ));
//                 }
//             }
//             Ok(migrations)
//         })
//     }
// }

/// Allows blocking on async code without creating a nested runtime.
fn run_async_command<F: std::future::Future>(cmd: F) -> F::Output {
    if tokio::runtime::Handle::try_current().is_ok() {
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(cmd))
    } else {
        tauri::async_runtime::block_on(cmd)
    }
}

/// Tauri SQL plugin builder.
#[derive(Default)]
pub struct Builder {
    // migrations: Option<HashMap<String, MigrationList>>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    // /// Add migrations to a database.
    // #[must_use]
    // pub fn add_migrations(mut self, db_url: &str, migrations: Vec<Migration>) -> Self {
    //     self.migrations
    //         .get_or_insert(Default::default())
    //         .insert(db_url.to_string(), MigrationList(migrations));
    //     self
    // }

    pub fn build<R: Runtime>(mut self) -> TauriPlugin<R, Option<PluginConfig>> {
        PluginBuilder::<R, Option<PluginConfig>>::new("sql")
            .invoke_handler(tauri::generate_handler![
                commands::load,
                commands::execute,
                commands::select,
                commands::close
            ])
            .setup(|app, api| {
                let config = api.config().clone().unwrap_or_default();

                run_async_command(async move {
                    let instances = DbInstances::default();
                    let mut lock = instances.0.write().await;

                    for db in config.preload {
                        let pool = DbPool::connect(&db, app).await?;

                        // if let Some(migrations) =
                        //     self.migrations.as_mut().and_then(|mm| mm.remove(&db))
                        // {
                        //     let migrator = Migrator::new(migrations).await?;
                        //     pool.migrate(&migrator).await?;
                        // }

                        lock.insert(db, pool);
                    }
                    drop(lock);

                    app.manage(instances);
                    // app.manage(Migrations(Mutex::new(
                    //     self.migrations.take().unwrap_or_default(),
                    // )));

                    Ok(())
                })
            })
            .on_event(|app, event| {
                if let RunEvent::Exit = event {
                    run_async_command(async move {
                        let instances = &*app.state::<DbInstances>();
                        let instances = instances.0.read().await;
                        for value in instances.values() {
                            value.close().await;
                        }
                    });
                }
            })
            .build()
    }
}


// /// Handle to Tempo's sqlite database stored in the data directory.
// pub struct DbHandle {
//     conn: sqlx::PoolConnection<Sqlite>,
// }

// impl DbHandle {
//     pub async fn new(db_path: &Path, namespace: &str) -> Result<Self> {
//         Ok(Self {
//             conn: DBS.get(db_path).await?.acquire().await?,
//         })
//     }
// }

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