use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use crate::{
    db::SharedDb,
    folder::Folder,
    misc::{new_ulid, path_to_str, Result, TempoError},
    shared::FolderInfo,
    structure::{
        create_tempo_folder, expect_valid_folder, get_client_shared_db_path,
        validate_folder_structure,
    },
};
use log::{error, info};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Tempo {
    inner: Arc<RwLock<TempoInner>>,
}

#[derive(Debug)]
struct TempoInner {
    pub data_dir: PathBuf,
    pub data_file: PathBuf,

    pub client_ulid: String,
    pub folders: HashMap<PathBuf, Arc<RwLock<RuntimeFolder>>>,
}

impl TempoInner {
    fn scan_folders(&mut self) -> Vec<FolderInfo> {
        let mut folders: Vec<FolderInfo> = vec![];

        for folder in self.folders.values() {
            let mut folder = folder.write().unwrap();
            let path = folder.path.clone();
            match folder.scan(&self.client_ulid) {
                Err(e) => error!(
                    "Error while scanning folder {}: {e}",
                    path_to_str(&folder.path)
                ),
                Ok(res) => match res {
                    None => folders.push(FolderInfo { path, error: None }),
                    Some(error) => folders.push(FolderInfo {
                        path,
                        error: Some(error),
                    }),
                },
            }
        }

        folders
    }

    fn scan_folder(&mut self, folder: &Path) -> Result<FolderInfo> {
        if let Some(f) = self.folders.get(folder) {
            let mut folder = f.write().unwrap();
            let path = folder.path.clone();
            if let Some(error) = folder.scan(&self.client_ulid)? {
                Ok(FolderInfo {
                    path,
                    error: Some(error),
                })
            } else {
                Ok(FolderInfo { path, error: None })
            }
        } else {
            Err(TempoError::Folder(format!(
                "Tried to scan unknown folder {}",
                path_to_str(folder)
            )))
        }
    }

    fn save(&self) -> Result<()> {
        Ok(serde_json::to_writer_pretty(
            fs::File::options()
                .write(true)
                .truncate(true)
                .create(true)
                .open(&self.data_file)?,
            &DataFile {
                ulid: self.client_ulid.clone(),
                folders: self
                    .folders
                    .iter()
                    .map(|(p, f)| {
                        let folder = f.read().unwrap();
                        (p.to_path_buf(), folder.username.clone())
                    })
                    .collect(),
            },
        )?)
    }

    fn is_username_free(&self, folder: &Path, username: &str) -> Result<bool> {
        match SharedDb::open(&get_client_shared_db_path(folder, username))? {
            Some(db) => Ok(db.get_ulid()? != self.client_ulid),
            None => Ok(true),
        }
    }

    fn get_data_dir_db(&self) -> Result<Option<SharedDb>> {
        SharedDb::open(&self.data_dir.join("shared.sqlite"))
    }
}

impl Tempo {
    pub fn new(data_dir: &Path) -> Result<Self> {
        let data_file = data_dir.join("folders.json");

        let data = if !data_file.exists() {
            fs::create_dir_all(data_dir)?;
            let data = DataFile {
                ulid: new_ulid(),
                folders: HashMap::new(),
            };
            serde_json::to_writer_pretty(fs::File::create(&data_file)?, &data)?;
            data
        } else {
            serde_json::from_reader(fs::File::open(&data_file)?)?
        };

        let mut folders: HashMap<PathBuf, Arc<RwLock<RuntimeFolder>>> = HashMap::new();

        // intial scan
        for (path, username) in data.folders {
            let error = scan_folder_structure(&path);
            folders.insert(
                path.clone(),
                Arc::new(RwLock::new(RuntimeFolder {
                    path,
                    error,
                    username,
                })),
            );
        }

        let inner = Arc::new(RwLock::new(TempoInner {
            client_ulid: data.ulid,
            data_dir: data_dir.to_path_buf(),
            data_file,
            folders,
            // watcher,
            // watched_folder,
        }));

        Ok(Self { inner })
    }

