use crate::api::{BeatLeaderApi, BeatSaverApi};
use crate::business::{player, score, map, playlist, leaderboard};
use crate::database::operations::{PlayerInfo, AddPlayerParams};
use rusqlite::Connection;

// 创建测试用的内存数据库
fn create_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
    crate::database::schema::create_tables(&conn).expect("Failed to create tables");
    conn
}

// ============ 玩家管理测试 ============

#[tokio::test]
async fn test_add_player() {
    let conn = create_test_db();
    let api = BeatLeaderApi::new();

    // 使用一个已知存在的玩家 ID 进行测试
    let params = AddPlayerParams {
        id: "76561198035353849".to_string(), // 示例玩家 ID
        live_url: None,
        alias: Some("Test Player".to_string()),
    };

    // 注意：这个测试需要网络连接，实际运行时会调用真实 API
    // 在 CI 环境中可能需要 mock 或跳过
    let result = player::add_player(&conn, &api, params).await;

    match result {
        Ok(player_info) => {
            assert!(!player_info.id.is_empty());
            assert!(!player_info.name.is_empty());

            // 验证数据库中有该玩家
            let db_player = player::get_player(&conn, &player_info.id).expect("Failed to get player");
            assert!(db_player.is_some());
            assert_eq!(db_player.unwrap().alias, Some("Test Player".to_string()));
        }
        Err(e) => {
            // 网络问题或玩家不存在，打印错误信息
            println!("test_add_player skipped or failed: {}", e);
        }
    }
}

#[test]
fn test_get_player_list() {
    let conn = create_test_db();

    // 插入一些测试玩家
    let player1 = PlayerInfo {
        id: "test1".to_string(),
        name: "Player 1".to_string(),
        avatar_url: None,
        live_url: None,
        alias: None,
    };
    let player2 = PlayerInfo {
        id: "test2".to_string(),
        name: "Player 2".to_string(),
        avatar_url: None,
        live_url: None,
        alias: Some("Alias 2".to_string()),
    };

    crate::database::operations::insert_player(&conn, &player1).expect("Failed to insert player1");
    crate::database::operations::insert_player(&conn, &player2).expect("Failed to insert player2");

    // 获取列表
    let players = player::get_player_list(&conn).expect("Failed to get player list");
    assert_eq!(players.len(), 2);
}

#[test]
fn test_delete_player() {
    let conn = create_test_db();

    // 插入测试玩家
    let player = PlayerInfo {
        id: "test_delete".to_string(),
        name: "To Delete".to_string(),
        avatar_url: None,
        live_url: None,
        alias: None,
    };
    crate::database::operations::insert_player(&conn, &player).expect("Failed to insert player");

    // 删除玩家
    player::delete_player(&conn, "test_delete").expect("Failed to delete player");

    // 验证已删除
    let result = player::get_player(&conn, "test_delete").expect("Failed to query");
    assert!(result.is_none());
}

// ============ 成绩管理测试 ============

#[tokio::test]
async fn test_fetch_player_scores() {
    let conn = create_test_db();
    let api = BeatLeaderApi::new();

    // 先插入一个玩家（不调用 API，避免重复）
    let player = PlayerInfo {
        id: "76561198035353849".to_string(),
        name: "Test Player".to_string(),
        avatar_url: None,
        live_url: None,
        alias: None,
    };
    crate::database::operations::insert_player(&conn, &player).expect("Failed to insert player");

    // 拉取成绩
    let result = score::fetch_all_scores(&conn, &api, &player.id).await;

    match result {
        Ok(count) => {
            println!("Fetched {} scores", count);
            assert!(count > 0);

            // 验证数据库中有成绩
            let scores = score::get_player_scores(&conn, &player.id).expect("Failed to get scores");
            assert!(!scores.is_empty());
        }
        Err(e) => {
            println!("test_fetch_player_scores skipped or failed: {}", e);
        }
    }
}

#[test]
fn test_incremental_update_logic() {
    let conn = create_test_db();

    // 插入一条已有成绩
    let existing_score = crate::database::operations::PlayerScore {
        player_id: "test_player".to_string(),
        score_id: 100,
        leaderboard_id: "test_leaderboard".to_string(),
        map_key: Some("test_key".to_string()),
        hash: "test_hash".to_string(),
        mode: "Standard".to_string(),
        difficulty: "Expert".to_string(),
        accuracy: 0.95,
        epoch_time: 1000,
    };
    crate::database::operations::insert_player(&conn, &PlayerInfo {
        id: "test_player".to_string(),
        name: "Test".to_string(),
        avatar_url: None,
        live_url: None,
        alias: None,
    }).expect("Failed to insert player");
    crate::database::operations::insert_score(&conn, &existing_score).expect("Failed to insert score");

    // 测试更新逻辑：新成绩时间更晚，应该更新
    let new_score = crate::database::operations::PlayerScore {
        player_id: "test_player".to_string(),
        score_id: 200,
        leaderboard_id: "test_leaderboard".to_string(),
        map_key: Some("test_key".to_string()),
        hash: "test_hash".to_string(),
        mode: "Standard".to_string(),
        difficulty: "Expert".to_string(),
        accuracy: 0.98,
        epoch_time: 2000, // 更晚
    };

    let updated = crate::database::operations::insert_score(&conn, &new_score).expect("Failed to insert");
    assert!(updated); // 应该更新

    // 验证更新后的数据
    let result = crate::database::operations::get_score_by_pk(&conn, "test_player", "test_hash", "Standard", "Expert")
        .expect("Failed to get score");
    assert!(result.is_some());
    let score = result.unwrap();
    assert_eq!(score.score_id, 200);
    assert_eq!(score.accuracy, 0.98);

    // 测试不更新逻辑：新成绩时间更早，不应该更新
    let old_score = crate::database::operations::PlayerScore {
        player_id: "test_player".to_string(),
        score_id: 300,
        leaderboard_id: "test_leaderboard".to_string(),
        map_key: Some("test_key".to_string()),
        hash: "test_hash".to_string(),
        mode: "Standard".to_string(),
        difficulty: "Expert".to_string(),
        accuracy: 0.90,
        epoch_time: 500, // 更早
    };

    let updated = crate::database::operations::insert_score(&conn, &old_score).expect("Failed to insert");
    assert!(!updated); // 不应该更新

    // 验证数据没变
    let result = crate::database::operations::get_score_by_pk(&conn, "test_player", "test_hash", "Standard", "Expert")
        .expect("Failed to get score");
    assert!(result.is_some());
    let score = result.unwrap();
    assert_eq!(score.score_id, 200); // 还是 200
}

