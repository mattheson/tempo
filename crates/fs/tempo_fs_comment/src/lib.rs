pub struct FsComment {}

impl tempo_provider::TempoComment for FsComment {
    type Id = tempo_id::Ulid;
}