    pub fn add_folder(&self, folder: &Path, username: &str) -> Result<()> {
        expect_valid_folder(folder)?;
        self.expect_unknown_folder(folder)?;

        let mut tempo = self.inner.write().unwrap();

        let f = RuntimeFolder {
            path: folder.to_path_buf(),
            username: username.to_string(),
            error: None,
        };

        let db =
            match SharedDb::open(&tempo.data_dir.join("shared.sqlite"))? {
                Some(db) => db,
                None => return Err(TempoError::Db(
                    "Missing plugin scan database, please scan your plugins before adding a folder"
                        .into(),
                )),
            };

        info!("here");

        db.copy_into_folder(folder, username)?;

        tempo
            .folders
            .insert(folder.to_path_buf(), Arc::new(RwLock::new(f)));

        tempo.save()?;

        Ok(())
    }

    pub fn create_folder(&self, folder: &Path) -> Result<()> {
        self.expect_unknown_folder(folder)?;
        create_tempo_folder(folder)
    }

    pub fn remove_folder(&self, folder: &Path) -> Result<()> {
        self.expect_known_folder(folder)?;

        let mut tempo = self.inner.write().unwrap();

        if tempo.folders.remove(folder).is_some() {
            tempo.save()?;
            Ok(())
        } else {
            Err(TempoError::Folder(format!(
                "Tried to remove unknown folder {}",
                path_to_str(folder)
            )))
        }
    }

    pub fn folder(&self, folder: &Path) -> Result<Folder> {
        let tempo = self.inner.read().unwrap();

        if let Some(f) = tempo.folders.get(folder) {
            Ok(Folder::new(Arc::downgrade(f), self.clone())?)
        } else {
            Err(TempoError::Folder(format!(
                "Unknown folder {}",
                path_to_str(folder)
            )))
        }
    }

    pub fn save(&self) -> Result<()> {
        self.inner.read().unwrap().save()
    }

    pub fn expect_known_folder(&self, folder: &Path) -> Result<()> {
        let tempo = self.inner.read().unwrap();

        if tempo.folders.contains_key(folder) {
            Ok(())
        } else {
            Err(TempoError::Folder(format!(
                "Unknown shared folder {}, expected a known folder",
                path_to_str(folder)
            )))
        }
    }

    pub fn expect_unknown_folder(&self, folder: &Path) -> Result<()> {
        let tempo = self.inner.read().unwrap();

        if !tempo.folders.contains_key(folder) {
            Ok(())
        } else {
            Err(TempoError::Folder(format!(
                "Already know of folder {}, expected an unknown folder",
                path_to_str(folder)
            )))
        }
    }

    pub fn get_store_path(&self) -> PathBuf {
        self.inner.read().unwrap().data_dir.join("tempo.json")
    }

    pub fn scan_folder(&self, folder: &Path) -> Result<FolderInfo> {
        let mut tempo = self.inner.write().unwrap();
        tempo.scan_folder(folder)
    }

    pub fn scan_folders(&self) -> Vec<FolderInfo> {
        self.inner.write().unwrap().scan_folders()
    }

    pub fn scan_plugins(&self) -> Result<()> {
        let tempo = self.inner.read().unwrap();

        info!("scanning plugins to {}", path_to_str(&tempo.data_dir));
        match crate::db::scan_plugins(&tempo.data_dir.join("shared.sqlite"), &tempo.client_ulid) {
            Ok(()) => Ok(()),
            Err(e) => {
                error!("failed to scan plugins: {e}");
                Err(e)
            }
        }
    }

