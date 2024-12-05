mod fs;

#[derive(Clone, Debug, PartialEq)]
pub struct Sha256Hash(String);

impl std::str::FromStr for Sha256Hash {
    type Err = Sha256HashError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 64 {
            return Err(Sha256HashError::ParseError(format!(
                "invalid length: expected 64, got {}",
                s.len()
            )));
        }

        if !s.chars().all(|c| c.is_digit(16)) {
            return Err(Sha256HashError::ParseError(
                "hash contains non-hexadecimal characters".to_string(),
            ));
        }
        Ok(Sha256Hash(s.to_string()))
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

#[derive(thiserror::Error, Debug)]
pub enum Sha256HashError {
    #[error("failed to parse hash: {0}")]
    ParseError(String),
}

pub trait Session {
    type Err: std::error::Error;

    type Id;

    type Info: Info;
    type Object: Object;
    type Ref: Ref;
    type Data: Data;

    /// Creates a new session.
    fn new(info: &Self::Info) -> Result<Self, Self::Err> where Self: Sized;

    /// Loads an existing session.
    fn load(id: &Self::Id) -> Result<Self, Self::Err> where Self: Sized;

    // objects --------------------------------------------------------

    /// Returns whether the given object exists.
    fn object_exists(hash: &Sha256Hash) -> Result<bool, Self::Err>;

    /// Gets an object given its hash. `None` if the object does not exist.
    fn get_object(hash: &Sha256Hash) -> Result<Option<Self::Object>, Self::Err>;

    /// Creates an object given an optional name, type tag, and data.
    fn create_object<R: std::io::Read>(name: Option<&str>, type_tag: &str, data: R) -> Result<Self::Object, Self::Err>;

    // refs --------------------------------------------------------

    // path is supplied as &Vec<String> for flexibility
    // `refs` is implied at beginning of path
    // for example:
    //    refs/hi/there
    // would be
    //    &["hi", "there"]

    /// Returns whether the given ref exists.
    fn ref_exists<S: AsRef<str>>(path: &[S]);
    fn get_ref<S: AsRef<str>>(path: &[S]) -> Result<Option<Self::Ref>, Self::Err>;


    // data
}

pub trait Info {

}

pub trait Object {
    fn get_name(&self) -> &str;
    fn get_type(&self) -> &str;
    fn get_data();
}

#[derive(Clone, Debug, PartialEq)]
pub enum RefTarget<R> {
    Object(Sha256Hash),
    Ref(R)
}

pub trait Ref {
}

pub trait Data {}
