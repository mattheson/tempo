#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error(transparent)]
	Io(#[from] std::io::Error),

	#[error(transparent)]
	Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;