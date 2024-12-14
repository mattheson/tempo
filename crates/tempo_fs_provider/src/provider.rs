pub struct FsProvider<R: tauri::Runtime> {
    pub(crate) db: tempo_db::Db<R>,
    pub(crate) tokio_handle: tokio::runtime::Handle,
}

impl<R: tauri::Runtime> tempo_provider::Provider<R> for FsProvider<R> {
    const NAMESPACE: &str = "fs";

    type Session = crate::FsSession;

    fn new(
        db: tempo_db::Db<R>,
        tokio_handle: tokio::runtime::Handle,
    ) -> Result<Self, tempo_provider::ProviderError> {
        Ok(Self { db, tokio_handle })
    }

    fn session(
        &self,
        id: &<Self::Session as tempo_provider::Session>::Id,
    ) -> Result<Self::Session, tempo_provider::ProviderError> {
        todo!()
    }

    fn new_session(&self, name: &str) -> Result<Self::Session, tempo_provider::ProviderError> {
        todo!()
    }
}
