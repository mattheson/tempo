mod stores;

/// File system (or file sync) provider.
/// Stores all Tempo data in a file sync-friendly folder hierarchy.
pub struct FsProvider {}

impl super::TempoProvider for FsProvider {
    const NAMESPACE: &str = "fs";

    fn new(db: crate::db::TempoDb, handle: tauri::AppHandle) -> anyhow::Result<Self> {
       todo!() 
    }
}