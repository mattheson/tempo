// stuff that maybe isn't needed


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

impl std::fmt::Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string())
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

/// String used as a valid key in a `Session`'s key-value store.
pub struct Key(String);

impl Key {
    /// Creates a new `Key`.
    ///
    /// # Restrictions on key contents
    /// - can't be empty
    /// - ASCII only
    /// - can only contain a-z A-Z 0-9 _ -
    /// - can only be 255 characters at most
    pub fn new(key: &str) -> Result<Self, String> {
        // TODO maybe too restricted? will change if needed
        // restrictions should help with allowing for more implementations w/o having to worry about encodings

        if key.is_empty() {
            return Err("key cannot be empty".to_string());
        }

        if !key.is_ascii() {
            return Err("key contains non-ASCII characters".to_string());
        }

        if !key
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err("key contains disallowed characters".to_string());
        }

        if key.len() > 255 {
            return Err("key exceeds 255 characters".to_string());
        }

        Ok(Self(key.to_string()))
    }
}

impl std::str::FromStr for Key {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for Key {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<std::path::Path> for Key {
    fn as_ref(&self) -> &std::path::Path {
        self.0.as_ref()
    }
}

impl std::ops::Deref for Key {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<std::ffi::OsString> for Key {
    type Error = String;

    fn try_from(value: std::ffi::OsString) -> Result<Self, Self::Error> {
        if let Some(value) = value.to_str() {
            Key::new(value)
        } else {
            Err("OsString appears to be invalid Unicode".to_string())
        }
    }
}

impl Key {
    pub fn into_inner(self) -> String {
        self.0
    }
}