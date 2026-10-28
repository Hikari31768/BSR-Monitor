use std::{sync::Mutex, time::Duration};
use tauri::{AppHandle, Manager, Emitter};
use crate::{commands, database::operations as db, progress::{emit_progress_event, ProgressPayload, TaskType},
    sync_batch::{run_bounded, BatchSummary}, task_state::TaskState};

pub fn spawn(app: AppHandle) {
    // Use the application's runtime so playlist enrichment can outlive the
    // startup batch; a temporary runtime would cancel those background tasks.
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let snapshot = (|| -> Result<_, String> {
            let state = app.state::<Mutex<rusqlite::Connection>>();
            let conn = state.lock().map_err(|e| e.to_string())?;
            Ok((db::get_all_players(&conn).map_err(|e| e.to_string())?,
                db::get_all_playlists(&conn).map_err(|e| e.to_string())?))
        })();
        match snapshot {
            Ok((players, playlists)) => {
                tokio::join!(refresh_players(app.clone(), players), refresh_playlists(app.clone(), playlists));
                // Refresh the score query only after both startup batches finish.
                let _ = app.emit("players-updated", "");
                let _ = app.emit("playlists-updated", "");
            }
            Err(error) => eprintln!("启动刷新失败: {error}"),
        }
    });
}

fn report(app: &AppHandle, task_id: &str, label: &str, total: usize, summary: &BatchSummary, done: bool) {
    let progress = if done { 100.0 } else { summary.completed as f64 / total as f64 * 100.0 };
    let message = if done {
        format!("{label}完成：成功 {}，失败 {}", summary.completed - summary.failed, summary.failed)
    } else {
        format!("{label}：已完成 {}/{}，失败 {}", summary.completed, total, summary.failed)
    };
    app.state::<TaskState>().update(task_id, progress, &message);
    let payload = ProgressPayload::new(TaskType::StartupRefresh, task_id.to_string())
        .with_progress(if done { "completed" } else { "refreshing" }.into(), progress)
        .with_display_name(format!("启动{label}"))
        .with_message(message);
    let _ = emit_progress_event(app, &payload);
}

async fn refresh_players(app: AppHandle, players: Vec<db::PlayerInfo>) {
    if players.is_empty() { return; }
    let task_id = format!("startup-players-{}", uuid::Uuid::new_v4());
    let state = app.state::<TaskState>();
    let Ok(_task) = state.begin(&task_id, "startup_refresh", "startup:players", "启动刷新玩家成绩") else { return; };
    let total = players.len();
    report(&app, &task_id, "刷新玩家成绩", total, &BatchSummary::default(), false);
    let worker_app = app.clone();
    let limit = app.state::<crate::settings::SettingsState>().concurrency.startup_players;
    let summary = run_bounded(players, limit, move |player| {
        let app = worker_app.clone();
        async move {
            let state = app.state::<TaskState>();
            commands::player::refresh_saved_player(&app, &state, &player.id,
                format!("startup-player-{}", uuid::Uuid::new_v4()), false).await
                .map(|_| player.name.clone()).map_err(|e| format!("{}: {e}", player.name))
        }
    }, |summary, result| {
        if let Err(error) = result { eprintln!("玩家刷新失败: {error}"); }
        report(&app, &task_id, "刷新玩家成绩", total, summary, false);
    }).await;
    report(&app, &task_id, "刷新玩家成绩", total, &summary, true);
}

async fn refresh_playlists(app: AppHandle, playlists: Vec<db::Playlist>) {
    if playlists.is_empty() { return; }
    let task_id = format!("startup-playlists-{}", uuid::Uuid::new_v4());
    let state = app.state::<TaskState>();
    let Ok(_task) = state.begin(&task_id, "startup_refresh", "startup:playlists", "启动刷新歌单") else { return; };
    let total = playlists.len();
    report(&app, &task_id, "刷新歌单", total, &BatchSummary::default(), false);
    let worker_app = app.clone();
    let limit = app.state::<crate::settings::SettingsState>().concurrency.startup_playlists;
    let summary = run_bounded(playlists, limit, move |playlist| {
        let app = worker_app.clone();
        async move {
            let state = app.state::<TaskState>();
            commands::playlist::refresh_saved_playlist(&app, &state, &playlist.source, &playlist.identifier,
                format!("startup-playlist-{}", uuid::Uuid::new_v4()), false).await
                .map(|_| playlist.name.clone()).map_err(|e| format!("{}: {e}", playlist.name))
        }
    }, |summary, result| {
        if let Err(error) = result { eprintln!("歌单刷新失败: {error}"); }
        report(&app, &task_id, "刷新歌单", total, summary, false);
    }).await;
    report(&app, &task_id, "刷新歌单", total, &summary, true);
}
