use tauri::AppHandle;
use tauri::Manager;
use crate::api::BeatLeaderApi;
use crate::database::operations::PlayerScore;
use crate::database::operations as db;
use std::sync::Mutex;

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