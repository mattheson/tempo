pub fn get_temp_dir(prefix: &str) -> anyhow::Result<tempo_misc::TempDir> {
    let tests_dir = std::path::PathBuf::from(env!("CARGO_WORKSPACE_DIR")).join("tests");

    if !tests_dir.exists() {
        std::fs::create_dir_all(&tests_dir)?;
    }

    let mut temp_dir = tempo_misc::TempDir::new(&tests_dir, prefix)?;
    temp_dir.persist();
    temp_dir.save_on_panic();

    Ok(temp_dir)
}
