#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unknown db type: {0}")]
    UnknownDbType(String),

    #[error("not a tempo database: {0}")]
    InvalidDb(String),

    #[error("already a tempo database: {0}")]
    AlreadyDb(String),

    #[error("missing root directory for tree db: {0}")]
    MissingTree(std::path::PathBuf),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    ConnectionError(#[from] crate::ConnectionError),

    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
}

// TODO perhaps just use one error enum

impl From<Error> for crate::ConnectionError {
    fn from(value: Error) -> Self {
        Self::Other(Box::new(value))
    }
}