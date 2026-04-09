//! BeCode Desktop - Tauri Application Entry Point
//!
//! Beautiful bee-themed AI coding assistant.

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;

fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .invoke_handler(tauri::generate_handler![
            // Chat commands
            commands::chat::send_message,
            commands::chat::cancel_execution,
            commands::chat::compact_context,
            // Settings commands
            commands::settings::get_config,
            commands::settings::save_config,
            commands::settings::set_api_key,
            commands::settings::get_api_key,
            commands::settings::list_providers,
            commands::settings::list_models,
            // File commands
            commands::file::load_file_tree,
            commands::file::read_file,
            commands::file::get_file_preview,
            commands::file::select_project_folder,
            // Session commands
            commands::session::list_sessions,
            commands::session::load_session,
            commands::session::save_session,
            commands::session::delete_session,
            commands::session::export_session,
        ])
        .setup(|_app| {
            tracing::info!("BeCode Desktop started successfully");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running BeCode");
}
