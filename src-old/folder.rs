use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, RwLock, Weak},
};

use log::error;

use crate::{
    channel::Channel,
    daw::plugin::ProjectPluginScan,
    db::{iter_shared_db, SharedDb},
    misc::{path_to_str, Result, TempoError},
    shared::{ChannelData, FolderData, PluginScan, SharedNote, TempoResult},
    structure::{get_client_shared_db_path, iter_channels, iter_notes},
    tempo::{RuntimeFolder, Tempo},
    types::{ChannelDoc, FileInfo},
};

pub struct Folder {
    tempo: Tempo,
    // emitter: StateEmitter,
    inner: FolderInner,
}

#[derive(Debug, Clone)]
pub struct FolderInner {
    folder: Weak<RwLock<RuntimeFolder>>,
}

impl FolderInner {
    pub fn path(&self) -> Result<PathBuf> {
        self.expect_valid()?;
        let f = self.upgrade()?;
        let folder = f.read().unwrap();
        Ok(folder.path.clone())
    }

    pub fn username(&self) -> Result<String> {
        self.expect_valid()?;
        let f = self.upgrade()?;
        let folder = f.read().unwrap();
        Ok(folder.username.clone())
    }

    pub fn get_data(&self, db: &SharedDb) -> Result<FolderData> {
        self.expect_valid()?;
        let f = self.upgrade()?;
        let folder = f.read().unwrap();
        FolderData::new(&folder.path, &folder.username, db)
    }

    fn upgrade(&self) -> Result<Arc<RwLock<RuntimeFolder>>> {
        match self.folder.upgrade() {
            Some(a) => Ok(a),
            None => Err(TempoError::Folder(
                "The folder was removed before performing the requested action".into(),
            )),
        }
    }

    pub fn expect_valid(&self) -> Result<()> {
        let f = self.upgrade()?;
        let folder = f.read().unwrap();
        if let Some(e) = folder.error.as_ref() {
            Err(TempoError::Folder(format!(
                "The requested action on folder {} cannot be performed, folder is invalid: {e}",
                path_to_str(&folder.path)
            )))
        } else {
            Ok(())
        }
    }
}

impl Folder {
    pub fn new(folder: Weak<RwLock<RuntimeFolder>>, tempo: Tempo) -> Result<Self> {
        Ok(Self {
            // emitter,
            tempo,
            inner: FolderInner { folder },
        })
    }

    pub fn channel(&self, channel_ulid: Option<&str>) -> Result<Channel> {
        self.inner.expect_valid()?;

        Channel::load(self.tempo.clone(), self.inner.clone(), channel_ulid)
    }

    pub fn create_channel(&self, channel_name: &str) -> Result<Channel> {
        self.inner.expect_valid()?;
        Channel::create(self.tempo.clone(), self.inner.clone(), channel_name)
    }

    // pub fn username(&self) -> Result<String> {
    //     self.inner.username()
    // }

    // pub fn path(&self) -> Result<PathBuf> {
    //     self.inner.path()
    // }

    // /// Returns a `Vec` of usernames of other users found in this folder.
    // pub fn get_other_users(&self) -> Result<Vec<String>> {
    //     self.inner.ensure_valid()?;

    //     todo!()
    // }

    pub fn get_data(&self) -> Result<FolderData> {
        match self.tempo.get_data_dir_db()? {
            Some(db) => self.inner.get_data(&db),
            None => Err(TempoError::Folder(
                "Please scan your plugins before opening a folder".into(),
            )),
        }
    }

    pub fn scan_project_plugins(&self, project: &Path) -> Result<PluginScan> {
        let folder = self.inner.path()?;

        match ProjectPluginScan::new(project) {
            Err(e) => Err(e),
            Ok(s) => {
                // TODO create trait for project plugin scanners
                // and maybe for filerefs as well
                match s {
                    ProjectPluginScan::Ableton(mut s) => {
                        for (username, db) in iter_shared_db(&folder)? {
                            if let Err(e) = s.scan_db(&db, &username) {
                                error!("scan_project_plugins(): failed to scan db for {username} in {}: {e}", path_to_str(&folder));
                            }
                        }
                        Ok(s.done())
                    }
                }
            }
        }
    }

    pub fn file_info(&self, file_sha256: &str) -> Result<FileInfo> {
        FileInfo::load(&self.inner.path()?, file_sha256)
    }

    /// Copies a `shared.sqlite` db into this folder if it's valid
    pub fn copy_db(&self, db: &Path) -> Result<()> {
        self.inner.expect_valid()?;

        fs::copy(
            db,
            get_client_shared_db_path(&self.inner.path()?, &self.inner.username()?),
        )?;

        Ok(())
    }
}

impl FolderData {
    /// Creates a new `FolderData`.
    /// Loads the entire state of the supplied folder.
    pub fn new(folder: &Path, username: &str, db: &SharedDb) -> Result<Self> {
        // TODO this obviously will not work well for large folders

        let mut global: HashMap<String, TempoResult<SharedNote>> = HashMap::new();

        for (_, note_ulid) in iter_notes(folder, None)? {
            let note = SharedNote::new(folder, username, None, &note_ulid, db);
            global.insert(note_ulid, note);
        }

        let mut channels = HashMap::new();

        for (_, channel_ulid) in iter_channels(folder)? {
            let meta = match ChannelDoc::load(folder, username, &channel_ulid) {
                Ok(d) => TempoResult::Ok(d),
                Err(e) => TempoResult::Err(format!("Failed to load channel doc: {e}")),
            };

            let mut notes: HashMap<String, TempoResult<SharedNote>> = HashMap::new();

            for (_, note_ulid) in iter_notes(folder, Some(&channel_ulid))? {
                let note = SharedNote::new(folder, username, Some(&channel_ulid), &note_ulid, db);
                notes.insert(note_ulid, note);
            }

            channels.insert(channel_ulid, ChannelData { meta, notes });
        }

        Ok(FolderData {
            username: username.into(),
            global,
            channels,
        })
    }
}
