mod error;
pub use error::{Error, Result};

mod object;
pub use object::FsObject;

mod map;
pub use map::{FsMap, FsData};

#[cfg(test)]
mod test;

use tempo_misc::{Sha256Hash, Key};
use tempo_provider::Value;

pub struct FsProvider<R: tauri::Runtime> {
    pub(crate) db: tempo_db::Db<R>,
    pub(crate) tokio_handle: tokio::runtime::Handle,
}

// impl<'provider, R: tauri::Runtime> tempo_provider::Provider<'provider, R> for FsProvider<R> {
//     type Err = Error;
//     const NAMESPACE: &'static str = "fs";

//     type Session = crate::FsSession<'provider>;

//     fn new(db: tempo_db::Db<R>, tokio_handle: tokio::runtime::Handle) -> Result<Self> {
//         Ok(Self { db, tokio_handle })
//     }

// }

pub struct FsSession<'fssession> {
    pub(crate) path: std::path::PathBuf,
    pub(crate) object_store: crate::object::FsObjectStore,
    pub(crate) kv_store: crate::map::FsMap<'fssession>,
}

impl<'session> tempo_provider::Session<'session> for FsSession<'session> {
    type Id = std::path::Path;
    type Err = crate::Error;
}

impl<'objstore> tempo_provider::ObjectStore<'objstore> for crate::FsSession<'objstore> {
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

impl<'map> tempo_provider::Map<'map> for crate::FsSession<'map> {
    type Err = Error;

    fn exists(&self, key: &Key) -> Result<bool> {
        todo!()
    }

    fn get(&'map self, path: &Key) -> Result<Option<Value<FsMap<'map>, FsData<'map>>>> {
        todo!()
    }

    fn set(&self, key: &Key, data: impl std::io::Read) -> Result<()> {
        todo!()
    }

    fn clear(&self, _key: &Key) -> Result<()> {
        panic!("clear not implemented for FsSession map")
    }
}