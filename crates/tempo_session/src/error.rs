#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error while parsing sha256: {0}")]
    Sha256(String),

    #[error("invalid key value: {0}")]
    Key(String),
}