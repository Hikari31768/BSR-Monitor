use tauri::AppHandle;
use tauri::Manager;
use tauri::Emitter;
use tauri::State;
use tauri_plugin_opener::OpenerExt;
use crate::api::BeatLeaderApi;
use crate::database::operations::{PlayerInfo};
use crate::database::operations as db;
use std::sync::Mutex;
use uuid::Uuid;
use crate::progress::{ProgressPayload, TaskType, emit_progress_event};
use crate::task_state::TaskState;
use serde::{Serialize};

/// Response from add_player/add_playlist commands
#[derive(Serialize)]
pub struct AddPlayerResult {
    pub task_id: String,
    pub player: PlayerInfo,
}

/// 玩家信息带成绩数量
#[derive(Serialize)]
pub struct PlayerWithScoreCount {
    #[serde(flatten)]
    pub player: PlayerInfo,
    pub score_count: i64,
}

/// 添加玩家（含全量拉取成绩）或增量更新（如已存在）- 支持进度反馈
#[tauri::command]
pub async fn add_player(
    app: AppHandle,
    task_state: State<'_, TaskState>,
    id: String,
    live_url: Option<String>,
    alias: Option<String>,
    task_id: Option<String>,
) -> Result<AddPlayerResult, String> {
    let _permit = task_state.player_slots.acquire().await.map_err(|e| e.to_string())?;
    let task_id = task_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let api = BeatLeaderApi::new().map_err(|e| e.to_string())?;

    // 先调用 API 获取玩家信息（支持自定义 ID，返回原始数字 ID）
    let player_response = api.get_player(&id)
        .await
        .map_err(|e| e.to_string())?;

    // 使用 API 返回的原始数字 ID 作为系统主键
    let canonical_id = player_response.id.clone();

    // 使用原始 ID 注册任务，确保前端进度恢复能正确匹配
    let _task = task_state.begin(&task_id, "add_player", &canonical_id, &player_response.name)?;

    // 使用原始 ID 检查是否已存在
    let existing = {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::get_player_by_id(&guard, &canonical_id).map_err(|e| e.to_string())?
    };

    let is_incremental = existing.is_some();
    let result = fetch_and_save_player(
        &app,
        &task_state,
        &canonical_id,
        player_response,
        task_id.clone(),
        live_url,
        alias,
        api,
        is_incremental,
        true,
        app.state::<crate::settings::SettingsState>().concurrency.add_player_pages,
    ).await;

    // 无论成功失败都移除任务记录
    task_state.remove(&task_id);

    if result.is_ok() { let _ = app.emit("players-updated", &canonical_id); }
    result.map(|player| AddPlayerResult { task_id, player })
}

