/// File system (or file sync) provider.
/// Stores all Tempo data in a file sync-friendly folder hierarchy.
pub struct FsProvider {
    db_handle: tempo_db::DbHandle,
    app_handle: tauri::AppHandle,
    tokio_handle: tokio::runtime::Handle
}

impl tempo_provider::TempoProvider for FsProvider {
    const NAMESPACE: &str = "fs";

    type Session = tempo_fs_session::FsSession;

    fn new(
        db_handle: tempo_db::DbHandle,
        app_handle: tauri::AppHandle,
        tokio_handle: tokio::runtime::Handle,
    ) -> anyhow::Result<Self> {
        Ok(Self { db_handle, app_handle, tokio_handle })
    }

    fn session(
        &self,
        id: &<Self::Session as tempo_provider::TempoSession>::Id,
    ) -> anyhow::Result<Self::Session> {
        todo!()
    }
}
