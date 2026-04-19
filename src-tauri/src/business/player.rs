use crate::api::{BeatLeaderApi, PlayerResponse};
use crate::database::operations::{PlayerInfo, AddPlayerParams};
use crate::database::operations as db;
use crate::business::{BusinessError, score};
use rusqlite::Connection;

/// 添加玩家
///
/// 1. 调用 API 获取玩家信息
/// 2. 存入数据库
/// 3. 全量拉取玩家成绩
pub async fn add_player(
    conn: &Connection,
    api: &BeatLeaderApi,
    params: AddPlayerParams,
) -> Result<PlayerInfo, BusinessError> {
    // 调用 API 获取玩家信息
    let player_response = api.get_player(&params.id).await?;

    // 构建玩家信息
    let player_info = PlayerInfo {
        id: player_response.id,
        name: player_response.name,
        avatar_url: Some(player_response.avatar),
        live_url: params.live_url,
        alias: params.alias,
    };

    // 存入数据库
    db::insert_player(conn, &player_info)?;

    // 全量拉取玩家成绩
    score::fetch_all_scores(conn, api, &player_info.id).await?;

    Ok(player_info)
}

/// 获取玩家列表
pub fn get_player_list(conn: &Connection) -> Result<Vec<PlayerInfo>, BusinessError> {
    db::get_all_players(conn).map_err(BusinessError::from)
}

/// 获取单个玩家
pub fn get_player(conn: &Connection, id: &str) -> Result<Option<PlayerInfo>, BusinessError> {
    db::get_player_by_id(conn, id).map_err(BusinessError::from)
}

/// 删除玩家
pub fn delete_player(conn: &Connection, id: &str) -> Result<(), BusinessError> {
    db::delete_player(conn, id).map_err(BusinessError::from)
}

/// 更新玩家信息（手动更新 live_url 或 alias）
pub fn update_player(conn: &Connection, player: &PlayerInfo) -> Result<(), BusinessError> {
    db::update_player(conn, player).map_err(BusinessError::from)
}

/// 从 API 响应创建玩家信息（不拉取成绩）
pub async fn fetch_player_info(
    api: &BeatLeaderApi,
    player_id: &str,
) -> Result<PlayerResponse, BusinessError> {
    api.get_player(player_id).await.map_err(BusinessError::from)
}