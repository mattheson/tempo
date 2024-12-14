use tempo_misc::{Key, Sha256Hash};

/// Object store similar to Git's object store.
pub(crate) struct FsObjectStore(pub(crate) std::path::PathBuf);

impl FsObjectStore {
    /// `dir` should be the root directory of the object store, i.e. the directory holding the 2-char directories.
    pub(crate) fn new<P>(dir: P) -> crate::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
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
            Err(crate::FsProviderError::MissingObjStore)
        }
    }
}

fn is_valid_object_store(_dir: &std::path::Path) -> Result<bool, crate::FsProviderError> {
    // if we need other verification steps
    Ok(true)
}

impl tempo_provider::ObjectStore for crate::FsSession {
    type Object = FsObject;

    fn obj_exists(&self, hash: &tempo_provider::Sha256Hash) -> Result<bool, Self::Err> {
        Ok(std::fs::exists(
            self.object_store.0.join(&hash[0..2]).join(&hash[2..]),
        )?)
    }

    fn get_obj(
        &self,
        hash: &tempo_provider::Sha256Hash,
    ) -> Result<Option<Self::Object>, Self::Err> {
        let path = self.object_store.0.join(&hash[0..2]).join(&hash[2..]);

        if std::fs::exists(&path)? {
            Ok(Some(FsObject {
                path,
                hash: hash.clone(),
            }))
        } else {
            Ok(None)
        }
    }

    fn create_object(&self, data: impl std::io::Read) -> Result<Self::Object, Self::Err> {}
}

pub struct FsObject {
    pub(crate) path: std::path::PathBuf,
    pub(crate) hash: tempo_provider::Sha256Hash,
}

impl tempo_provider::Object for FsObject {
    fn hash(&self) -> Result<tempo_provider::Sha256Hash, Self::Err> {
        Ok(self.hash.clone())
    }

    fn read(&self) -> Result<impl std::io::Read, Self::Err> {
        Ok(std::fs::File::open(&self.path)?)
    }
}
