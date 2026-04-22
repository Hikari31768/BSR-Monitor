use tauri::AppHandle;
use tauri::Manager;
use crate::api::BeatLeaderApi;
use crate::database::operations::{PlayerInfo};
use crate::database::operations as db;
use std::sync::Mutex;
use uuid::Uuid;
use crate::progress::{ProgressPayload, emit_progress_event};

/// 添加玩家（含全量拉取成绩）或增量更新（如已存在）- 支持进度反馈
#[tauri::command]
pub async fn add_player(
    app: AppHandle,
    id: String,
    live_url: Option<String>,
    alias: Option<String>,
) -> Result<PlayerInfo, String> {
    let task_id = Uuid::new_v4().to_string();
    let api = BeatLeaderApi::new().map_err(|e| e.to_string())?;

    // 检查是否已存在
    let existing = {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::get_player_by_id(&guard, &id).map_err(|e| e.to_string())?
    };

    if existing.is_some() {
        // 已存在 -> 增量更新
        update_existing_player(app, id, task_id, live_url, alias, api).await
    } else {
        // 不存在 -> 全量添加
        add_new_player(app, id, task_id, live_url, alias, api).await
    }
}

/// 全量添加新玩家
async fn add_new_player(
    app: AppHandle,
    id: String,
    task_id: String,
    live_url: Option<String>,
    alias: Option<String>,
    api: BeatLeaderApi,
) -> Result<PlayerInfo, String> {
    // Step 1: 发送初始进度
    let payload = ProgressPayload::new("add_player".to_string(), task_id.clone())
        .with_progress("fetching_player_info".to_string(), 10.0)
        .with_message("正在获取玩家信息...".to_string());
    emit_progress_event(&app, &payload).map_err(|e| e.to_string())?;

    // Step 2: 调用 API 获取玩家信息
    let player_response = api.get_player(&id)
        .await
        .map_err(|e| e.to_string())?;

    // 构建玩家信息
    let player_info = PlayerInfo {
        id: player_response.id.clone(),
        name: player_response.name.clone(),
        avatar_url: Some(player_response.avatar.clone()),
        live_url,
        alias,
    };

    // 发送第二阶段进度（含玩家信息，前端可先行显示）
    let payload = ProgressPayload::new("add_player".to_string(), task_id.clone())
        .with_progress("saving_player_info".to_string(), 20.0)
        .with_message(format!("已获取玩家: {}", player_response.name))
        .with_player_info(player_response.name.clone(), player_response.avatar.clone());
    emit_progress_event(&app, &payload).map_err(|e| e.to_string())?;

    // Step 3: 存入数据库
    {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::insert_player(&guard, &player_info).map_err(|e| e.to_string())?;
    }

    // Step 4: 发送开始拉取成绩进度
    let payload = ProgressPayload::new("add_player".to_string(), task_id.clone())
        .with_progress("fetching_scores".to_string(), 30.0)
        .with_message("开始拉取成绩...".to_string());
    emit_progress_event(&app, &payload).map_err(|e| e.to_string())?;

    // Step 5: 全量拉取成绩
    let page_size = 100;
    let mut page = 1;
    let mut total_scores = 0;

    loop {
        // API 调用
        let response = api.get_player_scores(&id, page, page_size)
            .await
            .map_err(|e| e.to_string())?;

        if response.data.is_empty() {
            break;
        }

        // 转换成绩数据
        let scores_to_insert: Vec<crate::database::operations::PlayerScore> = response.data
            .iter()
            .map(|score_data| {
                let (map_key, _, _) = db::parse_leaderboard_id(&score_data.leaderboard.id);
                let difficulty = db::difficulty_from_value(score_data.leaderboard.difficulty);
                crate::database::operations::PlayerScore {
                    player_id: id.clone(),
                    score_id: score_data.score.id as i64,
                    leaderboard_id: score_data.leaderboard.id.clone(),
                    map_key,
                    hash: score_data.leaderboard.song_hash.clone(),
                    mode: score_data.leaderboard.mode_name.clone(),
                    difficulty,
                    accuracy: score_data.score.accuracy,
                    epoch_time: score_data.score.epoch_time,
                }
            })
            .collect();

        // 数据库操作
        {
            let state = app.try_state::<Mutex<rusqlite::Connection>>()
                .ok_or("Database not initialized")?;
            let guard = state.lock().map_err(|e| e.to_string())?;
            for score in &scores_to_insert {
                db::insert_score(&guard, score).map_err(|e| e.to_string())?;
            }
        }

        // 计算总页数
        let total_pages = (response.metadata.total + response.metadata.items_per_page - 1)
            / response.metadata.items_per_page;

        // 更新进度（30%-100%区间映射）
        total_scores += scores_to_insert.len();
        let page_progress = if total_pages > 1 {
            30.0 + ((page as f64 / total_pages as f64) * 70.0)
        } else {
            100.0
        };
        let payload = ProgressPayload::new("add_player".to_string(), task_id.clone())
            .with_progress("fetching_scores".to_string(), page_progress)
            .with_message(format!("正在拉取第 {} 页成绩 (共约 {} 页)，已累计 {} 条成绩...", page, total_pages, total_scores));
        emit_progress_event(&app, &payload).map_err(|e| e.to_string())?;

        // 检查是否还有更多页
        if page >= total_pages {
            break;
        }
        page += 1;
    }

    // 发送完成进度
    let payload = ProgressPayload::new("add_player".to_string(), task_id.clone())
        .with_progress("completed".to_string(), 100.0)
        .with_message(format!("完成！共拉取 {} 条成绩", total_scores));
    emit_progress_event(&app, &payload).map_err(|e| e.to_string())?;

    Ok(player_info)
}

