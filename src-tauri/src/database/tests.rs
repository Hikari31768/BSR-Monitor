use super::{operations::*, schema, score_matching::build_map_scores_response};
use rusqlite::Connection;

fn database() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
    assert!(!schema::create_tables(&conn).unwrap());
    assert!(!schema::create_tables(&conn).unwrap());
    for id in ["p1", "p2"] {
        conn.execute("INSERT INTO PlayerInfo (id, name) VALUES (?1, ?1)", [id]).unwrap();
    }
    conn
}

fn score(player: &str, hash: &str, mode: &str, time: i64, accuracy: f32) -> PlayerScore {
    PlayerScore { player_id: player.into(), score_id: time, map_key: Some("abc".into()), hash: hash.into(),
        mode: mode.into(), difficulty: "Expert".into(), accuracy, epoch_time: time }
}

fn diff(hash: &str, mode: &str) -> MapDiff {
    MapDiff { key: "abc".into(), hash: hash.into(), is_current: true, mode: mode.into(),
        difficulty: "Expert".into(), stars_rating: Some(5.0), stars_fetched_at: Some(100),
        pass_rating: Some(2.0), acc_rating: None, tech_rating: None }
}

#[test]
fn page_upsert_shares_difficulties_and_only_accepts_newer_scores() {
    let db = database();
    let scores = [score("p1", "ABC", "Standard", 10, 0.9), score("p2", "abc", "Standard", 20, 0.8)];
    assert_eq!(insert_scores_batch(&db, &scores).unwrap(), 2);
    let count: i64 = db.query_row("SELECT COUNT(*) FROM MapDiff", [], |r| r.get(0)).unwrap();
    assert_eq!(count, 1);
    assert_eq!(insert_scores_batch(&db, &scores).unwrap(), 0);
    assert_eq!(insert_scores_batch(&db, &[score("p1", "abc", "Standard", 9, 0.99)]).unwrap(), 0);
    assert_eq!(insert_scores_batch(&db, &[score("p1", "abc", "Standard", 30, 0.7)]).unwrap(), 1);
    assert_eq!(get_latest_score_time(&db, "p1").unwrap(), Some(30));
    assert_eq!(get_scores_by_player(&db, "p1").unwrap()[0].accuracy, 0.7);
    assert_eq!(get_scores_by_player(&db, "p1").unwrap()[0].score_id, 30);
    assert_eq!(get_scores_for_maps(&db, &["abc".into()], &["p1".into(), "p2".into()]).unwrap().len(), 2);
    assert!(get_scores_for_maps(&db, &[], &["p1".into()]).unwrap().is_empty());
    assert!(get_scores_for_maps(&db, &["abc".into()], &[]).unwrap().is_empty());
}