/// 统一处理玩家添加/更新（公开以便 startup 调用）
pub async fn fetch_and_save_player(
    app: &AppHandle,
    task_state: &TaskState,
    id: &str,
    player_response: crate::api::beatleader::PlayerResponse,
    task_id: String,
    live_url: Option<String>,
    alias: Option<String>,
    api: BeatLeaderApi,
    is_incremental: bool,
    emit_events: bool,
    page_concurrency: usize,
) -> Result<PlayerInfo, String> {
    let message_prefix = if is_incremental { "更新" } else { "" };

    // Step 1: 发送初始进度
    let msg = format!("{}{}正在获取玩家信息...", message_prefix, if is_incremental { "" } else { "正在" });
    if emit_events {
        let payload = ProgressPayload::new(TaskType::AddPlayer, task_id.clone()).with_target(id.to_string())
            .with_progress("fetching_player_info".to_string(), 2.0)
            .with_message(msg.clone());
        emit_progress_event(app, &payload).map_err(|e| e.to_string())?;
    }
    task_state.update(&task_id, 2.0, &msg);

    // 构建玩家信息 — 使用 API 返回的原始数字 ID
    let mut player_info = PlayerInfo {
        id: id.to_string(),
        name: player_response.name.clone(),
        avatar_url: Some(player_response.avatar.clone()),
        live_url,
        alias,
        updated_at: None,
    };

    // 发送第二阶段进度（含玩家信息，前端可先行显示）
    let msg = format!("已获取玩家: {}", player_response.name);
    if emit_events {
        let payload = ProgressPayload::new(TaskType::AddPlayer, task_id.clone()).with_target(id.to_string())
            .with_progress("saving_player_info".to_string(), 5.0)
            .with_message(msg.clone())
            .with_player_name(player_response.name.clone())
            .with_player_avatar_url(player_response.avatar.clone());
        emit_progress_event(app, &payload).map_err(|e| e.to_string())?;
    }
    task_state.update(&task_id, 5.0, &msg);

    // Step 3: 存入数据库
    {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::insert_player(&guard, &player_info).map_err(|e| e.to_string())?;
    }

    // 如果是增量更新，获取本地最新成绩时间
    let latest_local_epoch = if is_incremental {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::get_score_sync_time(&guard, id).map_err(|e| e.to_string())?
    } else {
        None
    };

    // Step 4: 发送开始拉取成绩进度
    let progress_msg = if is_incremental { "正在拉取新成绩..." } else { "开始拉取成绩..." };
    if emit_events {
        let payload = ProgressPayload::new(TaskType::AddPlayer, task_id.clone()).with_target(id.to_string())
            .with_progress("fetching_scores".to_string(), 8.0)
            .with_message(progress_msg.to_string());
        emit_progress_event(app, &payload).map_err(|e| e.to_string())?;
    }
    task_state.update(&task_id, 8.0, progress_msg);

    // Step 5: 拉取成绩
    let page_size = 100;
    let mut total_scores = 0;
    let mut successful_pages = 0usize;

    let mut pager = crate::api::score_pager::ScorePager::new(api, id.to_string(), page_size, latest_local_epoch, page_concurrency);
    while let Some((page, response)) = pager.next_page().await.map_err(|e| e.to_string())? {
        let save_started = std::time::Instant::now();
        // 处理每条成绩
        let mut scores_to_insert: Vec<crate::database::operations::PlayerScore> = Vec::new();

        for score_data in &response.data {
            // 增量更新逻辑：遇到不新于本地记录的成绩时停止
            if let Some(latest) = latest_local_epoch {
                if score_data.score.epoch_time <= latest {
                    continue;
                }
            }

            let (map_key, _, _) = db::parse_leaderboard_id(&score_data.leaderboard.id);
            let difficulty = db::difficulty_from_value(score_data.leaderboard.difficulty);
            scores_to_insert.push(crate::database::operations::PlayerScore {
                player_id: id.to_string(),
                score_id: score_data.score.id as i64,
                map_key,
                hash: score_data.leaderboard.song_hash.to_lowercase(),
                mode: score_data.leaderboard.mode_name.clone(),
                difficulty,
                accuracy: score_data.score.accuracy,
                epoch_time: score_data.score.epoch_time,
            });
        }

        // 批量插入成绩
        if !scores_to_insert.is_empty() {
            let state = app.try_state::<Mutex<rusqlite::Connection>>()
                .ok_or("Database not initialized")?;
            let guard = state.lock().map_err(|e| e.to_string())?;
            total_scores += db::insert_scores_batch(&guard, &scores_to_insert).map_err(|e| e.to_string())?;
        }

        eprintln!("Score page {page}: save={:?}", save_started.elapsed());

        // 计算总页数
        let per_page = if response.metadata.items_per_page > 0 { response.metadata.items_per_page } else { page_size };
        let total_pages = response.metadata.total.saturating_add(per_page - 1) / per_page;
        successful_pages += 1;

        // 更新进度（8%-100%区间映射，成绩拉取占92%）
        let page_progress = if total_pages > 1 {
            8.0 + ((successful_pages as f64 / total_pages as f64).min(1.0) * 92.0)
        } else {
            100.0
        };
        let msg = if is_incremental {
            format!("{}/{} 页，已新增成绩 {} 条...", successful_pages, total_pages, total_scores)
        } else {
            format!("{}/{} 页，已拉取成绩 {} 条...", successful_pages, total_pages, total_scores)
        };
        if emit_events {
            let payload = ProgressPayload::new(TaskType::AddPlayer, task_id.clone()).with_target(id.to_string())
                .with_progress("fetching_scores".to_string(), page_progress)
                .with_message(msg.clone());
            emit_progress_event(app, &payload).map_err(|e| e.to_string())?;
        }
        task_state.update(&task_id, page_progress, &msg);

    }

    // Only a fully successful sync advances the visible update time.
    {
        let state = app.state::<Mutex<rusqlite::Connection>>();
        let conn = state.lock().map_err(|e| e.to_string())?;
        player_info.updated_at = Some(db::mark_player_updated(&conn, id).map_err(|e| e.to_string())?);
    }

    // 发送完成进度
    let msg = format!("{}完成！{} {} 条成绩",
        if is_incremental { "更新" } else { "" },
        if is_incremental { "新增" } else { "共拉取" },
        total_scores);
    if emit_events {
        let payload = ProgressPayload::new(TaskType::AddPlayer, task_id.clone()).with_target(id.to_string())
            .with_progress("completed".to_string(), 100.0)
            .with_message(msg.clone());
        emit_progress_event(app, &payload).map_err(|e| e.to_string())?;
    }
    task_state.update(&task_id, 100.0, &msg);

    Ok(player_info)
}

/// 获取所有玩家 ID 列表（启动刷新用）
pub fn get_all_player_ids(conn: &rusqlite::Connection) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT id FROM PlayerInfo")?;
    let ids = stmt.query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ids)
}

/// 获取玩家昵称（启动刷新显示用）
pub fn get_player_name(conn: &rusqlite::Connection, id: &str) -> Result<Option<String>, rusqlite::Error> {
    let result: Option<String> = conn.query_row(
        "SELECT COALESCE(alias, name) FROM PlayerInfo WHERE id = ?1",
        [id],
        |row| row.get(0),
    )?;
    Ok(result)
}

