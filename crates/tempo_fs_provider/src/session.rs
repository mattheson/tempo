mod id;
mod meta;

pub use id::FsSessionId;

pub struct FsSession {}

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