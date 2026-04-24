mod database;
pub mod api;
pub mod commands;
pub mod progress;
pub mod task_state;

use task_state::TaskState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(TaskState::default())
        .setup(|app| {
            let app_handle = app.handle();
            database::init_database(&app_handle)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::player::add_player,
            commands::player::get_player_list,
            commands::player::get_player,
            commands::player::delete_player,
            commands::player::update_player,
            commands::score::update_player_scores,
            commands::score::get_player_scores,
            commands::score::get_player_score_count,
            commands::score::get_map_scores,
            commands::score::get_playlist_scores,
            commands::map::get_map_by_key,
            commands::map::get_map_by_hash,
            commands::playlist::add_playlist,
            commands::playlist::get_playlist_list,
            commands::playlist::get_playlist,
            commands::playlist::delete_playlist,
            commands::playlist::get_playlist_maps,
            commands::leaderboard::get_leaderboard,
            task_state::get_active_tasks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}