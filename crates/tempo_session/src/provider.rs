/// Providers provide write access to Tempo sessions.
/// Providers are expected to handle asynchronously updating Tempo's SQLite database (see `tempo_db` crate).
pub trait Provider {
    /// Internal identifier for data specific to this provider impl.
    const NAMESPACE: &str;

    type Session: super::Session;

    /// Initializes this provider.
    fn new(
        db: tempo_db::Db,
        app_handle: tauri::AppHandle,
        tokio_handle: tokio::runtime::Handle,
    ) -> anyhow::Result<Self>;

    /// Retrieves an existing session.
    fn session(&self, id: &<Self::Session as TempoSession>::Id) -> anyhow::Result<Self::Session>;
}
