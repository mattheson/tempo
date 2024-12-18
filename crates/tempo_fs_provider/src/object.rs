use crate::{Error, Result};
use tempo_misc::{Sha256Hash, TempFile};

// we use std::fs::exists() over Path::exists() since we want to fail hard if there are permission errors/stuff is missing

/// Object store similar to Git's object store.
pub(crate) struct FsObjectStore(pub(crate) std::path::PathBuf);

impl FsObjectStore {
    /// Creates a new store. The given directory must not exist already.
    pub(crate) fn create(dir: impl AsRef<std::path::Path>) -> Result<Self> {
        if !std::fs::exists(dir.as_ref())? {
            std::fs::create_dir_all(dir.as_ref())?;
            Ok(Self(dir.as_ref().to_path_buf()))
        } else {
            Err(Error::ObjectStore(
                "called FsObjectStore::create() pointing to an existing directory/file".to_string(),
            ))
        }
    }

    /// `dir` should be the root directory of the object store, i.e. the directory holding the 2-char directories.
    pub(crate) fn load(dir: impl AsRef<std::path::Path>) -> Result<Self> {
        if std::fs::exists(dir.as_ref())? {
            if is_valid_object_store(dir.as_ref())? {
                Ok(Self(dir.as_ref().to_path_buf()))
            } else {
                Err(anyhow::anyhow!(
                    "{} is not a valid object store",
                    dir.as_ref().to_string_lossy()
                )
                .into())
            }
        } else {
            Err(crate::Error::Corrupt(format!(
                "could not find object store directory"
            )))
        }
    }

    /// Whether an object exists in this store.
    pub(crate) fn exists(&self, hash: &Sha256Hash) -> Result<bool> {
        Ok(std::fs::exists(self.0.join(&hash[0..2]).join(&hash[2..]))?)
    }

    /// Get an object.
    pub(crate) fn get(&self, hash: &Sha256Hash) -> Result<Option<FsObject>> {
        let path = self.0.join(&hash[0..2]).join(&hash[2..]);

        if std::fs::exists(&path)? {
            Ok(Some(FsObject {
                path,
                hash: hash.clone(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Create an object.
    pub(crate) fn create_object(&self, data: impl std::io::Read) -> Result<FsObject> {
        // read, save to temp file
        // move temp file
        let mut temp_file = TempFile::new()?.persist();
        let hash = tempo_misc::hash_and_copy(data, &mut temp_file)?;

        let obj_dir = self.0.join(&hash[0..2]);
        std::fs::create_dir_all(&obj_dir)?;

        let obj_path = obj_dir.join(&hash[2..]);

        std::fs::rename(temp_file.path(), &obj_path)?;

        Ok(FsObject {
            path: obj_path,
            hash,
        })
    }
}

fn is_valid_object_store(_dir: &std::path::Path) -> Result<bool> {
    // if we need other verification steps
    // TODO
    Ok(true)
}

fn is_empty_directory(dir: &std::path::Path) -> Result<bool> {
    let mut entries = std::fs::read_dir(dir)?;
    Ok(entries.next().is_none())
}

pub struct FsObject {
    pub(crate) path: std::path::PathBuf,
    pub(crate) hash: Sha256Hash,
}

impl tempo_provider::Object for FsObject {
    type Err = Error;

    fn hash(&self) -> Result<Sha256Hash> {
        Ok(self.hash.clone())
    }

    fn read(&self) -> Result<impl std::io::Read> {
        Ok(std::fs::File::open(&self.path)?)
    }
}
