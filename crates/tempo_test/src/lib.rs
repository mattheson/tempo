pub fn get_temp_dir(prefix: &str) -> anyhow::Result<tempdir::TempDir> {
    let tests_dir = std::path::PathBuf::from(env!("CARGO_WORKSPACE_DIR")).join("tests");

    if !tests_dir.exists() {
        std::fs::create_dir_all(&tests_dir)?;
    }

    let dir = tempdir::TempDir::new_in(&tests_dir, prefix)?;

    log::info!("temporary directory: {}", dir.path().to_string_lossy());

    Ok(dir)
}
