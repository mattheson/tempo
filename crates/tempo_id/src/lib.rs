// id types
// we serialize to bytes with serde/autosurgeon
// ids are saved as strings to sqlite

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Ulid(ulid::Ulid);

impl Ulid {
    pub fn new() -> Self {
        Self(ulid::Ulid::new())
    }
}

impl Default for Ulid {
    fn default() -> Self {
        Self::new()
    }
}

impl serde::Serialize for Ulid {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

struct UlidVisitor;

impl<'de> serde::de::Visitor<'de> for UlidVisitor {
    type Value = Ulid;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("ulid in byte format (16 bytes)")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Ulid(ulid::Ulid::from_bytes(v.try_into().map_err(
            |_| E::custom(format!("found {} bytes for ulid, expected 16", v.len())),
        )?)))
    }
}

impl<'de> serde::Deserialize<'de> for Ulid {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_byte_buf(UlidVisitor)
    }
}

impl autosurgeon::Hydrate for Ulid {
    fn hydrate_bytes(_bytes: &[u8]) -> Result<Self, autosurgeon::HydrateError> {
        Ok(Ulid(ulid::Ulid::from_bytes(
            _bytes.try_into().map_err(|_| {
                autosurgeon::HydrateError::unexpected(
                    "16 bytes for ulid",
                    format!("{} bytes", _bytes.len()),
                )
            })?,
        )))
    }
}

impl autosurgeon::Reconcile for Ulid {
    type Key<'a> = autosurgeon::reconcile::NoKey;

    fn reconcile<R: autosurgeon::Reconciler>(&self, mut reconciler: R) -> Result<(), R::Error> {
        reconciler.bytes(self.0.to_bytes())
    }
}

impl std::fmt::Display for Ulid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for Ulid {
    type Err = ulid::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(ulid::Ulid::from_string(s)?))
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Uuid(uuid::Uuid);

impl Uuid {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for Uuid {
    fn default() -> Self {
        Self::new()
    }
}

impl serde::Serialize for Uuid {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

struct UuidVisitor;

impl<'de> serde::de::Visitor<'de> for UuidVisitor {
    type Value = Uuid;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("uuid in byte format")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Uuid(uuid::Uuid::from_bytes(v.try_into().map_err(
            |_| E::custom(format!("found {} bytes for ulid", v.len())),
        )?)))
    }
}

impl<'de> serde::Deserialize<'de> for Uuid {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_byte_buf(UuidVisitor)
    }
}

impl autosurgeon::Hydrate for Uuid {
    fn hydrate_bytes(_bytes: &[u8]) -> Result<Self, autosurgeon::HydrateError> {
        Ok(Uuid(uuid::Uuid::from_bytes(
            _bytes.try_into().map_err(|_| {
                autosurgeon::HydrateError::unexpected(
                    "bytes for uuid",
                    format!("{} bytes", _bytes.len()),
                )
            })?,
        )))
    }
}

impl autosurgeon::Reconcile for Uuid {
    type Key<'a> = autosurgeon::reconcile::NoKey;

    fn reconcile<R: autosurgeon::Reconciler>(&self, mut reconciler: R) -> Result<(), R::Error> {
        reconciler.bytes(self.0.as_bytes())
    }
}

pub struct FileRef {
    sha256_bytes: Vec<u8>,
    string: Option<String>,
}

impl FileRef {

}