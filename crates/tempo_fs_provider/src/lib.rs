/// File system (or file sync) provider.
/// Stores all Tempo data in a file sync-friendly folder hierarchy.
pub struct FsProvider {}

impl tempo_provider::TempoProvider for FsProvider {
    const NAMESPACE: &str = "fs";

    fn new(db: tempo_db::TempoDb, handle: tauri::AppHandle) -> anyhow::Result<Self> {
        todo!()
    }

    fn session<S: tempo_provider::Session>(&self, id: S::Id) -> anyhow::Result<S> {
        todo!()
    }
}
