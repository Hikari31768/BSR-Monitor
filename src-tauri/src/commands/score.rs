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
}

/// 成绩行数据
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScoreRowData {
    pub mode: String,
    pub difficulty: String,
    pub is_current_version: bool, // 是否当前版本难度
    pub scores: Vec<PlayerScoreCell>,
}

/// 单个玩家在某个难度的成绩单元格
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerScoreCell {
    pub player_id: String,
    pub accuracy: f32,
}

/// 无成绩难度的分组（按模式）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NoScoreDiffGroup {
    pub mode: String,
    pub difficulties: Vec<String>,
}

/// MapScoresResponse 结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MapScoresResponse {
    pub map: db::Map,
    pub current_diffs: Vec<CurrentDiffInfo>,
    pub scores: Vec<ScoreRowData>,
    pub no_score_diffs: Vec<NoScoreDiffGroup>, // 无成绩的难度（按模式分组）
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
pub fn get_map_scores(
    app: AppHandle,
    key: String,
    player_ids: Vec<String>,
) -> Result<MapScoresResponse, String> {
    let state = app.try_state::<Mutex<rusqlite::Connection>>()
        .ok_or("Database not initialized")?;
    let guard = state.lock().map_err(|e| e.to_string())?;

    // 1. 获取谱面基本信息
    let map = db::get_map_by_key(&guard, &key)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Map not found: {}", key))?;

    // 2. 获取谱面所有版本的难度信息
    let all_diffs = db::get_map_diffs_by_key(&guard, &key).map_err(|e| e.to_string())?;

    // 3. 找出当前版本的难度列表
    let current_diffs: Vec<CurrentDiffInfo> = all_diffs
        .iter()
        .filter(|d| d.is_current)
        .map(|d| CurrentDiffInfo {
            mode: d.mode.clone(),
            difficulty: d.difficulty.clone(),
            hash: d.hash.clone(),
        })
        .collect();

    // 4. 获取所有版本的 hash 列表
    let all_hashes: Vec<String> = all_diffs
        .iter()
        .map(|d| d.hash.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    // 5. 查询所有指定玩家的成绩
    let mut all_scores: Vec<PlayerScore> = Vec::new();
    for player_id in &player_ids {
        let player_scores = db::get_scores_by_hashes_and_player(&guard, &all_hashes, player_id)
            .map_err(|e| e.to_string())?;
        all_scores.extend(player_scores);
    }

    // 6. 构建当前版本难度集合
    let current_diff_set: std::collections::HashSet<(String, String)> = current_diffs
        .iter()
        .map(|d| (d.mode.clone(), d.difficulty.clone()))
        .collect();

    // 7. 按 (mode, difficulty) 分组成绩
    let mut scores_by_diff: std::collections::HashMap<(String, String), Vec<(String, PlayerScore)>> = std::collections::HashMap::new();
    for score in all_scores {
        let key = (score.mode.clone(), score.difficulty.clone());
        scores_by_diff.entry(key).or_default().push((score.player_id.clone(), score));
    }

    // 8. 构建成绩行（只包含有成绩的难度）和无成绩难度分组
    let mut score_rows: Vec<ScoreRowData> = Vec::new();
    let mut no_score_diffs_by_mode: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    // 8.1 首先处理所有当前版本难度
    for diff in &current_diffs {
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

        if !scores.is_empty() {
            // 有成绩：加入成绩行
            score_rows.push(ScoreRowData {
                mode: diff.mode.clone(),
                difficulty: diff.difficulty.clone(),
                is_current_version: true,
                scores,
            });
        } else {
            // 无成绩：加入无成绩分组
            no_score_diffs_by_mode
                .entry(diff.mode.clone())
                .or_default()
                .push(diff.difficulty.clone());
        }
    }

    // 8.2 然后添加有成绩但不在当前版本的难度
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

            if !scores.is_empty() {
                score_rows.push(ScoreRowData {
                    mode: key.0.clone(),
                    difficulty: key.1.clone(),
                    is_current_version: false,
                    scores,
                });
            }
        }
    }

    // 9. 按难度排序成绩行
    score_rows.sort_by(|a, b| {
        let mode_order = mode_sort_order(&a.mode).cmp(&mode_sort_order(&b.mode));
        if mode_order != std::cmp::Ordering::Equal {
            return mode_order;
        }
        difficulty_sort_order(&a.difficulty).cmp(&difficulty_sort_order(&b.difficulty))
    });

    // 10. 构建无成绩难度分组（按模式排序，每个模式内难度排序）
    let mut no_score_diffs: Vec<NoScoreDiffGroup> = no_score_diffs_by_mode
        .into_iter()
        .map(|(mode, difficulties)| {
            let mut sorted_diffs = difficulties;
            sorted_diffs.sort_by_key(|d| difficulty_sort_order(d));
            NoScoreDiffGroup {
                mode,
                difficulties: sorted_diffs,
            }
        })
        .collect();
    no_score_diffs.sort_by_key(|g| mode_sort_order(&g.mode));

    Ok(MapScoresResponse {
        map,
        current_diffs,
        scores: score_rows,
        no_score_diffs,
    })
}

