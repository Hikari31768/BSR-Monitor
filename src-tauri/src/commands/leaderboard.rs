use crate::api::BeatLeaderApi;
use crate::api::LeaderboardResponse;

/// 获取排行榜详情
#[tauri::command]
pub async fn get_leaderboard(leaderboard_id: String) -> Result<LeaderboardResponse, String> {
    let api = BeatLeaderApi::new().map_err(|e| e.to_string())?;
    api.get_leaderboard(&leaderboard_id)
        .await
        .map_err(|e| e.to_string())
}