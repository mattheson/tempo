type Result<T> = std::result::Result<T, String>;

pub struct TempoState {
    pub dbs: tempo_db::Dbs<tauri::Wry>,
}

type State<'a> = tauri::State<'a, TempoState>;

#[tauri::command]
pub async fn db_test(state: State<'_>) -> Result<String> {
    Ok(format!("{:#?}", state.dbs))
}
