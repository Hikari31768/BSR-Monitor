mod database;
pub mod api;
pub mod commands;
pub mod progress;
pub mod task_state;

use task_state::TaskState;
use tauri::{Manager, Emitter};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(TaskState::default())
        .setup(|app| {
            let app_handle = app.handle();
            database::init_database(&app_handle)?;

            // 启动时串行刷新已添加玩家的成绩（单个整合任务，不逐条弹）
            let app_handle_clone = app_handle.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
                rt.block_on(async move {
                    // 等待短暂时间确保 UI 就绪
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                    // 获取所有玩家
                    let player_ids = {
                        let state = match app_handle_clone.try_state::<std::sync::Mutex<rusqlite::Connection>>() {
                            Some(s) => s,
                            None => return,
                        };
                        let guard = match state.lock() {
                            Ok(g) => g,
                            Err(_) => return,
                        };
                        match commands::player::get_all_player_ids(&guard) {
                            Ok(ids) => ids,
                            Err(_) => return,
                        }
                    };

                    if player_ids.is_empty() {
                        return;
                    }

                    let total = player_ids.len();
                    let task_id = format!("startup-{}", uuid::Uuid::new_v4());
                    let task_state = match app_handle_clone.try_state::<TaskState>() {
                        Some(s) => s,
                        None => return,
                    };
                    task_state.register_with_name(&task_id, "startup_refresh", "0", "启动刷新玩家成绩");
                    task_state.update(&task_id, 0.0, &format!("正在刷新玩家成绩 (0/{})", total));

                    // 发射进度事件给前端
                    let payload = progress::ProgressPayload::new(progress::TaskType::StartupRefresh, task_id.clone())
                        .with_progress("refreshing".to_string(), 0.0)
                        .with_message(format!("正在刷新玩家成绩 (0/{})", total));
                    let _ = app_handle_clone.emit("progress-update", &payload);

                    for (idx, player_id) in player_ids.iter().enumerate() {
                        let current = idx + 1;
                        let progress = (current as f64 / total as f64) * 100.0;

                        // 先获取玩家名字用于显示
                        let player_name: String = {
                            let state = app_handle_clone.try_state::<std::sync::Mutex<rusqlite::Connection>>();
                            match state {
                                Some(s) => match s.lock() {
                                    Ok(guard) => match commands::player::get_player_name(&guard, player_id) {
                                        Ok(Some(name)) => name,
                                        _ => "未知玩家".to_string(),
                                    },
                                    Err(_) => "未知玩家".to_string(),
                                },
                                None => "未知玩家".to_string(),
                            }
                        };

                        let msg = format!("正在刷新 {}/{}: {}", current, total, player_name);
                        task_state.update(&task_id, progress, &msg);
                        let payload = progress::ProgressPayload::new(progress::TaskType::StartupRefresh, task_id.clone())
                            .with_progress("refreshing".to_string(), progress)
                            .with_message(msg);
                        let _ = app_handle_clone.emit("progress-update", &payload);

                        // 执行增量更新
                        let api = match crate::api::BeatLeaderApi::new() {
                            Ok(a) => a,
                            Err(_) => continue,
                        };
                        let _ = commands::player::fetch_and_save_player(
                            &app_handle_clone,
                            &task_state,
                            player_id.clone(),
                            format!("sub-{}", uuid::Uuid::new_v4()),
                            None,
                            None,
                            api,
                            true,
                            false, // 不发射进度事件，由外部整合任务统一管理
                        )
                        .await;
                    }

                    task_state.update(&task_id, 100.0, "玩家成绩刷新完成");
                    let payload = progress::ProgressPayload::new(progress::TaskType::StartupRefresh, task_id.clone())
                        .with_progress("completed".to_string(), 100.0)
                        .with_message("玩家成绩刷新完成".to_string());
                    let _ = app_handle_clone.emit("progress-update", &payload);

                    // 3秒后自动清理
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    task_state.remove(&task_id);
                });
            });

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
            commands::playlist::add_playlist,
            commands::playlist::get_playlist_list,
            commands::playlist::get_playlist,
            commands::playlist::delete_playlist,
            commands::playlist::get_playlist_maps,
            commands::playlist::refresh_playlist_ratings,
            commands::leaderboard::get_leaderboard,
            commands::common::get_version,
            task_state::get_active_tasks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}