mod error;
pub use error::{Error, Result};

mod object;
pub use object::FsObject;

mod map;
pub use map::FsMap;

#[cfg(test)]
mod test;

use tempo_misc::Sha256Hash;

pub struct FsProvider<R: tauri::Runtime> {
    pub(crate) db: tempo_db::Db<R>,
    pub(crate) tokio_handle: tokio::runtime::Handle,
}

impl<R: tauri::Runtime> tempo_provider::Provider<R> for FsProvider<R> {
    type Err = Error;
    const NAMESPACE: &str = "fs";

    type Session = crate::FsSession;

    fn new(db: tempo_db::Db<R>, tokio_handle: tokio::runtime::Handle) -> Result<Self> {
        Ok(Self { db, tokio_handle })
    }

    fn session(&self, id: impl AsRef<std::path::Path>) -> Result<Option<FsSession>> {
        todo!()
    }

    fn new_session(&self, name: &str) -> Result<FsSession> {
        todo!()
    }
}



pub struct FsSession {
    pub(crate) path: std::path::PathBuf,
    pub(crate) object_store: crate::object::FsObjectStore,
    pub(crate) kv_store: crate::map::FsMap,
}

impl tempo_provider::Session for FsSession {
    type Id = std::path::Path;
    type Err = crate::Error;
}

impl tempo_provider::ObjectStore for crate::FsSession {
    type Err = Error;
    type Object = crate::object::FsObject;

    fn object_exists(&self, hash: &Sha256Hash) -> Result<bool> {
        self.object_store.exists(hash)
    }

    fn get_object(
        &self,
        hash: &Sha256Hash,
    ) -> Result<Option<FsObject>> {
        self.object_store.get(hash)
    }

    fn create_object(&self, data: impl std::io::Read) -> Result<FsObject> {
        self.object_store.create_object(data)
    }

    fn remove_object(&self, _hash: &Sha256Hash) -> Result<()> {
        panic!("remove_object() not implemented for fs provider object store");
    }
}