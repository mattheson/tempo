// identifiers

use crate::Error;

#[derive(Clone, Debug, PartialEq)]
pub struct Sha256Hash(String);

impl Sha256Hash {
    pub fn new(hash: &str) -> Result<Self, Error> {
        if hash.len() != 64 {
            return Err(crate::Error::Sha256(format!(
                "invalid length: expected 64, got {}",
                hash.len()
            )));
        }

        if !hash.chars().all(|c| c.is_digit(16)) {
            return Err(crate::Error::Sha256(
                "hash contains non-hexadecimal characters".to_string(),
            ));
        }

        Ok(Self(hash.to_string()))
    }
}

impl std::str::FromStr for Sha256Hash {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl std::fmt::Display for Sha256Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for Sha256Hash {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for Sha256Hash {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Sha256Hash {
    fn into_inner(self) -> String {
        self.0
    }
}

/// String used as a key in a `Session`'s key-value store.
pub struct Key(String);

impl Key {
    /// Creates a new `Key`.
    ///
    /// **Restrictions on key contents:**
    /// - ASCII only
    /// - can only contain a-z A-Z 0-9 _ -
    /// - can only be 255 characters at most
    pub fn new(key: &str) -> Result<Self, Error> {
        // TODO maybe too restricted? will change if needed
        // restrictions should help with allowing for more implementations w/o having to worry about encodings

        if !key.is_ascii() {
            return Err(Error::Key("key contains non-ASCII characters".to_string()));
        }

        if !key
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(Error::Key("key contains disallowed characters".to_string()));
        }

        if key.len() > 255 {
            return Err(Error::Key("key exceeds 255 characters".to_string()));
        }

        Ok(Self(key.to_string()))
    }
}

impl std::str::FromStr for Key {
    type Err = Error;

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

impl std::ops::Deref for Key {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Key {
    fn into_inner(self) -> String {
        self.0
    }
}
