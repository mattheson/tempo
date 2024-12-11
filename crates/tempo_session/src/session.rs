use crate::{Key, Sha256Hash};

/// A Tempo session.
/// Sessions, at their core, consist of an object store along with a key-value store.
/// This interface is very minimal and provides no specific types of objects/values, all data is stored as byte arrays.
pub trait Session: Sized + Send + Sync + ObjectStore + Map {
    type Err: std::error::Error;

    /// Impl-specific identifier for a session.
    type Id;
    type Info: Info;

    /// Gets info about this session.
    fn info(&self) -> Result<Self::Info, <Self as Session>::Err>;
}

pub trait Info: Sized + Send + Sync {
    type Err: std::error::Error;

    fn get_name(&self) -> String;
}

pub trait ObjectStore: Sized + Send + Sync {
    type Err: std::error::Error;

    type Object: Object;

    /// Returns whether the given object exists.
    fn obj_exists(&self, hash: &Sha256Hash) -> Result<bool, Self::Err>;

    /// Gets an object given its hash. `None` if the object doesn't exist.
    fn get_obj(&self, hash: &Sha256Hash) -> Result<Option<Self::Object>, Self::Err>;

    /// Creates an object.
    fn create_object<R: std::io::Read>(&self, data: R) -> Result<Self::Object, Self::Err>;
}

/// Map/key-value store.
pub trait Map: Sized + Send + Sync {
    type Err: std::error::Error;

    type Data: Data;

    /// Number of key-value entries in this map.
    fn size(&self) -> u64;
    fn iter(&self) -> Result<impl Iterator<Item = (Key, Value<Self, Self::Data>)>, Self::Err>;

    fn exists(&self, key: &Key) -> Result<bool, Self::Err>;

    fn get(&self, key: &Key) -> Result<Option<Value<Self, Self::Data>>, Self::Err>;

    fn set_map(&self, key: &Key) -> Result<Self, Self::Err>;

    fn set_data<R: std::io::Read>(&self, key: &Key, data: R) -> Result<(), Self::Err>;
}

/// Arbitrary bytes stored in key-value store.
pub trait Data: Sized + Send + Sync {
    type Err: std::error::Error;

    fn new<R: std::io::Read>(data: R) -> Result<Self, Self::Err>;
    fn read<R: std::io::Read>(&self) -> Result<R, Self::Err>;
}

/// Object in object store.
pub trait Object: Sized + Send + Sync {
    type Err: std::error::Error;

    fn read<R: std::io::Read>(&self) -> Result<R, Self::Err>;
}

/// A value stored in a `Session`'s key-value map.
/// Only nested maps/data (raw bytes) are supported.
pub enum Value<M: Map, D: Data> {
    Map(M),
    Data(D),
}
