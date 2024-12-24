#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unknown db type: {0}")]
    UnknownDbType(String)
}