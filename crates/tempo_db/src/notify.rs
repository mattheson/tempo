/// Sends updates to frontend for reactive changes.
pub struct DbNotifier<R: tauri::Runtime>(pub(crate) std::sync::Arc<tauri::AppHandle<R>>);

// runtime generic is required here to allow for using tauri's mock app for testing

impl<R: tauri::Runtime> Clone for DbNotifier<R> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<R: tauri::Runtime> DbNotifier<R> {
    pub fn new(handle: tauri::AppHandle<R>) -> Self {
        Self(std::sync::Arc::new(handle))
    }
}
