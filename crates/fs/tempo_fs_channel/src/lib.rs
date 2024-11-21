pub struct FsChannel {

}

impl tempo_provider::TempoChannel for crate::FsChannel {
    type Id = tempo_id::Ulid;
    type Note = tempo_fs_note::FsNote;

    fn set_name(self, name: &str) -> anyhow::Result<()> {
        todo!()
    }

    fn create_note(self, note: tempo_types::NewNote) -> anyhow::Result<Self::Note> {
        todo!()
    }

    fn note(self, id: &tempo_id::Ulid) -> anyhow::Result<Self::Note> {
        todo!()
    }
}