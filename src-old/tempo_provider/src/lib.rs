mod error;
pub use error::*;

use tempo_misc::{Key, Sha256Hash};

/// Providers provide write access to Tempo sessions.
/// Providers are expected to handle asynchronously updating Tempo's SQLite database (see the `tempo_db` crate).
pub trait Provider<'provider, R: tauri::Runtime>: Sized + Send + Sync {
    /// Internal identifier for data specific to this provider impl.
    const NAMESPACE: &'static str;

    type Err: std::error::Error;
    type SessionId;

    /// Initializes this provider.
    fn new(
        db: tempo_db::Db<R>,
        tokio_handle: tokio::runtime::Handle,
    ) -> impl std::future::Future<Output = Result<Self, Self::Err>> + Send;

    /// Creates a new session.
    fn new_session(
        &self,
        name: &str,
    ) -> impl std::future::Future<Output = Result<Self::SessionId, Self::Err>> + Send;

    /// Returns number of objects in this session.
    fn num_objects(
        &self,
        session: impl AsRef<Self::SessionId>
    ) -> impl std::future::Future<Output = Result<usize, Self::Err>> + Send;

    /// Returns whether the given object exists.
    fn object_exists(
        &self,
        session: impl AsRef<Self::SessionId>,
        hash: &Sha256Hash,
    ) -> impl std::future::Future<Output = Result<bool, Self::Err>> + Send;

    /// Gets an object given its hash.
    /// 
    /// `None` if the object doesn't exist.
    fn get_object(
        &self,
        session: impl AsRef<Self::SessionId>,
        hash: &Sha256Hash,
    ) -> impl std::future::Future<Output = Result<Option<impl tokio::io::AsyncRead>, Self::Err>> + Send;

    /// Creates an object.
    /// 
    /// **This fails if an object with an identical hash already exists.**
    fn create_object(
        &self,
        session: impl AsRef<Self::SessionId>,
        data: impl tokio::io::AsyncRead,
    ) -> impl std::future::Future<Output = Result<Sha256Hash, Self::Err>> + Send;

    /// Removes an object.
    /// TODO: make sure that using this doesn't break Tempo.
    fn remove_object(
        &self,
        session: impl AsRef<Self::SessionId>,
        hash: &Sha256Hash,
    ) -> impl std::future::Future<Output = Result<(), Self::Err>> + Send;

    /// Iterates over all keys in this map.
    fn iter(
        &self,
    ) -> impl std::future::Future<Output = Result<impl Iterator<Item = Key>, Self::Err>> + Send;
}

/// Map/key-value store.
pub trait Map: Sized + Send + Sync {
    type Err: std::error::Error;

    /// Number of key-value entries in this map.
    fn size(self) -> impl std::future::Future<Output = Result<usize, Self::Err>> + Send;


    /// Returns whether the given key is present in this map.
    fn exists(
        &self,
        key: &Key,
    ) -> impl std::future::Future<Output = Result<bool, Self::Err>> + Send;

    /// Gets a value. `None` if value not found.
    fn get(
        &self,
        path: &Key,
    ) -> impl std::future::Future<Output = Result<Option<Value<impl Map, impl Data>>, Self::Err>> + Send;

    /// Sets a key to some `Data`.
    /// This will automatically create nested maps as needed.
    /// Fails if `path` points to an existing key-map pair.
    fn set(
        &self,
        path: &Key,
        data: impl std::io::Read,
    ) -> impl std::future::Future<Output = Result<(), Self::Err>> + Send;

    /// Clears a key-value entry.
    /// TODO: make sure that using this doesn't break Tempo.
    fn clear(&self, path: &Key) -> impl std::future::Future<Output = Result<(), Self::Err>> + Send;
}

/// Arbitrary bytes stored in key-value store.
pub trait Data: Sized + Send + Sync {
    type Err: std::error::Error;

    fn read(
        self,
    ) -> impl std::future::Future<Output = Result<impl tokio::io::AsyncRead, Self::Err>> + Send;
}

/// Object stored in object store.
pub trait Object: Sized + Send + Sync {
    type Err: std::error::Error;

    fn hash(&self) -> &Sha256Hash;

    fn read(
        self,
    ) -> impl std::future::Future<Output = Result<impl tokio::io::AsyncRead, Self::Err>> + Send;
}

/// A value stored in a `Session`'s key-value map.
/// Only nested maps/data (raw bytes) are supported.
pub enum Value<M: Map, D: Data> {
    Map(M),
    Data(D),
}
