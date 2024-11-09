use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    daw::{
        ableton::{self, AbletonPluginRef, ScannedAbletonPlugin},
        macos::{self, AudioUnitId},
    },
    misc::{path_to_str, Result, TempoError},
    structure::{get_client_dir_path, get_client_shared_db_path, iter_clients},
};

use log::{error, info};
use rusqlite::{params, Connection, OpenFlags};
pub use schema::*;
use time::OffsetDateTime;

mod schema {
    use super::*;

    /// Creates the tables for the shared info db.
    /// This is stored as `shared.sqlite` in the data directory and is copied into Tempo folders.
    /// For now this database only contains info about plugins.
    pub fn setup_shared_schema(con: &Connection) -> Result<()> {
        con.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS info (
                ulid TEXT NOT NULL,
                scan_time DATETIME NOT NULL

                /*
                os TEXT NOT NULL,
                os_version TEXT NOT NULL
                */
            );

            CREATE TABLE IF NOT EXISTS ableton_vst (
                id INTEGER NOT NULL,

                name TEXT NOT NULL,
                vendor TEXT NOT NULL,

                UNIQUE(id, name, vendor) 
            );

            CREATE TABLE IF NOT EXISTS ableton_vst3 (
                f0 INTEGER NOT NULL,
                f1 INTEGER NOT NULL,
                f2 INTEGER NOT NULL,
                f3 INTEGER NOT NULL,

                name TEXT NOT NULL,
                vendor TEXT NOT NULL,

                UNIQUE(f0, f1, f2, f3, name, vendor)
            );

            -- this is a separate since macOS provides built in api for scanning audio units
            -- which i assume that most daws use
            -- other plugin formats need to be scanned manually by the daw

            CREATE TABLE IF NOT EXISTS audio_units (
                type INTEGER NOT NULL,
                subtype INTEGER NOT NULL,
                manufacturer INTEGER NOT NULL,

                name TEXT NOT NULL,
                vendor TEXT NOT NULL,

                UNIQUE(type, subtype, manufacturer, name, vendor)
            );

            -- we need to avoid creation of any temporary files
            PRAGMA journal_mode = DELETE;
            PRAGMA temp_store = MEMORY;
            "#,
        )?;
        Ok(())
    }

    // pub struct SharedInfoRow {
    //     pub ulid: String,
    //     pub scan_time: u64,
    // }

    pub struct SharedAbletonVstRow {
        pub id: u32,

        pub name: String,
        pub vendor: String,
    }

    pub struct SharedAbletonVst3Row {
        pub fields: [i32; 4],

        pub name: String,
        pub vendor: String,
    }

    pub struct SharedAudioUnitRow {
        pub au_type: u32,
        pub au_subtype: u32,
        pub au_manufacturer: u32,

        pub name: String,
        pub vendor: String,
    }

    #[derive(Debug)]
    pub struct PluginNameVendor {
        pub name: String,
        pub vendor: String,
    }
}

/// Info database that's stored in data directory.
/// Just contains listing of plugins.
/// Copied into Tempo folders.
pub struct SharedDb {
    db: PathBuf,
    con: Connection,
}

