#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error(transparent)]
	Io(#[from] std::io::Error),

	#[error(transparent)]
	Other(#[from] anyhow::Error),

	#[error("corrupt/invalid session: {0}")]
	Corrupt(String),

	#[error("object store error: {0}")]
	ObjectStore(String)
}

pub type Result<T> = std::result::Result<T, Error>;