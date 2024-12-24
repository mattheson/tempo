/// Common errors for providers.
#[derive(thiserror::Error, Debug)]
pub enum ProviderError {
    #[error("unknown session: {0}")]
    UnknownSession(String)
}
