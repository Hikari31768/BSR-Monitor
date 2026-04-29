use tauri::AppHandle;
use tauri::Manager;
use crate::api::BeatLeaderApi;
use crate::database::operations::PlayerScore;
use crate::database::operations as db;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

// ============ 新增：打谱查询相关命令 ============

/// 单个谱面的难度信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CurrentDiffInfo {
    pub mode: String,
    pub difficulty: String,
    pub hash: String,
    #[serde(default)]
    pub stars: Option<f32>,
}

/// 成绩行数据
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScoreRowData {
    pub mode: String,
    pub difficulty: String,
    pub is_current_version: bool, // 是否当前版本难度
    #[serde(default)]
    pub stars: Option<f32>,       // 星级
    pub scores: Vec<PlayerScoreCell>,
}

/// 单个玩家在某个难度的成绩单元格
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerScoreCell {
    pub player_id: String,
    pub accuracy: f32,
}


/// MapScoresResponse 结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MapScoresResponse {
    pub map: db::Map,
    pub current_diffs: Vec<CurrentDiffInfo>,
    pub scores: Vec<ScoreRowData>,
}

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

    build_map_scores_response(&guard, map, all_diffs, &player_ids)
}

/// 查询歌单某页谱面在指定玩家下的成绩
#[tauri::command]
pub async fn get_playlist_scores(
    app: AppHandle,
    playlist_id: i64,
    page: i32,
    player_ids: Vec<String>,
    reverse: bool,
) -> Result<PlaylistScoresResponse, String> {
    let (playlist, map_keys, total_pages) = {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;

        // 1. 获取歌单信息
        let playlist = db::get_playlist_by_id(&guard, playlist_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Playlist not found: {}", playlist_id))?;

        // 2. 获取分页的谱面 keys
        let (map_keys, total_pages) = db::get_playlist_maps_paginated(&guard, playlist_id, page, 100, reverse)
            .map_err(|e| e.to_string())?;

        (playlist, map_keys, total_pages)
    }; // guard 在此处被 drop

    // 3. 逐个查询每个谱面的成绩（直接使用已有星级）
    let state = app.try_state::<Mutex<rusqlite::Connection>>()
        .ok_or("Database not initialized")?;
    let guard = state.lock().map_err(|e| e.to_string())?;

    let mut maps: Vec<MapScoresResponse> = Vec::new();
    for key in map_keys {
        // 使用相同的逻辑查询单个谱面
        let all_diffs = db::get_map_diffs_by_key(&guard, &key).map_err(|e| e.to_string())?;

        if all_diffs.is_empty() {
            continue;
        }

        let map = match db::get_map_by_key(&guard, &key).map_err(|e| e.to_string())? {
            Some(m) => m,
            None => continue,
        };

        match build_map_scores_response(&guard, map, all_diffs, &player_ids) {
            Ok(response) => maps.push(response),
            Err(_) => continue,
        }
    }

    Ok(PlaylistScoresResponse {
        playlist,
        page,
        total_pages,
        maps,
    })
}

/// 构建单个谱面的成绩响应（公共函数）
fn build_map_scores_response(
    guard: &std::sync::MutexGuard<rusqlite::Connection>,
    map: db::Map,
    all_diffs: Vec<db::MapDiff>,
    player_ids: &[String],
) -> Result<MapScoresResponse, String> {
    // 1. 找出当前版本的难度列表（包含星级）
    let current_diffs: Vec<CurrentDiffInfo> = all_diffs
        .iter()
        .filter(|d| d.is_current)
        .map(|d| CurrentDiffInfo {
            mode: d.mode.clone(),
            difficulty: d.difficulty.clone(),
            hash: d.hash.clone(),
            stars: d.stars_rating,
        })
        .collect();

    // 2. 获取所有版本的 hash 列表
    let all_hashes: Vec<String> = all_diffs
        .iter()
        .map(|d| d.hash.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    // 3. 查询所有指定玩家的成绩
    let mut all_scores: Vec<PlayerScore> = Vec::new();
    for player_id in player_ids {
        let player_scores = db::get_scores_by_hashes_and_player(guard, &all_hashes, player_id)
            .map_err(|e| e.to_string())?;
        all_scores.extend(player_scores);
    }

    // 4. 构建成绩行（从 all_diffs 中获取星级）
    let score_rows = build_score_rows(&current_diffs, &all_diffs, all_scores);

    Ok(MapScoresResponse {
        map,
        current_diffs,
        scores: score_rows,
    })
}

/// 构建成绩行（每个难度一行，scores 为空表示无人打过）
fn build_score_rows(
    current_diffs: &[CurrentDiffInfo],
    all_diffs: &[db::MapDiff],
    all_scores: Vec<PlayerScore>,
) -> Vec<ScoreRowData> {
    // 1. 构建当前版本难度集合 + 星级映射
    let current_diff_set: std::collections::HashSet<(String, String)> = current_diffs
        .iter()
        .map(|d| (d.mode.clone(), d.difficulty.clone()))
        .collect();

    // 2. 按 (mode, difficulty) 分组成绩
    let mut scores_by_diff: std::collections::HashMap<(String, String), Vec<(String, PlayerScore)>> = std::collections::HashMap::new();
    for score in all_scores {
        let key = (score.mode.clone(), score.difficulty.clone());
        scores_by_diff.entry(key).or_default().push((score.player_id.clone(), score));
    }

    // 3. 为每个当前版本难度创建一行
    let mut score_rows: Vec<ScoreRowData> = Vec::new();

    for diff in current_diffs {
        let key = (diff.mode.clone(), diff.difficulty.clone());
        let player_scores = scores_by_diff.get(&key).cloned().unwrap_or_default();

        let mut player_best: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
        for (player_id, score) in player_scores {
            let entry = player_best.entry(player_id).or_insert(0.0);
            if score.accuracy > *entry {
                *entry = score.accuracy;
            }
        }

        let scores: Vec<PlayerScoreCell> = player_best
            .into_iter()
            .map(|(player_id, accuracy)| PlayerScoreCell {
                player_id,
                accuracy,
            })
            .collect();

        // 查找该难度的星级（从 all_diffs 中按 hash+mode+difficulty 匹配）
        let stars = all_diffs
            .iter()
            .find(|d| d.mode == diff.mode && d.difficulty == diff.difficulty && d.is_current)
            .and_then(|d| d.stars_rating);

        // 无论有无成绩都创建一行
        score_rows.push(ScoreRowData {
            mode: diff.mode.clone(),
            difficulty: diff.difficulty.clone(),
            is_current_version: true,
            stars,
            scores,
        });
    }

    // 4. 添加有成绩但不在当前版本的难度
    for (key, player_scores) in &scores_by_diff {
        if !current_diff_set.contains(key) {
            let mut player_best: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
            for (player_id, score) in player_scores {
                let entry = player_best.entry(player_id.clone()).or_insert(0.0);
                if score.accuracy > *entry {
                    *entry = score.accuracy;
                }
            }

            let scores: Vec<PlayerScoreCell> = player_best
                .into_iter()
                .map(|(player_id, accuracy)| PlayerScoreCell {
                    player_id,
                    accuracy,
                })
                .collect();

            score_rows.push(ScoreRowData {
                mode: key.0.clone(),
                difficulty: key.1.clone(),
                is_current_version: false,
                stars: None, // 旧版本不显示星级
                scores,
            });
        }
    }

    score_rows
}

/// 按需拉取并存储谱面星级（顺序调用，避免并发给 BeatLeader 造成负担）
async fn fetch_and_store_stars(
    app: &AppHandle,
    hashes: &[String],
) -> Result<(), String> {
    if hashes.is_empty() {
        return Ok(());
    }

    // 找出尚未拉取星级的 hash
    let hashes_to_fetch = {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::get_hashes_without_stars(&guard, hashes).map_err(|e| e.to_string())?
    };

    if hashes_to_fetch.is_empty() {
        return Ok(());
    }

    let api = BeatLeaderApi::new().map_err(|e| e.to_string())?;

    // 顺序拉取每个 hash 的星级数据
    for hash in &hashes_to_fetch {
        match api.get_leaderboards_by_hash(hash).await {
            Ok(response) => {
                // 将每个难度的星级写入数据库
                let state = app.try_state::<Mutex<rusqlite::Connection>>()
                    .ok_or("Database not initialized")?;
                let guard = state.lock().map_err(|e| e.to_string())?;
                for entry in &response.leaderboards {
                    let diff = &entry.difficulty;
                    if let Some(stars) = diff.stars {
                        let mode_name = if diff.mode_name.is_empty() {
                            "Standard".to_string()
                        } else {
                            diff.mode_name.clone()
                        };
                        let difficulty_name = if diff.difficulty_name.is_empty() {
                            db::difficulty_from_value(diff.value)
                        } else {
                            diff.difficulty_name.clone()
                        };
                        let _ = db::update_diff_stars(&guard, hash, &mode_name, &difficulty_name, stars);
                    }
                }
            }
            Err(_) => {
                // 单个 hash 拉取失败不影响其他 hash
                continue;
            }
        }
    }

    Ok(())
}

/// 异步拉取单谱面的星级（前端在拿到数据后静默调用）
#[tauri::command]
pub async fn fetch_map_stars(
    app: AppHandle,
    key: String,
) -> Result<(), String> {
    let all_hashes: Vec<String> = {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        let diffs = db::get_map_diffs_by_key(&guard, &key).map_err(|e| e.to_string())?;
        diffs.iter().map(|d| d.hash.clone()).collect::<std::collections::HashSet<_>>().into_iter().collect()
    };
    if all_hashes.is_empty() {
        return Ok(());
    }
    fetch_and_store_stars(&app, &all_hashes).await
}

/// 异步拉取歌单当前页所有谱面的星级（前端在拿到数据后静默调用）
#[tauri::command]
pub async fn fetch_playlist_stars(
    app: AppHandle,
    playlist_id: i64,
    page: i32,
    reverse: bool,
) -> Result<(), String> {
    let all_hashes: Vec<String> = {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        let (map_keys, _) = db::get_playlist_maps_paginated(&guard, playlist_id, page, 100, reverse)
            .map_err(|e| e.to_string())?;
        let mut hashes: Vec<String> = Vec::new();
        for key in &map_keys {
            let diffs = db::get_map_diffs_by_key(&guard, key).map_err(|e| e.to_string())?;
            for d in &diffs {
                if !hashes.contains(&d.hash) {
                    hashes.push(d.hash.clone());
                }
            }
        }
        hashes
    };
    if all_hashes.is_empty() {
        return Ok(());
    }
    fetch_and_store_stars(&app, &all_hashes).await
}

/// 增量更新玩家成绩
#[tauri::command]
pub async fn update_player_scores(
    app: AppHandle,
    player_id: String,
) -> Result<i64, String> {
    let api = BeatLeaderApi::new().map_err(|e| e.to_string())?;
    let page_size = 100;
    let mut page = 1;
    let mut total_updated = 0;
    let mut should_stop = false;

    // 获取本地最新成绩时间
    let latest_local_epoch = {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::get_latest_score_time(&guard, &player_id).map_err(|e| e.to_string())?
    };

    loop {
        if should_stop {
            break;
        }

        // API 调用
        let response = api.get_player_scores(&player_id, page, page_size)
            .await
            .map_err(|e| e.to_string())?;

        if response.data.is_empty() {
            break;
        }

        // 批量处理成绩
        let mut scores_to_upsert: Vec<PlayerScore> = Vec::new();

        for score_data in response.data {
            // 增量更新逻辑：遇到不新于本地记录的成绩时停止
            if let Some(latest) = latest_local_epoch {
                if score_data.score.epoch_time <= latest {
                    should_stop = true;
                    break;
                }
            }

            // 转换成绩数据
            let (map_key, _, _) = db::parse_leaderboard_id(&score_data.leaderboard.id);
            let difficulty = db::difficulty_from_value(score_data.leaderboard.difficulty);
            scores_to_upsert.push(PlayerScore {
                player_id: player_id.clone(),
                score_id: score_data.score.id as i64,
                leaderboard_id: score_data.leaderboard.id.clone(),
                map_key,
                hash: score_data.leaderboard.song_hash.clone(),
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
            for score in &scores_to_upsert {
                db::insert_score(&guard, score).map_err(|e| e.to_string())?;
            }
            total_updated += scores_to_upsert.len() as i64;
        }

        // 检查是否还有更多页
        let total_pages = (response.metadata.total + response.metadata.items_per_page - 1)
            / response.metadata.items_per_page;
        if page >= total_pages {
            break;
        }

        page += 1;
    }

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