/// 获取玩家列表
#[tauri::command]
pub fn get_player_list(app: AppHandle) -> Result<Vec<PlayerInfo>, String> {
    let state = app.try_state::<Mutex<rusqlite::Connection>>()
        .ok_or("Database not initialized")?;
    let guard = state.lock().map_err(|e| e.to_string())?;
    db::get_all_players(&guard).map_err(|e| e.to_string())
}

/// 批量获取玩家列表（含成绩数量）
#[tauri::command]
pub fn get_players_with_score_counts(app: AppHandle) -> Result<Vec<PlayerWithScoreCount>, String> {
    let state = app.try_state::<Mutex<rusqlite::Connection>>()
        .ok_or("Database not initialized")?;
    let guard = state.lock().map_err(|e| e.to_string())?;

    let results = db::get_players_with_score_counts(&guard)
        .map_err(|e| e.to_string())?;

    Ok(results.into_iter().map(|(player, score_count)| {
        PlayerWithScoreCount { player, score_count }
    }).collect())
}

/// 获取单个玩家详情
#[tauri::command]
pub fn get_player(app: AppHandle, id: String) -> Result<Option<PlayerInfo>, String> {
    let state = app.try_state::<Mutex<rusqlite::Connection>>()
        .ok_or("Database not initialized")?;
    let guard = state.lock().map_err(|e| e.to_string())?;
    db::get_player_by_id(&guard, &id).map_err(|e| e.to_string())
}

/// 删除玩家
#[tauri::command]
pub fn delete_player(app: AppHandle, task_state: State<'_, TaskState>, id: String) -> Result<(), String> {
    // 检查是否有正在进行的添加任务
    let delete_id = format!("delete-player-{}", Uuid::new_v4());
    let _claim = task_state.begin(&delete_id, "delete_player", &id, "删除玩家")?;
    let state = app.try_state::<Mutex<rusqlite::Connection>>()
        .ok_or("Database not initialized")?;
    let guard = state.lock().map_err(|e| e.to_string())?;
    db::delete_player(&guard, &id).map_err(|e| e.to_string())
}

/// 更新玩家信息（liveURL、alias）
#[tauri::command]
pub fn update_player(app: AppHandle, player: PlayerInfo) -> Result<(), String> {
    let state = app.try_state::<Mutex<rusqlite::Connection>>()
        .ok_or("Database not initialized")?;
    let guard = state.lock().map_err(|e| e.to_string())?;
    db::update_player(&guard, &player).map_err(|e| e.to_string())
}

/// 在系统默认浏览器中打开 URL
#[tauri::command]
pub fn open_url(window: tauri::Window, url: String) -> Result<(), String> {
    window.opener().open_url(&url, None::<&str>)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// 手动刷新单个玩家的成绩（增量更新）
#[tauri::command]
pub async fn refresh_player_scores(
    app: AppHandle,
    task_state: State<'_, TaskState>,
    player_id: String,
) -> Result<String, String> {
    let task_id = format!("refresh-player-{}", Uuid::new_v4());
    refresh_saved_player(&app, &task_state, &player_id, task_id.clone(), true).await?;
    Ok(task_id)
}

/// Shared by manual refresh and the bounded startup workers.
pub(crate) async fn refresh_saved_player(
    app: &AppHandle, task_state: &TaskState, player_id: &str, task_id: String, emit_events: bool,
) -> Result<PlayerInfo, String> {
    let kind = if emit_events { "refresh_player_scores" } else { "startup_player" };
    let _task = task_state.begin(&task_id, kind, player_id, player_id)?;
    let display_name = {
        let state = app.state::<Mutex<rusqlite::Connection>>();
        let conn = state.lock().map_err(|e| e.to_string())?;
        let player = db::get_player_by_id(&conn, player_id).map_err(|e| e.to_string())?
            .ok_or("玩家已移除，跳过刷新")?;
        player.alias.unwrap_or(player.name)
    };
    task_state.register_with_name(&task_id, kind, player_id, &display_name);
    let _permit = task_state.player_slots.acquire().await.map_err(|e| e.to_string())?;
    let result = async {
        let api = BeatLeaderApi::new().map_err(|e| e.to_string())?;
        let response = api.get_player(player_id).await.map_err(|e| e.to_string())?;
        let canonical_id = response.id.clone();
        fetch_and_save_player(app, task_state, &canonical_id, response,
            task_id.clone(), None, None, api, true, emit_events, crate::settings::MAX_CONCURRENCY).await
    }.await;
    match &result {
        Ok(player) if emit_events => { let _ = app.emit("players-updated", &player.id); }
        Err(error) if emit_events => {
            let payload = ProgressPayload::new(TaskType::RefreshPlayerScores, task_id)
                .with_progress("failed".into(), 100.0)
                .with_target(player_id.to_string()).with_message(error.clone());
            let _ = emit_progress_event(app, &payload);
        }
        _ => {}
    }
    result
}
