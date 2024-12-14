pub struct FsSession {
    pub(crate) object_store: crate::object::FsObjectStore,
}

impl tempo_provider::Session for FsSession {}

impl tempo_provider::Map for FsSession {}
