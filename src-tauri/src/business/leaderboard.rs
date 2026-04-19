use crate::api::{BeatLeaderApi, LeaderboardResponse};
use crate::business::BusinessError;

/// 获取排行榜详情
///
/// 通过 leaderboardId 获取排行榜信息，包含各版本 hash
pub async fn get_leaderboard(
    api: &BeatLeaderApi,
    leaderboard_id: &str,
) -> Result<LeaderboardResponse, BusinessError> {
    api.get_leaderboard(leaderboard_id).await.map_err(BusinessError::from)
}

/// 解析 leaderboardId 获取谱面 key
///
/// 返回 (mapKey, difficultyValue, modeValue)
pub fn parse_leaderboard_id(leaderboard_id: &str) -> (Option<String>, i32, i32) {
    crate::database::operations::parse_leaderboard_id(leaderboard_id)
}