/// 查询歌单某页谱面在指定玩家下的成绩
#[tauri::command]
pub fn get_playlist_scores(
    app: AppHandle,
    playlist_id: i64,
    page: i32,
    player_ids: Vec<String>,
) -> Result<PlaylistScoresResponse, String> {
    let state = app.try_state::<Mutex<rusqlite::Connection>>()
        .ok_or("Database not initialized")?;
    let guard = state.lock().map_err(|e| e.to_string())?;

    // 1. 获取歌单信息
    let playlist = db::get_playlist_by_id(&guard, playlist_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Playlist not found: {}", playlist_id))?;

    // 2. 获取分页的谱面 keys
    let (map_keys, total_pages) = db::get_playlist_maps_paginated(&guard, playlist_id, page, 100)
        .map_err(|e| e.to_string())?;

    // 3. 逐个查询每个谱面的成绩
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

        let current_diffs: Vec<CurrentDiffInfo> = all_diffs
            .iter()
            .filter(|d| d.is_current)
            .map(|d| CurrentDiffInfo {
                mode: d.mode.clone(),
                difficulty: d.difficulty.clone(),
                hash: d.hash.clone(),
            })
            .collect();

        let all_hashes: Vec<String> = all_diffs
            .iter()
            .map(|d| d.hash.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let mut all_scores: Vec<PlayerScore> = Vec::new();
        for player_id in &player_ids {
            let player_scores = db::get_scores_by_hashes_and_player(&guard, &all_hashes, player_id)
                .map_err(|e| e.to_string())?;
            all_scores.extend(player_scores);
        }

        // 构建当前版本难度集合
        let current_diff_set: std::collections::HashSet<(String, String)> = current_diffs
            .iter()
            .map(|d| (d.mode.clone(), d.difficulty.clone()))
            .collect();

        let mut scores_by_diff: std::collections::HashMap<(String, String), Vec<(String, PlayerScore)>> = std::collections::HashMap::new();
        for score in all_scores {
            let key = (score.mode.clone(), score.difficulty.clone());
            scores_by_diff.entry(key).or_default().push((score.player_id.clone(), score));
        }

        let mut score_rows: Vec<ScoreRowData> = Vec::new();
        let mut no_score_diffs_by_mode: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

        // 首先处理所有当前版本难度
        for diff in &current_diffs {
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

            if !scores.is_empty() {
                score_rows.push(ScoreRowData {
                    mode: diff.mode.clone(),
                    difficulty: diff.difficulty.clone(),
                    is_current_version: true,
                    scores,
                });
            } else {
                no_score_diffs_by_mode
                    .entry(diff.mode.clone())
                    .or_default()
                    .push(diff.difficulty.clone());
            }
        }

        // 然后添加有成绩但不在当前版本的难度
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

                if !scores.is_empty() {
                    score_rows.push(ScoreRowData {
                        mode: key.0.clone(),
                        difficulty: key.1.clone(),
                        is_current_version: false,
                        scores,
                    });
                }
            }
        }

        score_rows.sort_by(|a, b| {
            let mode_order = mode_sort_order(&a.mode).cmp(&mode_sort_order(&b.mode));
            if mode_order != std::cmp::Ordering::Equal {
                return mode_order;
            }
            difficulty_sort_order(&a.difficulty).cmp(&difficulty_sort_order(&b.difficulty))
        });

        // 构建无成绩难度分组
        let mut no_score_diffs: Vec<NoScoreDiffGroup> = no_score_diffs_by_mode
            .into_iter()
            .map(|(mode, difficulties)| {
                let mut sorted_diffs = difficulties;
                sorted_diffs.sort_by_key(|d| difficulty_sort_order(d));
                NoScoreDiffGroup {
                    mode,
                    difficulties: sorted_diffs,
                }
            })
            .collect();
        no_score_diffs.sort_by_key(|g| mode_sort_order(&g.mode));

        maps.push(MapScoresResponse {
            map,
            current_diffs,
            scores: score_rows,
            no_score_diffs,
        });
    }

    Ok(PlaylistScoresResponse {
        playlist,
        page,
        total_pages,
        maps,
    })
}

