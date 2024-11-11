// =====================================================================================
// This file is licensed under either of
// Apache License, Version 2.0 or MIT license, at your option.
// =====================================================================================
// You may obtain a copy of the Apache License, Version 2.0 at
// http://www.apache.org/licenses/LICENSE-2.0
// =====================================================================================
// You may obtain a copy of the MIT License at
// https://opensource.org/licenses/MIT
// =====================================================================================

// ableton plugin database scanning

/*
tempo reads from Ableton's Live-files/Live-plugins databases.
we look for the "plugins" table.

in live 12 ableton added the Live-plugins database which appears to be the new location of the "plugins" table.
older versions of ableton use the Live-files database.

algorithm for locating plugin database is as follows:
- get all db entries in Live Database folder, filter any non-db entries, sort them by modification time
- move any Live-files entries to the front of our db entry Vec
 */

use std::{
    collections::HashSet,
    fs::{self, read_dir, DirEntry, ReadDir},
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::{
    db::SharedAbletonVst3Row,
    db::SharedAbletonVstRow,
    misc::{path_to_str, Result, TempoError},
};
use log::{error, info, warn};
use rusqlite::Connection;
use rusqlite::OpenFlags;

/// Checks whether there's at least one live database with a valid plugin table
pub fn have_plugin_db() -> Result<bool> {
    if !fs::exists(get_ableton_db_dir()?)? {
        return Ok(false)
    }

    for ent in get_organized_dbs()? {
        match check_db_for_plugins_table(&ent.path()) {
            Ok(true) => return Ok(true),
            Ok(false) => (),
            Err(e) => error!("have_plugin_db(): error while checking db: {:#?}", e),
        }
    }
    Ok(false)
}

/// Gets databases in Live Database directory sorted by modification time
fn get_sorted_dbs() -> Result<Vec<DirEntry>> {
    let mut dirs: Vec<(DirEntry, SystemTime)> = get_db_dir_entries()?
        .filter_map(|ent| match ent {
            Ok(ent) => Some(ent),
            Err(e) => {
                error!("error while finding directory entry: {:#?}", e);
                None
            }
        })
        .filter_map(|ent| {
            if ent.file_name().to_string_lossy().ends_with("db") {
                Some(ent)
            } else {
                warn!("found non-db file {:#?}", ent);
                None
            }
        })
        .filter_map(|ent| match ent.metadata() {
            Ok(m) => Some((ent, m)),
            Err(e) => {
                error!("failed to read metadata: {:#?}", e);
                None
            }
        })
        .filter_map(|(ent, m)| match m.modified() {
            Ok(t) => Some((ent, t)),
            Err(e) => {
                error!("failed to read modification time: {:#?}", e);
                None
            }
        })
        .collect();

    // most recently modified at front
    dirs.sort_by(|(_, lt), (_, rt)| lt.cmp(rt).reverse());

    let res = dirs.into_iter().map(|(dir, _)| dir).collect();
    info!("get_db_dir_entries_sorted(): got sorted dbs: {:#?}", res);

    Ok(res)
}

/// gets dbs sorted by recent modification time
/// gets dbs in moves any Live-plugins dbs to the front of the sorted db vec
fn get_organized_dbs() -> Result<Vec<DirEntry>> {
    let dbs = get_sorted_dbs()?;

    let mut live_plugins_dbs: Vec<DirEntry> = vec![];
    let mut live_files_dbs: Vec<DirEntry> = vec![];

    for db in dbs {
        // lossy should be ok here
        let db_os_name = db.file_name();
        let db_filename = db_os_name.to_string_lossy();
        if db_filename.starts_with("Live-plugins") {
            live_plugins_dbs.push(db);
        } else if db_filename.starts_with("Live-files") {
            live_files_dbs.push(db);
        } else {
            info!("get_organized_dbs(): found db that doesn't start with Live-plugins or Live-files, skipping: {db_filename}");
        }
    }

    live_plugins_dbs.append(&mut live_files_dbs);

    Ok(live_plugins_dbs)
}

#[cfg(target_os = "macos")]
fn get_ableton_db_dir() -> Result<PathBuf> {
    use directories::UserDirs;

    let user_dirs = UserDirs::new().ok_or(TempoError::Other(
        "Could not open user home directory".into(),
    ))?;

    let home_dir = user_dirs.home_dir();

    Ok(home_dir
            .join("Library")
            .join("Application Support")
            .join("Ableton")
            .join("Live Database"))
}

fn get_db_dir_entries() -> Result<ReadDir> {
    let read = read_dir(get_ableton_db_dir()?)?;
    Ok(read)
}

/// checks an ableton db for valid plugins table
fn check_db_for_plugins_table(db: &Path) -> Result<bool> {
    // TODO dont know if i need pools here
    let con = rusqlite::Connection::open_with_flags(db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    check_db_plugin_schema(&con)
}

fn check_db_plugin_schema(con: &Connection) -> Result<bool> {
    let mut needed_cols: HashSet<String> = HashSet::new();
    needed_cols.insert("dev_identifier".into());
    needed_cols.insert("name".into());
    needed_cols.insert("vendor".into());

    con.pragma(None, "table_info", "plugins", |row| {
        let col_name: String = row.get(1)?;
        needed_cols.remove(&col_name);
        Ok(())
    })?;

    Ok(needed_cols.is_empty())
}

pub struct AbletonPluginRow {
    pub dev_id: String,
    pub name: String,
    pub vendor: String,
}

pub enum ScannedAbletonPlugin {
    Vst(SharedAbletonVstRow),
    Vst3(SharedAbletonVst3Row),
}

// TODO it should probably be ok to load all of the plugin info into memory? not too hard to avoid this if needed
// i have a lot of plugins and it's the plugin info is only around 200 kb
// there is not much plugin info in db, the biggest user of db storage is the files table

/// Gets all plugins in the Ableton plugin database.
/// The most recently modified database will be used.
pub fn scan_plugin_db() -> Result<Vec<ScannedAbletonPlugin>> {
    let ents = get_organized_dbs()?;

    if ents.is_empty() {
        return Err(TempoError::Ableton("Failed to find Ableton plugin database".into()))
    }

    let mut ents = ents.into_iter();
    let con = loop {
        match ents.next() {
            None => {
                return Err(TempoError::Ableton(
                    "Failed to load plugin database, exhaused all db entries".into(),
                ))
            }
            Some(ent) => {
                info!("scan_plugin_db(): scanning {}", ent.file_name().to_string_lossy());
                match rusqlite::Connection::open_with_flags(
                    ent.path(),
                    OpenFlags::SQLITE_OPEN_READ_ONLY,
                ) {
                    Ok(con) => match check_db_plugin_schema(&con) {
                        Ok(true) => break con,
                        Ok(false) => {
                            warn!("scan_plugin_db(): found db but with invalid plugin schema at {}", path_to_str(&ent.path()));
                        }
                        Err(e) => {
                            error!("scan_plugin_db(): error while checking db schema: {e}");
                        }
                    },
                    Err(e) => {
                        error!("scan_plugin_db(): failed to open connection to Ableton db at {}, error: {e}", path_to_str(&ent.path()))
                    }
                }
            }
        }
    };

    let mut stmt = con
        .prepare("SELECT dev_identifier, name, vendor FROM plugins")
        .map_err(|e| TempoError::Ableton(format!("Failed to prepare sql statement: {e}")))?;

    let rows: Vec<rusqlite::Result<AbletonPluginRow>> = stmt
        .query_map([], |row| {
            Ok(AbletonPluginRow {
                dev_id: row.get(0)?,
                name: row.get(1)?,
                vendor: row.get(2)?,
            })
        })?
        .collect();

    Ok(rows
        .into_iter()
        .filter_map(|row| match row {
            Ok(row) => match ScannedAbletonPlugin::try_from(&row) {
                Ok(row) => Some(row),
                Err(e) => {
                    error!("scan_plugin_db(): error while parsing Ableton plugin db row: {e}");
                    None
                }
            },
            Err(e) => {
                error!("scan_plugin_db(): error while reading Ableton plugin db row: {e}");
                None
            }
        })
        .collect())
}

impl TryFrom<&AbletonPluginRow> for ScannedAbletonPlugin {
    type Error = TempoError;

    fn try_from(value: &AbletonPluginRow) -> Result<Self> {
        // plugin id is url encoded, this should be safe for vst plugin ids with : in name
        let parts: Vec<&str> = value.dev_id.split(":").collect();

        if parts.len() != 4 {
            return Err(TempoError::Plugin(format!(
                "Failed to parse dev_id, found {}",
                value.dev_id
            )));
        }

        match parts[1] {
            "vst" => Ok(ScannedAbletonPlugin::Vst(SharedAbletonVstRow {
                id: parse_vst_id(parts[3])?,
                name: value.name.clone(),
                vendor: value.vendor.clone(),
            })),
            "vst3" => Ok(ScannedAbletonPlugin::Vst3(SharedAbletonVst3Row {
                fields: parse_vst3_id(parts[3])?,
                name: value.name.clone(),
                vendor: value.vendor.clone(),
            })),
            t => Err(TempoError::Ableton(format!(
                "Unknown plugin type found in Ableton database: {t}"
            ))),
        }
    }
}

// note that the following functions don't take the whole dev id, they take just the part following the last : in the id

/// Takes a vst u32 id, converts it to bytes
pub fn parse_vst_id(id: &str) -> Result<u32> {
    let split: Vec<&str> = id.split("?").collect();
    if split.len() != 2 {
        return Err(TempoError::ParseId(format!(
            "Error while parsing Ableton VST id, expected 2 parts from splitting on ?, found {}",
            split.len()
        )));
    }

    let id = match split[0].parse::<u32>() {
        Ok(id) => id,
        Err(e) => {
            return Err(TempoError::ParseId(format!(
                "Error while parsing Ableton VST id: {e}"
            )))
        }
    };

    Ok(id)
}

/// Takes a GUID in COM format, converts it into values.
/// For VST3 ids
pub fn parse_vst3_id(guid: &str) -> Result<[i32; 4]> {
    let split: Vec<&str> = guid.split("-").collect();
    if split.len() != 5 {
        return Err(TempoError::ParseId(format!(
            "Error while parsing Ableton VST3 id, expected 5 parts, found {}",
            split.len()
        )));
    }
    if split[0].len() != 8 {
        return Err(TempoError::ParseId(format!(
            "Error while parsing GUID, expected 4 bytes in first element, found {}",
            split[0].len()
        )));
    }

    let joined = split.into_iter().fold(String::new(), |s, v| s + v);

    let b: [u8; 16] = hex::decode(joined)?.try_into().map_err(|e: Vec<u8>| {
        TempoError::ParseId(format!(
            "Error while parsing GUID, failed to build 16 bytes, found {} instead",
            e.len()
        ))
    })?;

    let f0: i32 = i32::from_be_bytes([b[0], b[1], b[2], b[3]]);
    let f1: i32 = i32::from_be_bytes([b[4], b[5], b[6], b[7]]);
    let f2: i32 = i32::from_be_bytes([b[8], b[9], b[10], b[11]]);
    let f3: i32 = i32::from_be_bytes([b[12], b[13], b[14], b[15]]);

    Ok([f0, f1, f2, f3])
}

// TODO add tests here again