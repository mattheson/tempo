use id::FsSessionId;

pub struct FsSession {}

impl FsSession {
    /// Creates a new fs session **within** the given directory.
    /// Error if P is already a session.
    pub fn new<P>(dir: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
    }

    /// Returns whether the given directory is an fs session.
    pub fn is_session(dir: P) -> anyhow::Result<bool> {}
}

impl tempo_provider::TempoSession for crate::FsSession {
    type Id = crate::id::FsSessionId;
    type Channel = tempo_fs_channel::FsChannel;

    fn set_name(self, name: &str) -> anyhow::Result<()> {
        todo!()
    }

    fn create_channel(self, name: &str) -> anyhow::Result<Self::Channel> {
        todo!()
    }

    fn channel(
        self,
        id: &<Self::Channel as tempo_provider::TempoChannel>::Id,
    ) -> anyhow::Result<Self::Channel> {
        todo!()
    }
}

pub(crate) struct FsSessionMeta {
    pub schema: u64,
}

impl FsSessionMeta {
    pub fn save_to(p: &Path) -> anyhow::Result<()> {
        serde_json::to_writer_pretty(
            std::fs::File::options().create_new(true).open(p)?,
            &Self {
                schema: super::FS_SESSION_SCHEMA,
            },
        )
    }
}

fn create_session(p: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(path)
}