/// 获取模式排序权重
fn mode_sort_order(mode: &str) -> i32 {
    match mode {
        "Standard" => 1,
        "OneSaber" => 2,
        "NoArrows" => 3,
        "90Degree" => 4,
        "360Degree" => 5,
        "Lightshow" => 6,
        "Lawless" => 7,
        "Legacy" => 8,
        _ => 99,
    }
}

/// 获取难度排序权重
fn difficulty_sort_order(difficulty: &str) -> i32 {
    match difficulty {
        "Easy" => 1,
        "Normal" => 2,
        "Hard" => 3,
        "Expert" => 4,
        "ExpertPlus" => 5,
        _ => 99,
    }
}

/// 增量更新玩家成绩
#[tauri::command]
pub async fn update_player_scores(
    app: AppHandle,
    player_id: String,
) -> Result<i64, String> {
    let api = BeatLeaderApi::new();
    let page_size = 100;
    let mut page = 1;
    let mut total_updated = 0;
    let mut should_stop = false;

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

        // 处理每条成绩
        for score_data in response.data {
            // 转换成绩数据
            let (map_key, _, _) = db::parse_leaderboard_id(&score_data.leaderboard.id);
            let difficulty = db::difficulty_from_value(score_data.leaderboard.difficulty);
            let score = PlayerScore {
                player_id: player_id.clone(),
                score_id: score_data.score.id as i64,
                leaderboard_id: score_data.leaderboard.id.clone(),
                map_key,
                hash: score_data.leaderboard.song_hash.clone(),
                mode: score_data.leaderboard.mode_name.clone(),
                difficulty,
                accuracy: score_data.score.accuracy,
                epoch_time: score_data.score.epoch_time,
            };

            // 数据库操作：检查是否已存在
            let existing = {
                let state = app.try_state::<Mutex<rusqlite::Connection>>()
                    .ok_or("Database not initialized")?;
                let guard = state.lock().map_err(|e| e.to_string())?;
                db::get_score_by_pk(&guard, &score.player_id, &score.hash, &score.mode, &score.difficulty)
                    .map_err(|e| e.to_string())?
            };

            match existing {
                Some(existing_score) => {
                    // 如果新成绩时间 <= 本地记录，停止拉取
                    if score.epoch_time <= existing_score.epoch_time {
                        should_stop = true;
                        break;
                    }
                    // 否则更新记录
                    {
                        let state = app.try_state::<Mutex<rusqlite::Connection>>()
                            .ok_or("Database not initialized")?;
                        let guard = state.lock().map_err(|e| e.to_string())?;
                        db::insert_score(&guard, &score).map_err(|e| e.to_string())?;
                    }
                    total_updated += 1;
                }
                None => {
                    // 新记录，插入
                    {
                        let state = app.try_state::<Mutex<rusqlite::Connection>>()
                            .ok_or("Database not initialized")?;
                        let guard = state.lock().map_err(|e| e.to_string())?;
                        db::insert_score(&guard, &score).map_err(|e| e.to_string())?;
                    }
                    total_updated += 1;
                }
            }
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