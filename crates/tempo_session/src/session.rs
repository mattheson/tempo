use crate::{Key, Sha256Hash};

/// A Tempo session.
pub trait Session: Sized + Send + Sync + ObjStore + Map {
    type Err: std::error::Error;

    /// Impl-specific identifier for a session.
    type Id;
    type Info: Info;

    /// Creates a new session.
    fn new(info: &Self::Info) -> Result<Self, <Self as Session>::Err>;

    /// Loads an existing session.
    fn load(id: &Self::Id) -> Result<Self, <Self as Session>::Err>;

    /// Gets info about this session.
    fn info(&self) -> Result<Self::Info, <Self as Session>::Err>;
}

pub trait Info: Sized + Send + Sync {
    type Err: std::error::Error;
}

pub trait ObjStore: Sized + Send + Sync {
    type Err: std::error::Error;

    type Object: Object;

    /// Returns whether the given object exists.
    fn obj_exists(&self, hash: &Sha256Hash) -> Result<bool, Self::Err>;

    /// Gets an object given its hash. `None` if the object doesn't exist.
    fn get_obj(&self, hash: &Sha256Hash) -> Result<Option<Self::Object>, Self::Err>;

    /// Creates an object given an optional name, type tag, and data.
    /// **Type tag must not be empty.**
    fn create_object<R: std::io::Read>(
        &self,
        name: Option<&str>,
        type_tag: &str,
        data: R,
    ) -> Result<Self::Object, Self::Err>;
}

/// Map/key-value store.
pub trait Map: Sized + Send + Sync {
    type Err: std::error::Error;

    type Data: Data;
    type Object: Object;

    /// Number of key-value entries in this map.
    fn len(&self) -> u64;
    fn iter(&self) -> impl Iterator<Item = (Key, Value<Self, Self::Data, Self::Object>)>;

    fn exists(&self, key: &Key) -> Result<bool, Self::Err>;

    fn get(&self, key: &Key) -> Result<Option<Value<Self, Self::Data, Self::Object>>, Self::Err>;

    fn get_resolved(
        &self,
        key: &Key,
    ) -> Result<Option<ResolvedValue<Self, Self::Data, Self::Object>>, Self::Err>;

    fn set(&self, key: &Key, val: &Value<Self, Self::Data, Self::Object>) -> Result<(), Self::Err>;
}

/// Arbitrary bytes.
pub trait Data: Sized + Send + Sync {
    type Err: std::error::Error;

    fn new<R: std::io::Read>(data: R) -> Result<Self, Self::Err>;
    fn read<R: std::io::Read>(&self) -> Result<R, Self::Err>;
}

/// Object in object store.
pub trait Object: Sized + Send + Sync {
    type Err: std::error::Error;

    fn get_name(&self) -> Option<&str>;
    fn get_type(&self) -> &str;

    fn read<R: std::io::Read>(&self) -> Result<R, Self::Err>;
}

/// A value stored in a `Session`'s key-value map.
pub enum Value<M: Map, D: Data, O: Object> {
    ObjRef(Option<O>),
    ValRef(Option<Box<Value<M, D, O>>>),
    Map(M),
    Data(D),
}

/// A value stored in a `Session`'s key-value map, but with any reference types resolved.
pub enum ResolvedValue<M: Map, D: Data, O: Object> {
    Map(M),
    Object(Option<O>),
    Data(D),
}
