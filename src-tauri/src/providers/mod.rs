pub mod fs;

/// Handles perisistance, synchronization and modification of session data.
/// Updates contents of sqlite db.
pub trait TempoProvider: Sized {
    const NAMESPACE: &str;

    /// Initializes this provider.
    fn new(db_handle: crate::db::TempoDb, app_handle: tauri::AppHandle) -> anyhow::Result<Self>;

    /// Gets a session for modification.
    fn session<S: Session>(&self, id: S::Id) -> anyhow::Result<S>;
}

pub trait Session {
    type Id;

    fn set_name(name: &str) -> anyhow::Result<()>;
    fn create_channel(name: &str) -> anyhow::Result<impl Channel>;

    fn channel<C: Channel>(id: C::Id) -> anyhow::Result<C>;
}

pub trait Channel {
    type Id;

    fn set_name(name: &str) -> anyhow::Result<()>;
    fn create_note(note: crate::types::NewNote) -> anyhow::Result<impl Note>;
}

/// Notes are always identified with ulids in the Tempo app.
/// Providers could internally use different identifiers.
pub trait Note {

}

pub trait Comment {}
