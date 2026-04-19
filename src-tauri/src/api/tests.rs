use crate::api::{BeatLeaderApi, BeatSaverApi, ApiError};

#[tokio::test]
async fn test_beatleader_get_player() {
    let api = BeatLeaderApi::new();
    let player_id = "76561198159152141"; // YoLeax 的 Steam ID

    let result = api.get_player(player_id).await;
    assert!(result.is_ok(), "Failed to get player: {:?}", result.err());

    let player = result.unwrap();
    assert_eq!(player.id, player_id);
    assert!(!player.name.is_empty());
    println!("Player: {} (avatar: {})", player.name, player.avatar);
}

#[tokio::test]
async fn test_beatleader_get_player_scores() {
    let api = BeatLeaderApi::new();
    let player_id = "76561198159152141";

    let result = api.get_player_scores(player_id, 1, 10).await;
    assert!(result.is_ok(), "Failed to get scores: {:?}", result.err());

    let scores = result.unwrap();
    assert!(!scores.data.is_empty());
    println!("Got {} scores, total: {}", scores.data.len(), scores.metadata.total);

    for score in &scores.data[0..3] {
        println!(
            "  - leaderboard: {}, hash: {}, mode: {}, diff: {}, acc: {:.2}%",
            score.leaderboard.id,
            score.leaderboard.song_hash,
            score.leaderboard.mode_name,
            score.leaderboard.difficulty,
            score.score.accuracy * 100.0
        );
    }
}

#[tokio::test]
async fn test_beatsaver_get_map_by_key() {
    let api = BeatSaverApi::new();
    let key = "4c89a"; // Windows 7 Startup

    let result = api.get_map_by_key(key).await;
    assert!(result.is_ok(), "Failed to get map: {:?}", result.err());

    let map = result.unwrap();
    assert_eq!(map.id, key);
    println!("Map: {} by {}", map.metadata.song_name, map.metadata.level_author_name);
}

#[tokio::test]
async fn test_beatsaver_get_map_by_hash() {
    let api = BeatSaverApi::new();
    let hash = "f01764441f594952b39013bd774feae7533afcc8";

    let result = api.get_map_by_hash(hash).await;
    assert!(result.is_ok(), "Failed to get map by hash: {:?}", result.err());

    let map = result.unwrap();
    println!("Map by hash: {}", map.metadata.song_name);
}

#[tokio::test]
async fn test_beatsaver_get_playlist() {
    let api = BeatSaverApi::new();
    let playlist_id = 123; // 示例歌单

    let result = api.get_playlist(playlist_id, 0).await;
    // 可能不存在，只验证 API 调用本身能正常工作
    match result {
        Ok(playlist) => {
            println!("Playlist: {} ({} maps)", playlist.playlist.name, playlist.maps.len());
        }
        Err(ApiError::NotFound(_)) => {
            println!("Playlist {} not found (expected for test)", playlist_id);
        }
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}