/// 增量更新已存在的玩家
async fn update_existing_player(
    app: AppHandle,
    id: String,
    task_id: String,
    live_url: Option<String>,
    alias: Option<String>,
    api: BeatLeaderApi,
) -> Result<PlayerInfo, String> {
    // Step 1: 发送初始进度
    let payload = ProgressPayload::new("add_player".to_string(), task_id.clone())
        .with_progress("fetching_player_info".to_string(), 10.0)
        .with_message("正在获取玩家信息...".to_string());
    emit_progress_event(&app, &payload).map_err(|e| e.to_string())?;

    // Step 2: 调用 API 获取最新玩家信息
    let player_response = api.get_player(&id)
        .await
        .map_err(|e| e.to_string())?;

    // 构建玩家信息（保留已有 live_url/alias）
    let player_info = PlayerInfo {
        id: player_response.id.clone(),
        name: player_response.name.clone(),
        avatar_url: Some(player_response.avatar.clone()),
        live_url,
        alias,
    };

    // 发送进度（含玩家信息）
    let payload = ProgressPayload::new("add_player".to_string(), task_id.clone())
        .with_progress("saving_player_info".to_string(), 20.0)
        .with_message(format!("已获取玩家: {}", player_response.name))
        .with_player_info(player_response.name.clone(), player_response.avatar.clone());
    emit_progress_event(&app, &payload).map_err(|e| e.to_string())?;

    // Step 3: 更新数据库中的玩家信息
    {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::insert_player(&guard, &player_info).map_err(|e| e.to_string())?;
    }

    // Step 4: 获取本地最新成绩时间，用于增量拉取
    let latest_local_epoch = {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::get_latest_score_time(&guard, &id).map_err(|e| e.to_string())?
    };

    // Step 5: 发送开始更新成绩进度
    let payload = ProgressPayload::new("add_player".to_string(), task_id.clone())
        .with_progress("fetching_scores".to_string(), 30.0)
        .with_message("正在拉取新成绩...".to_string());
    emit_progress_event(&app, &payload).map_err(|e| e.to_string())?;

    // Step 6: 增量拉取成绩（从最新的一条开始）
    let page_size = 100;
    let mut page = 1;
    let mut total_new_scores = 0;

    loop {
        // API 调用
        let response = api.get_player_scores(&id, page, page_size)
            .await
            .map_err(|e| e.to_string())?;

        if response.data.is_empty() {
            break;
        }

        let mut reached_local = false;

        // 转换并插入成绩
        for score_data in &response.data {
            // 增量更新逻辑：遇到不新于本地记录的成绩时停止
            if let Some(latest) = latest_local_epoch {
                if score_data.score.epoch_time <= latest {
                    reached_local = true;
                    break;
                }
            }

            let (map_key, _, _) = db::parse_leaderboard_id(&score_data.leaderboard.id);
            let difficulty = db::difficulty_from_value(score_data.leaderboard.difficulty);
            let score = crate::database::operations::PlayerScore {
                player_id: id.clone(),
                score_id: score_data.score.id as i64,
                leaderboard_id: score_data.leaderboard.id.clone(),
                map_key,
                hash: score_data.leaderboard.song_hash.clone(),
                mode: score_data.leaderboard.mode_name.clone(),
                difficulty,
                accuracy: score_data.score.accuracy,
                epoch_time: score_data.score.epoch_time,
            };

            {
                let state = app.try_state::<Mutex<rusqlite::Connection>>()
                    .ok_or("Database not initialized")?;
                let guard = state.lock().map_err(|e| e.to_string())?;
                db::insert_score(&guard, &score).map_err(|e| e.to_string())?;
            }
            total_new_scores += 1;
        }

        if reached_local {
            break;
        }

        // 计算总页数
        let total_pages = (response.metadata.total + response.metadata.items_per_page - 1)
            / response.metadata.items_per_page;

        // 更新进度（30%-100%区间映射）
        let page_progress = if total_pages > 1 {
            30.0 + ((page as f64 / total_pages as f64) * 70.0)
        } else {
            100.0
        };
        let payload = ProgressPayload::new("add_player".to_string(), task_id.clone())
            .with_progress("fetching_scores".to_string(), page_progress)
            .with_message(format!("正在拉取第 {} 页成绩 (共约 {} 页)，已新增 {} 条...", page, total_pages, total_new_scores));
        emit_progress_event(&app, &payload).map_err(|e| e.to_string())?;

        // 检查是否还有更多页
        if page >= total_pages {
            break;
        }
        page += 1;
    }

    // 发送完成进度
    let payload = ProgressPayload::new("add_player".to_string(), task_id.clone())
        .with_progress("completed".to_string(), 100.0)
        .with_message(format!("更新完成！新增 {} 条成绩", total_new_scores));
    emit_progress_event(&app, &payload).map_err(|e| e.to_string())?;

    Ok(player_info)
}

/// 获取玩家列表
#[tauri::command]
pub fn get_player_list(app: AppHandle) -> Result<Vec<PlayerInfo>, String> {
    let state = app.try_state::<Mutex<rusqlite::Connection>>()
        .ok_or("Database not initialized")?;
    let guard = state.lock().map_err(|e| e.to_string())?;
    db::get_all_players(&guard).map_err(|e| e.to_string())
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
pub fn delete_player(app: AppHandle, id: String) -> Result<(), String> {
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
