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
    /// `None` if the session is unknown.
    fn session(
        &self,
        id: impl AsRef<<Self::Session as Session>::Id>,
    ) -> Result<Option<Self::Session>, Self::Err>;

    /// Creates a new session.
    fn new_session(&self, name: &str) -> Result<Self::Session, Self::Err>;
}

/// A Tempo session.
/// Sessions, at their core, consist of an object store along with a key-value store.
/// This interface is very minimal and provides no specific types for objects/values, all data is stored as byte arrays.
pub trait Session: Sized + Send + Sync + ObjectStore + Map {
    /// Impl-specific identifier for a session.
    type Id: ?Sized;

    type Err: std::error::Error;
}

/// Object store/content-addressable store.
pub trait ObjectStore: Sized + Send + Sync {
    type Err: std::error::Error;
    type Object: Object;

    /// Returns whether the given object exists.
    fn object_exists(&self, hash: &Sha256Hash) -> Result<bool, Self::Err>;

    /// Gets an object given its hash. `None` if the object doesn't exist.
    fn get_object(&self, hash: &Sha256Hash) -> Result<Option<Self::Object>, Self::Err>;

    /// Creates an object.
    fn create_object(&self, data: impl std::io::Read) -> Result<Self::Object, Self::Err>;

    /// Removes an object.
    /// TODO: make sure that using this doesn't break Tempo.
    fn remove_object(&self, hash: &Sha256Hash) -> Result<(), Self::Err>;
}

/// Map/key-value store.
pub trait Map: Sized + Send + Sync {
    type Data: Data;
    type Err: std::error::Error;

    /// Number of key-value entries in this map.
    fn size(&self) -> u64;

    /// Iterates over all key-value pairs in this map.
    fn iter(&self) -> Result<impl Iterator<Item = (Key, Value<impl Map, Self::Data>)>, Self::Err>;

    /// Returns whether the given key is present in this map.
    fn exists(&self, path: &[Key]) -> Result<bool, Self::Err>;

    /// Gets a value. `None` if value not found.
    fn get(&self, path: &[Key]) -> Result<Option<Value<Self, Self::Data>>, Self::Err>;

    /// Sets a key to some `Data`.
    /// This will automatically create nested maps as needed.
    /// Fails if `path` points to an existing key-map pair.
    fn set(&self, path: &[Key], data: impl std::io::Read) -> Result<(), Self::Err>;

    /// Clears a key-value entry.
    /// TODO: make sure that using this doesn't break Tempo.
    fn clear(&self, path: &[Key]) -> Result<(), Self::Err>;
}

/// Arbitrary bytes stored in key-value store.
pub trait Data: Sized + Send + Sync {
    type Err: std::error::Error;

    fn read(&self) -> Result<impl std::io::Read, Self::Err>;
}

/// Object stored in object store.
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