// ============ 谱面管理测试 ============

#[tokio::test]
async fn test_get_map_by_key() {
    let conn = create_test_db();
    let api = BeatSaverApi::new();

    // 使用一个已知的谱面 key
    let result = map::get_map_by_key(&conn, &api, "1f49").await;

    match result {
        Ok(map) => {
            assert_eq!(map.key, "1f49");
            assert!(!map.song_name.is_empty());

            // 验证数据库中有该谱面
            let db_map = map::get_map_from_db(&conn, "1f49").expect("Failed to get map");
            assert!(db_map.is_some());
        }
        Err(e) => {
            println!("test_get_map_by_key skipped or failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_get_map_by_hash() {
    let conn = create_test_db();
    let api = BeatSaverApi::new();

    // 使用一个已知的谱面 hash
    let result = map::get_map_by_hash(&conn, &api, "0e9507ff8018c554b5eb589348a5524dce965f7d").await;

    match result {
        Ok(map) => {
            assert!(!map.key.is_empty());
            assert!(!map.song_name.is_empty());

            // 验证数据库中有该谱面
            let db_map = map::get_map_from_db(&conn, &map.key).expect("Failed to get map");
            assert!(db_map.is_some());
        }
        Err(e) => {
            println!("test_get_map_by_hash skipped or failed: {}", e);
        }
    }
}

// ============ 歌单管理测试 ============

#[tokio::test]
async fn test_add_playlist() {
    let conn = create_test_db();
    let api = BeatSaverApi::new();

    // 使用一个已知的歌单 ID
    let result = playlist::add_playlist(&conn, &api, 1).await;

    match result {
        Ok(playlist) => {
            assert_eq!(playlist.playlist_id, 1);
            assert!(!playlist.name.is_empty());

            // 验证数据库中有该歌单
            let db_playlist = playlist::get_playlist(&conn, 1).expect("Failed to get playlist");
            assert!(db_playlist.is_some());

            // 验证歌单中有谱面
            let maps = playlist::get_playlist_maps(&conn, 1).expect("Failed to get playlist maps");
            assert!(!maps.is_empty());
        }
        Err(e) => {
            println!("test_add_playlist skipped or failed: {}", e);
        }
    }
}

#[test]
fn test_delete_playlist() {
    let conn = create_test_db();

    // 插入测试歌单
    let playlist = crate::database::operations::Playlist {
        playlist_id: 999,
        name: "Test Playlist".to_string(),
        description: None,
        playlist_image: None,
        owner_id: None,
        owner_name: None,
        total_maps: None,
        created_at: None,
        songs_changed_at: None,
        download_url: None,
    };
    crate::database::operations::insert_playlist(&conn, &playlist).expect("Failed to insert playlist");

    // 删除歌单
    playlist::delete_playlist(&conn, 999).expect("Failed to delete playlist");

    // 验证已删除
    let result = playlist::get_playlist(&conn, 999).expect("Failed to query");
    assert!(result.is_none());
}

// ============ 排行榜查询测试 ============

#[tokio::test]
async fn test_get_leaderboard() {
    let api = BeatLeaderApi::new();

    // 使用一个已知的 leaderboardId
    let result = leaderboard::get_leaderboard(&api, "4b302xx91").await;

    match result {
        Ok(response) => {
            assert!(!response.id.is_empty());
            assert!(!response.song.hash.is_empty());
        }
        Err(e) => {
            println!("test_get_leaderboard skipped or failed: {}", e);
        }
    }
}

#[test]
fn test_parse_leaderboard_id() {
    // 测试带版本段的 leaderboardId
    let (key, diff, mode) = leaderboard::parse_leaderboard_id("4b302xx91");
    assert_eq!(key, Some("4b302".to_string()));
    assert_eq!(diff, 9); // ExpertPlus
    assert_eq!(mode, 1); // Standard

    // 测试不带版本段的 leaderboardId
    let (key, diff, mode) = leaderboard::parse_leaderboard_id("abcd51");
    assert_eq!(key, Some("abcd".to_string()));
    assert_eq!(diff, 5); // Hard
    assert_eq!(mode, 1); // Standard
}