/// Builds the shared database at the given path. Scans plugin databases found on this system.
/// This should probably only be used to build the database inside of the data directory, then the database should be copied into folders.
pub fn scan_plugins(db: &Path, client_ulid: &str) -> Result<()> {
    let ableton_plugins = ableton::scan_plugin_db()?;
    let audio_units = macos::scan_audio_units()?;

    if db.exists() {
        info!("removing old db at {}", path_to_str(db));
        fs::remove_file(db)?;
    }

    let con = match rusqlite::Connection::open(db) {
        Ok(c) => c,
        Err(e) => {
            info!(
                "rusqlite connection failed in scanned_plugins: {}",
                e.to_string()
            );
            return Err(TempoError::from(e));
        }
    };
    info!("opened connection");

    setup_shared_schema(&con)?;

    con.execute(
        "INSERT INTO info (ulid, scan_time) VALUES (?1, ?2)",
        params![client_ulid, OffsetDateTime::now_utc()],
    )?;

    {
        let mut ableton_vst_stmt =
            con.prepare("INSERT INTO ableton_vst (id, name, vendor) VALUES (?1, ?2, ?3)")?;

        let mut ableton_vst3_stmt = con.prepare(
            "INSERT INTO ableton_vst3 (f0, f1, f2, f3, name, vendor) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;

        for p in ableton_plugins.into_iter() {
            match p {
                ScannedAbletonPlugin::Vst(SharedAbletonVstRow { id, name, vendor }) => {
                    match ableton_vst_stmt.execute(params![id, name, vendor]) {
                        Ok(_) => (),
                        Err(e) => error!("error while inserting ableton vst row: {e}"),
                    }
                }
                ScannedAbletonPlugin::Vst3(SharedAbletonVst3Row {
                    fields: [f0, f1, f2, f3],
                    name,
                    vendor,
                }) => match ableton_vst3_stmt.execute(params![f0, f1, f2, f3, name, vendor]) {
                    Ok(_) => (),
                    Err(e) => error!("error while inserting ableton vst3 row: {e}"),
                },
            }
        }
    }

    if !audio_units.is_empty() {
        let mut audio_unit_stmt = con.prepare(
            "INSERT INTO audio_units (type, subtype, manufacturer, name, vendor) VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;

        for SharedAudioUnitRow {
            au_type,
            au_subtype,
            au_manufacturer,
            name,
            vendor,
        } in audio_units.into_iter()
        {
            match audio_unit_stmt.execute(params![
                au_type,
                au_subtype,
                au_manufacturer,
                name,
                vendor
            ]) {
                Ok(_) => (),
                Err(e) => error!("error while inserting audio unit row: {e}"),
            }
        }
    }

    Ok(())
}

impl SharedDb {
    /// Opens a shared database in read-only mode. `None` if no shared database is found in the given directory.
    pub fn open(db: &Path) -> Result<Option<Self>> {
        if !fs::exists(db)? {
            return Ok(None);
        }

        Ok(Some(Self {
            con: Connection::open_with_flags(db, OpenFlags::SQLITE_OPEN_READ_ONLY)?,
            db: db.to_path_buf(),
        }))
    }

    pub fn get_last_scan_time(&self) -> Result<OffsetDateTime> {
        Ok(self
            .con
            .query_row("SELECT scan_time FROM info", [], |row| row.get(0))?)
    }

    pub fn get_ulid(&self) -> Result<String> {
        Ok(self
            .con
            .query_row("SELECT ulid FROM info", [], |row| row.get(0))?)
    }

    /// Copies the `SharedDb` into the `clients` folder of a Tempo folder.
    /// This database will be named `info.sqlite` within the client's folder.
    /// No verification is done to see if there's an existing database.
    pub fn copy_into_folder(&self, folder: &Path, username: &str) -> Result<()> {
        let db_dir = get_client_dir_path(folder, username);
        fs::create_dir_all(&db_dir)?;
        let db_path = db_dir.join("shared.sqlite");
        fs::copy(&self.db, &db_path)?;
        Ok(())
    }

    /// Returns `PluginNameVendor` if the plugin is in this db.
    pub fn get_ableton_plugin(
        &self,
        plugin: &AbletonPluginRef,
    ) -> Result<Option<PluginNameVendor>> {
        match plugin {
            AbletonPluginRef::Vst { id, name } => self.get_ableton_vst(*id, name.as_deref()),
            AbletonPluginRef::Vst3 { fields, name: _ } => self.get_ableton_vst3(*fields),
            AbletonPluginRef::Au {
                id,
                name: _,
                manufacturer: _,
            } => self.get_audio_unit(id),
        }
    }

    /// `name` optimally should be provided, it's possible that vsts have the same id
    /// if multiple rows are found, `name` will be used to try to identify the desired plugin
    fn get_ableton_vst(&self, id: u32, _name: Option<&str>) -> Result<Option<PluginNameVendor>> {
        let con = &self.con;
        let mut stmt = con.prepare("SELECT name, vendor FROM ableton_vst WHERE id = ?1")?;
        let mut rows: Vec<PluginNameVendor> = stmt
            .query_map(params![id], |row| {
                Ok(PluginNameVendor {
                    name: row.get(0)?,
                    vendor: row.get(1)?,
                })
            })?
            .filter_map(|res| match res {
                Ok(r) => Some(r),
                Err(e) => {
                    error!(
                        "get_ableton_vst(): error while looking for {id} in {}: {e}",
                        path_to_str(&self.db)
                    );
                    None
                }
            })
            .collect();

        if rows.is_empty() {
            Ok(None)
        } else if rows.len() == 1 {
            Ok(Some(rows.remove(0)))
        } else {
            Self::found_multiple_vst(id, rows)
        }
    }

    #[cold]
    fn found_multiple_vst(
        id: u32,
        rows: Vec<PluginNameVendor>,
    ) -> Result<Option<PluginNameVendor>> {
        // this might actually happen so need to handle it
        // TODO
        Err(TempoError::Db(format!(
            "Found multiple vsts matching id {id}: {:#?}",
            rows
        )))
    }

    fn get_ableton_vst3(&self, fields: [i32; 4]) -> Result<Option<PluginNameVendor>> {
        let [f0, f1, f2, f3] = fields;
        let con = &self.con;
        let mut stmt = con.prepare("SELECT name, vendor FROM ableton_vst3 WHERE f0 == ?1 AND f1 == ?2 AND f2 == ?3 and f3 == ?4")?;
        let mut rows: Vec<PluginNameVendor> = stmt
            .query_map(params![f0, f1, f2, f3], |row| {
                Ok(PluginNameVendor {
                    name: row.get(0)?,
                    vendor: row.get(1)?,
                })
            })?
            .filter_map(|res| match res {
                Ok(r) => Some(r),
                Err(e) => {
                    error!(
                        "get_ableton_vst3(): error while looking for {:#?} in {}: {e}",
                        fields,
                        path_to_str(&self.db)
                    );
                    None
                }
            })
            .collect();

        if rows.is_empty() {
            Ok(None)
        } else if rows.len() == 1 {
            Ok(Some(rows.remove(0)))
        } else {
            Err(TempoError::Db(format!(
                "Found multiple Ableton VST3s with fields {:#?}, this should not happen",
                fields
            )))
        }
    }

    fn get_audio_unit(&self, id: &AudioUnitId) -> Result<Option<PluginNameVendor>> {
        let AudioUnitId {
            au_type,
            au_subtype,
            manufacturer,
        } = id;
        let con = &self.con;
        let mut stmt = con.prepare("SELECT name, vendor FROM audio_units WHERE type == ?1 AND subtype == ?2 AND manufacturer == ?3")?;
        let mut rows: Vec<PluginNameVendor> = stmt
            .query_map(params![au_type, au_subtype, manufacturer], |row| {
                Ok(PluginNameVendor {
                    name: row.get(0)?,
                    vendor: row.get(1)?,
                })
            })?
            .filter_map(|res| match res {
                Ok(r) => Some(r),
                Err(e) => {
                    error!(
                        "get_audio_unit(): error while looking for {:#?} in {}: {e}",
                        id,
                        path_to_str(&self.db)
                    );
                    None
                }
            })
            .collect();

        if rows.is_empty() {
            Ok(None)
        } else if rows.len() == 1 {
            Ok(Some(rows.remove(0)))
        } else {
            Err(TempoError::Db(format!(
                "Found multiple Audio Units with id {:#?}, this should not happen",
                id
            )))
        }
    }
}

// /// Info databases for other clients in a folder.
// pub struct InfoDbs {
//     dbs: HashMap<String, ReadOnlyInfoDb>
// }

// impl InfoDbs {
//     /// Assumes that `folder` has been validated.
//     pub fn new(folder: &Path) -> Result<Self> {
//         for (client_dir, username) in iter_clients(folder)? {
//             let db_dir = client_dir.join(format!("{username}.sqlite"));
//             let db =
//         }
//     }
// }

pub fn iter_shared_db(folder: &Path) -> Result<impl Iterator<Item = (String, SharedDb)>> {
    let folder = folder.to_owned();

    Ok(iter_clients(&folder)?.filter_map(move |(_, username)| {
        match SharedDb::open(&get_client_shared_db_path(&folder, &username)) {
            Ok(Some(db)) => Some((username, db)),
            Ok(None) => {
                error!("iter_shared_db(): iter_clients found client {username} but failed to load db in {}", path_to_str(&folder));
                None
            }
            Err(e) => {
                error!("iter_shared_db(): SharedDb::load() error: {e}");
                None
            }
        }
    }))
}

#[cfg(test)]
mod tests {
    use crate::{misc::new_ulid, tests::get_temp_dir};

    use super::*;

    #[test_log::test]
    fn test_plugin_scan() {
        scan_plugins(
            &get_temp_dir("test_plugin_scan").join("shared.sqlite"),
            &new_ulid(),
        )
        .unwrap()
    }
}
