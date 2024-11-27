pub struct FsNote {}

impl tempo_provider::TempoNote for FsNote {
    type Id = tempo_id::Ulid;
    type Comment = tempo_fs_comment::FsComment;
    type Attachment = tempo_fs_attachment::FsAttachment;

    fn attachment(self) -> anyhow::Result<Option<Self::Attachment>> {
        todo!()
    }

    fn comment(
        self,
        id: &<Self::Comment as tempo_provider::TempoComment>::Id,
    ) -> anyhow::Result<Self::Comment> {
        todo!()
    }
}
