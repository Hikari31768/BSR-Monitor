use crate::api::{BeatLeaderApi, CompactScoreResponse};
use crate::database::operations::{PlayerScore, difficulty_from_value, parse_leaderboard_id};
use crate::database::operations as db;
use crate::business::BusinessError;
use rusqlite::Connection;

/// 全量拉取玩家成绩
///
/// 从第一页开始，循环拉取所有成绩并存入数据库
pub async fn fetch_all_scores(
    conn: &Connection,
    api: &BeatLeaderApi,
    player_id: &str,
) -> Result<i64, BusinessError> {
    let page_size = 100;
    let mut page = 1;
    let mut total_inserted = 0;

    loop {
        // 按时间降序拉取
        let response = api.get_player_scores(player_id, page, page_size).await?;

        if response.data.is_empty() {
            break;
        }

        // 处理每条成绩
        for score_data in response.data {
            let score = convert_score_response(player_id, &score_data);

            // 插入成绩
            if db::insert_score(conn, &score)? {
                total_inserted += 1;
            }
        }

        // 检查是否还有更多页
        let total_pages = (response.metadata.total + response.metadata.items_per_page - 1) / response.metadata.items_per_page;
        if page >= total_pages {
            break;
        }

        page += 1;
    }

    Ok(total_inserted)
}

/// 增量更新玩家成绩
///
/// 按时间降序拉取，遇到已存在且时间不更新的成绩则停止
pub async fn incremental_update_scores(
    conn: &Connection,
    api: &BeatLeaderApi,
    player_id: &str,
) -> Result<i64, BusinessError> {
    let page_size = 100;
    let mut page = 1;
    let mut total_updated = 0;
    let mut should_stop = false;

    loop {
        if should_stop {
            break;
        }

        // 按时间降序拉取
        let response = api.get_player_scores(player_id, page, page_size).await?;

        if response.data.is_empty() {
            break;
        }

        // 处理每条成绩
        for score_data in response.data {
            let score = convert_score_response(player_id, &score_data);

            // 检查是否已存在
            let existing = db::get_score_by_pk(
                conn,
                &score.player_id,
                &score.hash,
                &score.mode,
                &score.difficulty,
            )?;

            match existing {
                Some(existing_score) => {
                    // 如果新成绩时间 <= 本地记录，停止拉取
                    if score.epoch_time <= existing_score.epoch_time {
                        should_stop = true;
                        break;
                    }
                    // 否则更新记录
                    db::insert_score(conn, &score)?;
                    total_updated += 1;
                }
                None => {
                    // 新记录，插入
                    db::insert_score(conn, &score)?;
                    total_updated += 1;
                }
            }
        }

        // 检查是否还有更多页
        let total_pages = (response.metadata.total + response.metadata.items_per_page - 1) / response.metadata.items_per_page;
        if page >= total_pages {
            break;
        }

        page += 1;
    }

    Ok(total_updated)
}

/// 获取玩家成绩列表
pub fn get_player_scores(conn: &Connection, player_id: &str) -> Result<Vec<PlayerScore>, BusinessError> {
    db::get_scores_by_player(conn, player_id).map_err(BusinessError::from)
}

/// 获取玩家成绩数量
pub fn get_player_score_count(conn: &Connection, player_id: &str) -> Result<i64, BusinessError> {
    db::get_score_count_by_player(conn, player_id).map_err(BusinessError::from)
}

/// 将 API 响应转换为数据库结构
fn convert_score_response(player_id: &str, response: &CompactScoreResponse) -> PlayerScore {
    let (map_key, _difficulty_value, _mode_value) = parse_leaderboard_id(&response.leaderboard.id);
    let difficulty = difficulty_from_value(response.leaderboard.difficulty);

    PlayerScore {
        player_id: player_id.to_string(),
        score_id: response.score.id as i64,
        leaderboard_id: response.leaderboard.id.clone(),
        map_key,
        hash: response.leaderboard.song_hash.clone(),
        mode: response.leaderboard.mode_name.clone(),
        difficulty,
        accuracy: response.score.accuracy,
        epoch_time: response.score.epoch_time,
    }
}