#[test]
fn failed_page_rolls_back_scores_and_new_difficulties() {
    let db = database();
    assert!(insert_scores_batch(&db, &[
        score("p1", "first", "Standard", 10, 0.9),
        score("missing-player", "second", "Standard", 20, 0.9),
    ]).is_err());
    assert_eq!(get_score_count_by_player(&db, "p1").unwrap(), 0);
    let count: i64 = db.query_row("SELECT COUNT(*) FROM MapDiff", [], |r| r.get(0)).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn metadata_enrichment_keeps_score_links_and_version_history() {
    let db = database();
    insert_scores_batch(&db, &[score("p1", "old", "Standard", 10, 0.9)]).unwrap();
    assert!(get_existing_hashes(&db, &["old".into()]).unwrap().is_empty());
    insert_map_diffs_batch(&db, "abc", &[diff("OLD", "Standard")]).unwrap();
    assert_eq!(get_existing_hashes(&db, &["OLD".into()]).unwrap(), vec!["old"]);
    insert_map_diffs_batch(&db, "abc", &[diff("new", "Standard")]).unwrap();
    let diffs = get_map_diffs_by_key(&db, "abc").unwrap();
    assert!(diffs.iter().any(|d| d.hash == "old" && !d.is_current && d.stars_rating == Some(5.0)));
    assert_eq!(get_scores_by_player(&db, "p1").unwrap()[0].hash, "old");
    insert_scores_batch(&db, &[score("p2", "new", "Lawless", 20, 0.8)]).unwrap();
    update_diff_ratings(&db, "abc", "NEW", "Lawless", "Expert", Some(6.0), None, None, None).unwrap();
    assert!(get_map_diffs_by_key(&db, "abc").unwrap().iter().any(|d| d.mode == "Lawless" && d.is_current));
}

#[test]
fn matching_prefers_current_version_and_separates_modes() {
    let map = Map { key: "abc".into(), song_name: "Song".into(), song_author_name: "Author".into(),
        level_author_name: "Mapper".into(), rating: None, cover_url: None, preview_url: None };
    let scores = [
        score("p1", "old", "Standard", 1, 0.99),
        score("p1", "new", "Standard", 2, 0.8),
        score("p2", "old", "Standard", 1, 0.9),
        score("p1", "new", "Lawless", 1, 0.7),
    ];
    let response = build_map_scores_response(map, &[diff("new", "Standard"), diff("new", "OneSaber")], &scores);
    let standard = response.scores.iter().find(|r| r.mode == "Standard").unwrap();
    assert_eq!(standard.scores[0].accuracy, 0.8);
    assert!(!standard.scores[0].is_expired);
    assert!(standard.scores[1].is_expired);
    assert!(response.scores.iter().find(|r| r.mode == "Lawless").unwrap().scores[0].is_expired);
    assert!(response.scores.iter().find(|r| r.mode == "OneSaber").unwrap().scores.is_empty());
}

#[test]
fn legacy_migration_deduplicates_hash_case_without_losing_newest_score() {
    let db = Connection::open_in_memory().unwrap();
    db.execute_batch("PRAGMA foreign_keys=ON;
        CREATE TABLE PlayerInfo (id TEXT PRIMARY KEY, name TEXT, avatarURL TEXT, liveURL TEXT, alias TEXT);
        INSERT INTO PlayerInfo(id) VALUES ('p1');
        CREATE TABLE MapDiff (key TEXT, hash TEXT, isCurrent INTEGER, mode TEXT, difficulty TEXT);
        INSERT INTO MapDiff VALUES ('abc', 'HASH', 1, 'Standard', 'Expert');
        CREATE TABLE PlayerScore (playerId TEXT, scoreId INTEGER, leaderboardId TEXT, mapKey TEXT,
            hash TEXT, mode TEXT, difficulty TEXT, accuracy REAL, epochTime INTEGER);
        INSERT INTO PlayerScore VALUES ('p1',1,'abc71','abc','HASH','Standard','Expert',0.99,1);
        INSERT INTO PlayerScore VALUES ('p1',2,'abc71','abc','hash','Standard','Expert',0.8,2);
    ").unwrap();
    assert!(schema::create_tables(&db).unwrap());
    assert!(!schema::create_tables(&db).unwrap());
    let scores = get_scores_by_player(&db, "p1").unwrap();
    assert_eq!(scores.len(), 1);
    assert_eq!(scores[0].epoch_time, 2);
    assert_eq!(scores[0].accuracy, 0.8);
    assert_eq!(scores[0].hash, "hash");
    assert_eq!(scores[0].score_id, 2);
    assert_eq!(get_score_sync_time(&db, "p1").unwrap(), Some(2));
    let violations: i64 = db.query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| r.get(0)).unwrap();
    assert_eq!(violations, 0);
}

#[test]
fn failed_legacy_migration_preserves_original_tables() {
    let db = Connection::open_in_memory().unwrap();
    db.execute_batch("PRAGMA foreign_keys=ON;
        CREATE TABLE MapDiff (key TEXT, hash TEXT, isCurrent INTEGER, mode TEXT, difficulty TEXT);
        CREATE TABLE PlayerScore (playerId TEXT, scoreId INTEGER, leaderboardId TEXT, mapKey TEXT,
            hash TEXT, mode TEXT, difficulty TEXT, accuracy REAL, epochTime INTEGER);
        INSERT INTO PlayerScore VALUES ('missing-player',1,'abc71','abc','hash','Standard','Expert',0.9,1);
    ").unwrap();
    assert!(schema::create_tables(&db).is_err());
    assert!(db.is_autocommit());
    assert!(schema::needs_score_migration(&db).unwrap());
    let count: i64 = db.query_row("SELECT COUNT(*) FROM PlayerScore WHERE scoreId=1", [], |r| r.get(0)).unwrap();
    assert_eq!(count, 1);
    let partial_tables: i64 = db.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE name IN ('PlayerScore_new','MapDiff_new')",
        [], |r| r.get(0),
    ).unwrap();
    assert_eq!(partial_tables, 0);
}

#[test]
fn successful_sync_time_persists_and_is_not_overwritten_by_profile_updates() {
    let db = database();
    assert_eq!(get_player_by_id(&db, "p1").unwrap().unwrap().updated_at, None);
    let timestamp = mark_player_updated(&db, "p1").unwrap();
    assert!(timestamp > 0);
    let mut player = get_player_by_id(&db, "p1").unwrap().unwrap();
    assert_eq!(player.updated_at, Some(timestamp));
    player.name = "Renamed".into();
    player.updated_at = None;
    insert_player(&db, &player).unwrap();
    update_player(&db, &player).unwrap();
    assert!(!schema::create_tables(&db).unwrap());
    assert_eq!(get_player_by_id(&db, "p1").unwrap().unwrap().updated_at, Some(timestamp));
    assert_eq!(get_all_players(&db).unwrap().iter().find(|p| p.id == "p1").unwrap().updated_at, Some(timestamp));
    let list = get_players_with_score_counts(&db).unwrap();
    let (player, count) = list.iter().find(|(p, _)| p.id == "p1").unwrap();
    assert_eq!(player.updated_at, Some(timestamp));
    assert_eq!(*count, 0);
    assert!(mark_player_updated(&db, "deleted-player").is_err());
}

#[test]
fn incomplete_sync_does_not_advance_retry_checkpoint() {
    let db = database();
    insert_scores_batch(&db, &[score("p1", "first-page", "Standard", 100, 0.9)]).unwrap();
    // The initial import failed after its newest page. A retry must still fetch all pages.
    assert_eq!(get_score_sync_time(&db, "p1").unwrap(), None);
    insert_scores_batch(&db, &[score("p1", "last-page", "Standard", 10, 0.8)]).unwrap();
    mark_player_updated(&db, "p1").unwrap();
    assert_eq!(get_score_sync_time(&db, "p1").unwrap(), Some(100));
    // A later refresh saves newer pages out of order, then fails before finishing.
    insert_scores_batch(&db, &[score("p1", "new-first-page", "Standard", 300, 0.95)]).unwrap();
    assert_eq!(get_latest_score_time(&db, "p1").unwrap(), Some(300));
    assert_eq!(get_score_sync_time(&db, "p1").unwrap(), Some(100));
    assert!(!schema::create_tables(&db).unwrap());
    assert_eq!(get_score_sync_time(&db, "p1").unwrap(), Some(100));
    insert_scores_batch(&db, &[score("p1", "missing-page", "Standard", 200, 0.85)]).unwrap();
    mark_player_updated(&db, "p1").unwrap();
    assert_eq!(get_score_sync_time(&db, "p1").unwrap(), Some(300));
    assert_eq!(get_score_count_by_player(&db, "p1").unwrap(), 4);
}

#[test]
fn replay_id_follows_best_current_expired_and_removed_difficulty_scores() {
    let map = Map { key: "abc".into(), song_name: "Song".into(), song_author_name: "Author".into(),
        level_author_name: "Mapper".into(), rating: None, cover_url: None, preview_url: None };
    let mut removed_a = score("removed", "old-a", "Standard", 5, 0.8);
    removed_a.difficulty = "Hard".into();
    let mut removed_b = score("removed", "old-b", "Standard", 6, 0.96);
    removed_b.difficulty = "Hard".into();
    let response = build_map_scores_response(map, &[diff("current", "Standard")], &[
        score("current", "current", "Standard", 1, 0.8),
        score("current", "current", "Standard", 2, 0.9),
        score("current", "old-a", "Standard", 7, 0.99),
        score("expired", "old-a", "Standard", 3, 0.8),
        score("expired", "old-b", "Standard", 4, 0.95),
        removed_a, removed_b,
    ]);
    for (player, expected_id, expected_accuracy, expired) in [
        ("current", 2, 0.9, false), ("expired", 4, 0.95, true), ("removed", 6, 0.96, true),
    ] {
        let cell = response.scores.iter().flat_map(|row| &row.scores).find(|cell| cell.player_id == player).unwrap();
        assert_eq!(cell.score_id, expected_id);
        assert_eq!(cell.accuracy, expected_accuracy);
        assert_eq!(cell.is_expired, expired);
    }
}

#[test]
fn earlier_normalized_database_upgrades_without_repeating_legacy_migration() {
    let db = database();
    insert_scores_batch(&db, &[score("p1", "hash", "Standard", 10, 0.9)]).unwrap();
    db.execute_batch("ALTER TABLE PlayerScore DROP COLUMN scoreId;
        ALTER TABLE PlayerInfo DROP COLUMN scoreSyncEpoch; PRAGMA user_version = 1;").unwrap();
    assert!(!schema::needs_score_migration(&db).unwrap());
    assert!(!schema::create_tables(&db).unwrap());
    assert_eq!(get_scores_by_player(&db, "p1").unwrap()[0].score_id, 0);
    assert_eq!(get_score_sync_time(&db, "p1").unwrap(), None);
    // Refreshing an equal-timestamp score can restore its previously discarded ID.
    assert_eq!(insert_scores_batch(&db, &[score("p1", "hash", "Standard", 10, 0.9)]).unwrap(), 1);
    assert_eq!(get_scores_by_player(&db, "p1").unwrap()[0].score_id, 10);
    mark_player_updated(&db, "p1").unwrap();
    assert_eq!(get_score_sync_time(&db, "p1").unwrap(), Some(10));
    assert!(!schema::create_tables(&db).unwrap());
}
