mod error;
pub use error::*;

use tempo_misc::{Key, Sha256Hash};

/// Providers provide write access to Tempo sessions.
/// Providers are expected to handle asynchronously updating Tempo's SQLite database (see the `tempo_db` crate).
pub trait Provider<R: tauri::Runtime>: Sized {
    /// Internal identifier for data specific to this provider impl.
    const NAMESPACE: &str;

    type Err: std::error::Error;
    type Session: Session;

    /// Initializes this provider.
    fn new(db: tempo_db::Db<R>, tokio_handle: tokio::runtime::Handle) -> Result<Self, Self::Err>;

    /// Retrieves an existing session.
    fn session(&self, id: &<Self::Session as Session>::Id) -> Result<Self::Session, Self::Err>;

    /// Creates a new session.
    fn new_session(&self, name: &str) -> Result<Self::Session, Self::Err>;
}

/// A Tempo session.
/// Sessions, at their core, consist of an object store along with a key-value store.
/// This interface is very minimal and provides no specific types for objects/values, all data is stored as byte arrays.
pub trait Session: Sized + Send + Sync + ObjectStore + Map {
    /// Impl-specific identifier for a session.
    type Id;

    type Err: std::error::Error;
}

/// Object store/content-addressable store.
pub trait ObjectStore: Sized + Send + Sync {
    type Err: std::error::Error;
    type Object: Object;

    /// Returns whether the given object exists.
    fn obj_exists(&self, hash: &Sha256Hash) -> Result<bool, Self::Err>;

    /// Gets an object given its hash. `None` if the object doesn't exist.
    fn get_obj(&self, hash: &Sha256Hash) -> Result<Option<Self::Object>, Self::Err>;

    /// Creates an object.
    fn create_object(&self, data: impl std::io::Read) -> Result<Self::Object, Self::Err>;
}

/// Map/key-value store.
pub trait Map: Sized + Send + Sync {
    type Data: Data;
    type Err: std::error::Error;

    /// Number of key-value entries in this map.
    fn size(&self) -> u64;
    fn iter(&self) -> Result<impl Iterator<Item = (Key, Value<Self, Self::Data>)>, Self::Err>;

    fn exists(&self, key: &Key) -> Result<bool, Self::Err>;

    fn get(&self, key: &Key) -> Result<Option<Value<Self, Self::Data>>, Self::Err>;

    fn set_map(&self, key: &Key) -> Result<Self, Self::Err>;

    fn set_data(&self, key: &Key, data: impl std::io::Read) -> Result<(), Self::Err>;
}

/// Arbitrary bytes stored in key-value store.
pub trait Data: Sized + Send + Sync {
    type Err: std::error::Error;

    fn new(data: impl std::io::Read) -> Result<Self, Self::Err>;
    fn read(&self) -> Result<impl std::io::Read, Self::Err>;
}

/// Object in object store.
pub trait Object: Sized + Send + Sync {
    type Err: std::error::Error;

    fn hash(&self) -> Result<Sha256Hash, Self::Err>;
    fn read(&self) -> Result<impl std::io::Read, Self::Err>;
}

/// A value stored in a `Session`'s key-value map.
/// Only nested maps/data (raw bytes) are supported.
pub enum Value<M: Map, D: Data> {
    Map(M),
    Data(D),
}
