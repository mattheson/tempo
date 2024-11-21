/// Handles perisistance, synchronization and modification of session data.
/// Acts as API for performing actions in Tempo sessions (besides reading data).
pub trait TempoProvider: Sized {
    /// Internal identifier for data specific to this provider impl.
    const NAMESPACE: &str;

    type Session: TempoSession;

    /// Initializes this provider.
    fn new(
        db_handle: tempo_db::TempoDb,
        app_handle: tauri::AppHandle,
        tokio_handle: tokio::runtime::Handle,
    ) -> anyhow::Result<Self>;

    fn session(&self, id: &<Self::Session as TempoSession>::Id) -> anyhow::Result<Self::Session>;
}

pub trait TempoSession {
    type Id;

    type Channel: TempoChannel;

    fn set_name(self, name: &str) -> anyhow::Result<()>;
    fn create_channel(self, name: &str) -> anyhow::Result<Self::Channel>;

    fn channel(self, id: &<Self::Channel as TempoChannel>::Id) -> anyhow::Result<Self::Channel>;
}

pub trait TempoChannel {
    type Id;

    type Note: TempoNote;

    fn set_name(self, name: &str) -> anyhow::Result<()>;
    fn create_note(self, note: tempo_types::NewNote) -> anyhow::Result<Self::Note>;

    fn note(self, id: &<Self::Note as TempoNote>::Id) -> anyhow::Result<Self::Note>;
}

pub trait TempoNote {
    type Id;
    type Attachment: TempoAttachment;
    type Comment: TempoComment;

    fn attachment(self) -> anyhow::Result<Option<Self::Attachment>>;
    fn comment(self, id: &<Self::Comment as TempoComment>::Id) -> anyhow::Result<Self::Comment>;
}

pub trait TempoAttachment {}

pub trait TempoComment {
    type Id;
}
