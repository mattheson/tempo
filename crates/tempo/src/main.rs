// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod tempo;

use std::path::PathBuf;

use tempo_misc::fatal_error;
// use tauri::Manager;

#[cfg(dev)]
fn get_data_dir() -> PathBuf {
    use clap::Parser;

    #[derive(Parser)]
    struct Flags {
        #[clap(
            short = 'd',
            long = "data",
            global = true,
            help = "Path to data directory",
            long_help = "Path to the Tempo data directory",
            name = "DATA"
        )]
        config: Option<PathBuf>,
    }

    let flags = Flags::parse();

    if let Some(p) = flags.config {
        p
    } else {
        eprintln!("data directory must be specified with -d in dev mode\nrun 'npm run tauri dev -- -- -- -d [data dir]' instead");
        std::process::exit(1)
    }
}

#[cfg(not(dev))]
fn get_data_dir() -> PathBuf {
    use directories::UserDirs;

    let err = "Failed to find the Documents directory.\nTempo stores its data in the Documents directory.\nYou might be using a unique setup.\nEnsure Tempo has Full Disk Access if you're on macOS.";

    // we store data dir in documents for now to simplify deleting data dir on future schema changes
    match UserDirs::new() {
        Some(d) => match d.document_dir() {
            Some(d) => d.join("Tempo"),
            None => fatal_error(&format!(
                "{}error: UserDirs::document_dir() returned None",
                err
            )),
        },
        None => fatal_error(&format!("{}error: UserDirs::new() returned None", err)),
    }
}
fn main() {
    // devtools is enabled in release for now
    let devtools = tauri_plugin_devtools::init();

    // sqlite logging
    // idk if this works
    // unsafe {
    //     rusqlite::trace::config_log(Some(|code, msg| info!("sqlite error {code}: {msg}"))).unwrap();
    // }

    // if cfg!(dev) && cfg!(target_os = "macos") && !check_full_disk().unwrap() {
    //     println!("error: the parent process which spawned Tempo does not have Full Disk Access.\nensure you grant Full Disk Access to your text editor/ide/terminal emulator.");
    //     std::process::exit(1);
    // }

    let builder = tauri::Builder::default().plugin(tauri_plugin_sql::Builder::new().build());

    let result = builder
        .plugin(devtools)
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_sql::Builder::new().build())
        .setup(|app| {
            let window =
                tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("/".into()))
                    .visible(true) // window opens after a little bit to prevent white screen
                    .inner_size(1024., 768.)
                    .min_inner_size(1024., 768.)
                    .title("Tempo")
                    .build()?;

            // let data_dir = get_data_dir();

            // match Tempo::new(&data_dir) {
            //     Ok(tempo) => {
            //         app.manage(tempo);

            //         let window = tauri::WebviewWindowBuilder::new(
            //             app,
            //             "main",
            //             tauri::WebviewUrl::App("index.html".into()),
            //         )
            //         .visible(false) // window opens after a little bit to prevent white screen
            //         .inner_size(1024., 768.)
            //         .min_inner_size(1024., 768.)
            //         .title("Tempo")
            //         .build()?;

            //         if cfg!(dev) {
            //             window.open_devtools();
            //         }
            //     },
            //     Err(e) => {
            //         fatal_error(&format!("encountered an error while loading Tempo.\nyou can delete the Tempo folder in your Documents directory to reset Tempo, this will not delete any data in your shared folders.\n\nerror: {e}"));
            //     },
            // }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // get_store_path,
            // scan_folders,
            // check_folder_inside_folder,
            // is_username_free,
            // create_or_add_folder,
            // scan_plugins,
            // create_channel,
            // create_note,
            // add_comment,
            // need_full_disk,
            // open_full_disk,
            // restart,
            // fatal,
            // copy_project,
            // get_file_info,
            // verify_user_has_ableton,
            // scan_folder,
            // scan_folders,
            // get_folder_data,
            // get_attachment_type,
            // remove_folder,
            // scan_project_file_refs,
            // scan_project_plugins,
            // get_last_plugin_scan_time
        ])
        .run(tauri::generate_context!());

    if let Err(e) = result {
        fatal_error(&format!("fatal tauri error!!! error: {e}"))
    }
}
