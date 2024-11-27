/// Metadata stored inside of a fs session (folder)

#[derive(serde::Serialize)]
pub(crate) struct FsMeta {
    folder_schema: usize,
    tempo_version: String,
}