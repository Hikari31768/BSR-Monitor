use tauri::AppHandle;
use tauri::{Manager, Emitter, State};
use crate::task_state::TaskState;
use crate::api::BeatLeaderApi;
use crate::database::operations::PlayerScore;
use crate::database::operations as db;
use crate::database::operations::PlaylistSource;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

// ============ 打谱查询相关命令 ============

pub use crate::database::score_matching::MapScoresResponse;
use crate::database::score_matching::build_map_scores_response;
use std::collections::{HashMap, HashSet};

/// PlaylistScoresResponse 结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaylistScoresResponse {
    pub playlist: db::Playlist,
    pub page: i32,
    pub total_pages: i32,
    pub maps: Vec<MapScoresResponse>,
}

/// 查询单个谱面在指定玩家下的成绩
#[tauri::command]
pub async fn get_map_scores(
    app: AppHandle,
    key: String,
    player_ids: Vec<String>,
) -> Result<MapScoresResponse, String> {
    let state = app.try_state::<Mutex<rusqlite::Connection>>()
        .ok_or("Database not initialized")?;
    let guard = state.lock().map_err(|e| e.to_string())?;

    let map = db::get_map_by_key(&guard, &key)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Map not found: {}", key))?;
    let all_diffs = db::get_map_diffs_by_key(&guard, &key).map_err(|e| e.to_string())?;

    let scores = db::get_scores_for_maps(&guard, &[map.key.clone()], &player_ids)
        .map_err(|e| e.to_string())?;
    Ok(build_map_scores_response(map, &all_diffs, &scores))
}

/// 查询歌单某页谱面在指定玩家下的成绩
#[tauri::command]
pub async fn get_playlist_scores(
    app: AppHandle,
    source: String,
    identifier: String,
    page: i32,
    player_ids: Vec<String>,
    reverse: bool,
) -> Result<PlaylistScoresResponse, String> {
    let playlist_source = match source.as_str() {
        "beatsaver" => PlaylistSource::Beatsaver,
        "mapper" => PlaylistSource::Mapper,
        "local" => PlaylistSource::Local,
        _ => return Err(format!("Invalid playlist source: {}", source)),
    };

    let (playlist, map_hashes, total_pages) = {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;

        // 1. 获取歌单信息
        let playlist = db::get_playlist(&guard, &playlist_source, &identifier)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Playlist not found: {}/{}", source, identifier))?;

        // 2. 获取分页的谱面 hashes
        let (map_hashes, total_pages) = db::get_playlist_hashes_paginated(&guard, &playlist_source, &identifier, page, 100, reverse)
            .map_err(|e| e.to_string())?;

        (playlist, map_hashes, total_pages)
    };

    // 3. 批量读取本页谱面、所有版本难度和所选玩家成绩。
    let state = app.try_state::<Mutex<rusqlite::Connection>>()
        .ok_or("Database not initialized")?;
    let guard = state.lock().map_err(|e| e.to_string())?;

    let by_hash: HashMap<_, _> = db::get_maps_by_hashes(&guard, &map_hashes)
        .map_err(|e| e.to_string())?.into_iter().collect();
    let keys: Vec<String> = by_hash.values().map(|m| m.key.clone()).collect::<HashSet<_>>().into_iter().collect();
    let mut diffs_by_key: HashMap<String, Vec<db::MapDiff>> = HashMap::new();
    for diff in db::get_map_diffs_for_keys(&guard, &keys).map_err(|e| e.to_string())? {
        diffs_by_key.entry(diff.key.clone()).or_default().push(diff);
    }
    let mut scores_by_key: HashMap<String, Vec<PlayerScore>> = HashMap::new();
    for score in db::get_scores_for_maps(&guard, &keys, &player_ids).map_err(|e| e.to_string())? {
        if let Some(key) = &score.map_key {
            scores_by_key.entry(key.clone()).or_default().push(score);
        }
    }
    let mut maps = Vec::new();
    for hash in map_hashes {
        if let Some(map) = by_hash.get(&hash.to_lowercase()) {
            let diffs = diffs_by_key.get(&map.key).map(Vec::as_slice).unwrap_or(&[]);
            let scores = scores_by_key.get(&map.key).map(Vec::as_slice).unwrap_or(&[]);
            maps.push(build_map_scores_response(map.clone(), diffs, scores));
        }
    }

    Ok(PlaylistScoresResponse {
        playlist,
        page,
        total_pages,
        maps,
    })
}