    /// Copies `shared.sqlite` into all valid folders.
    pub fn copy_db(&self) -> Result<()> {
        let db_path = self.inner.read().unwrap().data_dir.join("shared.sqlite");

        if !db_path.exists() {
            return Err(TempoError::Db(
                "Tried to copy shared database into folders without existing shared database"
                    .into(),
            ));
        }

        let mut failures: HashMap<String, String> = HashMap::new();

        for f in self.scan_folders() {
            if f.error.is_some() {
                continue;
            }
            match self.folder(&f.path) {
                Ok(folder) => {
                    if let Err(e) = folder.copy_db(&db_path) {
                        failures.insert(path_to_str(&f.path), e.to_string());
                    }
                }
                Err(e) => {
                    failures.insert(path_to_str(&f.path), e.to_string());
                }
            }
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(TempoError::Db(format!(
                "Failed to copy database into some folders:\n{}",
                failures
                    .into_iter()
                    .fold(String::new(), |s, (p, e)| { s + &format!("{p}: {e}\n") })
            )))
        }
    }

    pub fn get_last_plugin_scan_time(&self) -> Result<Option<i64>> {
        let tempo = self.inner.read().unwrap();

        let db = SharedDb::open(&tempo.data_dir.join("shared.sqlite"))?;

        Ok(match db {
            Some(db) => Some(db.get_last_scan_time()?.unix_timestamp()),
            None => None,
        })
    }

    pub fn is_username_free(&self, folder: &Path, username: &str) -> Result<bool> {
        let tempo = self.inner.read().unwrap();
        tempo.is_username_free(folder, username)
    }

    pub fn get_data_dir_db(&self) -> Result<Option<SharedDb>> {
        self.inner.read().unwrap().get_data_dir_db()
    }
}

impl Drop for Tempo {
    fn drop(&mut self) {
        match self.save() {
            Ok(()) => (),
            Err(e) => error!("error while saving Tempo in drop(): {e}"),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct DataFile {
    pub ulid: String,
    // { folder : username to use }
    pub folders: HashMap<PathBuf, String>,
}

#[derive(Debug)]
pub struct RuntimeFolder {
    pub path: PathBuf,
    pub username: String,
    pub error: Option<String>,
}

impl RuntimeFolder {
    /// Scans the validity of this folder.
    /// Returns the validity error of this folder if any.
    fn scan_structure(&mut self) -> Option<&str> {
        let error = scan_folder_structure(&self.path);
        if error != self.error {
            self.error = error;
        }
        self.error.as_deref()
    }

    fn scan_username(&mut self, client_ulid: &str) -> Result<Option<&str>> {
        let error = scan_folder_username(&self.path, client_ulid, &self.username)?;
        if error != self.error {
            self.error = error;
        }
        Ok(self.error.as_deref())
    }

    pub fn scan(&mut self, client_ulid: &str) -> Result<Option<String>> {
        let structure_error = self.scan_structure();
        if let Some(e) = structure_error {
            return Ok(Some(e.to_string()));
        }

        let username_error = self.scan_username(client_ulid)?;
        if let Some(e) = username_error {
            return Ok(Some(e.to_string()));
        }

        Ok(None)
    }
}

pub fn scan_folder_structure(folder: &Path) -> Option<String> {
    match validate_folder_structure(folder) {
        Ok(()) => None,
        Err(e) => Some(e),
    }
}

pub fn scan_folder_username(
    folder: &Path,
    client_ulid: &str,
    username: &str,
) -> Result<Option<String>> {
    match SharedDb::open(&get_client_shared_db_path(folder, username))? {
        None => Ok(Some("Your shared database is missing from this folder. Try removing and adding this folder again.".into())),
        Some(db) => {
            if db.get_ulid()? != client_ulid {
                Ok(Some(format!(
                    "Another user is using the username \"{username}\""
                )))
            } else {
                Ok(None)
            }
        }
    }
}

// async fn periodic_folder_scan(tempo: Arc<RwLock<TempoInner>>) {
//     loop {
//         tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
//         {
//             let mut data = tempo.write().unwrap();
//             data.scan_folders();
//         }
//     }
// }
