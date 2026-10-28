mod database;
pub mod api;
pub mod commands;
pub mod progress;
pub mod task_state;
mod sync_batch;
mod startup;
mod settings;

use task_state::TaskState;
use tauri::Manager;


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle();
            let settings = settings::SettingsState::load(app.path().app_data_dir()?)
                .map_err(std::io::Error::other)?;
            app.manage(TaskState::with_limits(&settings.concurrency));
            app.manage(settings);
            database::init_database(&app_handle)?;

            startup::spawn(app_handle.clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::player::add_player,
            commands::player::get_player_list,
            commands::player::get_players_with_score_counts,
            commands::player::get_player,
            commands::player::delete_player,
            commands::player::update_player,
            commands::player::open_url,
            commands::player::refresh_player_scores,
            commands::score::update_player_scores,
            commands::score::get_player_scores,
            commands::score::get_player_score_count,
            commands::score::get_map_scores,
            commands::score::get_playlist_scores,
            commands::map::get_map_by_key,
            commands::map::get_map_by_hash,
            commands::map::fetch_and_store_preview_url,
            commands::playlist::add_playlist,
            commands::playlist::refresh_playlist,
            commands::playlist::get_playlist_list,
            commands::playlist::get_playlist,
            commands::playlist::delete_playlist,
            commands::playlist::get_playlist_maps,
            commands::playlist::refresh_playlist_ratings,
            commands::leaderboard::get_leaderboard,
            commands::common::get_version,
            commands::common::open_data_dir,
            commands::common::load_user_prefs,
            commands::common::save_user_prefs,
            settings::get_settings,
            settings::save_settings,
            settings::restart_app,
            task_state::get_active_tasks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