/// 增量更新玩家成绩
#[tauri::command]
pub async fn update_player_scores(
    app: AppHandle,
    task_state: State<'_, TaskState>,
    player_id: String,
) -> Result<i64, String> {
    let task_id = format!("update-player-{}", uuid::Uuid::new_v4());
    let _task = task_state.begin(&task_id, "refresh_player_scores", &player_id, &player_id)?;
    let _permit = task_state.player_slots.acquire().await.map_err(|e| e.to_string())?;
    let api = BeatLeaderApi::new().map_err(|e| e.to_string())?;
    let page_size = 100;
    let mut total_updated = 0;

    // 获取本地最新成绩时间
    let latest_local_epoch = {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::get_score_sync_time(&guard, &player_id).map_err(|e| e.to_string())?
    };

    let mut pager = crate::api::score_pager::ScorePager::new(api, player_id.clone(), page_size, latest_local_epoch, crate::settings::MAX_CONCURRENCY);
    while let Some((page, response)) = pager.next_page().await.map_err(|e| e.to_string())? {
        let save_started = std::time::Instant::now();
        if response.data.is_empty() {
            break;
        }

        // 批量处理成绩
        let mut scores_to_upsert: Vec<PlayerScore> = Vec::new();

        for score_data in response.data {
            // 增量更新逻辑：遇到不新于本地记录的成绩时停止
            if let Some(latest) = latest_local_epoch {
                if score_data.score.epoch_time <= latest {
                    break;
                }
            }

            // 转换成绩数据
            let (map_key, _, _) = db::parse_leaderboard_id(&score_data.leaderboard.id);
            let difficulty = db::difficulty_from_value(score_data.leaderboard.difficulty);
            scores_to_upsert.push(PlayerScore {
                player_id: player_id.clone(),
                score_id: score_data.score.id as i64,
                leaderboard_id: Some(score_data.leaderboard.id.clone()),
                map_key,
                hash: score_data.leaderboard.song_hash.to_lowercase(),
                mode: score_data.leaderboard.mode_name.clone(),
                difficulty,
                accuracy: score_data.score.accuracy,
                epoch_time: score_data.score.epoch_time,
            });
        }

        // 批量插入/更新成绩
        if !scores_to_upsert.is_empty() {
            let state = app.try_state::<Mutex<rusqlite::Connection>>()
                .ok_or("Database not initialized")?;
            let guard = state.lock().map_err(|e| e.to_string())?;
            total_updated += db::insert_scores_batch(&guard, &scores_to_upsert).map_err(|e| e.to_string())? as i64;
        }
        eprintln!("Score page {page}: save={:?}", save_started.elapsed());
    }

    {
        let state = app.state::<Mutex<rusqlite::Connection>>();
        let conn = state.lock().map_err(|e| e.to_string())?;
        db::mark_player_updated(&conn, &player_id).map_err(|e| e.to_string())?;
    }
    let _ = app.emit("players-updated", &player_id);
    Ok(total_updated)
}

/// 获取玩家成绩列表
#[tauri::command]
pub fn get_player_scores(app: AppHandle, player_id: String) -> Result<Vec<PlayerScore>, String> {
    let state = app.try_state::<Mutex<rusqlite::Connection>>()
        .ok_or("Database not initialized")?;
    let guard = state.lock().map_err(|e| e.to_string())?;
    db::get_scores_by_player(&guard, &player_id).map_err(|e| e.to_string())
}

/// 获取玩家成绩数量
#[tauri::command]
pub fn get_player_score_count(app: AppHandle, player_id: String) -> Result<i64, String> {
    let state = app.try_state::<Mutex<rusqlite::Connection>>()
        .ok_or("Database not initialized")?;
    let guard = state.lock().map_err(|e| e.to_string())?;
    db::get_score_count_by_player(&guard, &player_id).map_err(|e| e.to_string())
}
