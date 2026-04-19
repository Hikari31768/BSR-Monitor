use rusqlite::Connection;
use serde::{Deserialize, Serialize};

// Map 表数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Map {
    pub key: String,
    pub song_name: String,
    pub song_author_name: String,
    pub level_author_name: String,
    pub rating: Option<f32>,
    pub cover_url: Option<String>,
    pub preview_url: Option<String>,
}

// MapDiff 表数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MapDiff {
    pub key: String,
    pub hash: String,
    pub is_current: bool,
    pub mode: String,
    pub difficulty: String,
}

// PlayerInfo 表数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerInfo {
    pub id: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub live_url: Option<String>,
    pub alias: Option<String>,
}

// 用于添加玩家时的输入参数
#[derive(Debug, Default)]
pub struct AddPlayerParams {
    pub id: String,
    pub live_url: Option<String>,
    pub alias: Option<String>,
}

// PlayerScore 表数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerScore {
    pub player_id: String,
    pub score_id: i64,
    pub leaderboard_id: String,
    pub map_key: Option<String>,
    pub hash: String,
    pub mode: String,
    pub difficulty: String,
    pub accuracy: f32,
    pub epoch_time: i64,
}

// Playlist 表数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Playlist {
    pub playlist_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub playlist_image: Option<String>,
    pub owner_id: Option<i64>,
    pub owner_name: Option<String>,
    pub total_maps: Option<i32>,
    pub created_at: Option<String>,
    pub songs_changed_at: Option<String>,
    pub download_url: Option<String>,
}

// PlaylistMap 表数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaylistMap {
    pub playlist_id: i64,
    pub map_key: String,
    pub map_order: f32,
    pub map: Option<Map>,
}

// 难度值转换
pub fn difficulty_from_value(value: i32) -> String {
    match value {
        1 => "Easy",
        3 => "Normal",
        5 => "Hard",
        7 => "Expert",
        9 => "ExpertPlus",
        _ => "Unknown",
    }.to_string()
}

pub fn difficulty_to_value(name: &str) -> i32 {
    match name {
        "Easy" => 1,
        "Normal" => 3,
        "Hard" => 5,
        "Expert" => 7,
        "ExpertPlus" => 9,
        _ => 0,
    }
}

// leaderboardId 解析
pub fn parse_leaderboard_id(leaderboard_id: &str) -> (Option<String>, i32, i32) {
    let x_count = leaderboard_id.matches('x').count();

    if x_count > 0 {
        let parts: Vec<&str> = leaderboard_id.split('x').collect();
        if parts.len() >= 2 {
            let key = parts[0].to_string();
            let suffix = parts.last().copied().unwrap_or("");
            if suffix.len() >= 2 {
                let difficulty = suffix[0..1].parse::<i32>().unwrap_or(0);
                let mode = suffix[1..].parse::<i32>().unwrap_or(0);
                return (Some(key), difficulty, mode);
            }
        }
        return (None, 0, 0);
    } else {
        if leaderboard_id.len() >= 2 {
            let mode_start = leaderboard_id.len() - 1;
            let mode = leaderboard_id[mode_start..].parse::<i32>().unwrap_or(0);
            let difficulty = leaderboard_id[mode_start - 1..mode_start].parse::<i32>().unwrap_or(0);
            let key = &leaderboard_id[0..mode_start - 1];
            return (Some(key.to_string()), difficulty, mode);
        }
        return (None, 0, 0);
    }
}

// ============ Map CRUD ============

pub fn insert_map(conn: &Connection, map: &Map) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO Map (key, songName, songAuthorName, levelAuthorName, rating, coverURL, previewURL)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            map.key,
            map.song_name,
            map.song_author_name,
            map.level_author_name,
            map.rating,
            map.cover_url,
            map.preview_url
        ],
    )?;
    Ok(())
}

pub fn get_map_by_key(conn: &Connection, key: &str) -> Result<Option<Map>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT key, songName, songAuthorName, levelAuthorName, rating, coverURL, previewURL FROM Map WHERE key = ?1"
    )?;

    let result = stmt.query_row([key], |row| {
        Ok(Map {
            key: row.get(0)?,
            song_name: row.get(1)?,
            song_author_name: row.get(2)?,
            level_author_name: row.get(3)?,
            rating: row.get(4)?,
            cover_url: row.get(5)?,
            preview_url: row.get(6)?,
        })
    });

    match result {
        Ok(map) => Ok(Some(map)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

/// 插入单个难度记录（不重置 isCurrent，用于增量更新）
pub fn insert_map_diff_single(conn: &Connection, diff: &MapDiff) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO MapDiff (key, hash, isCurrent, mode, difficulty)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            diff.key,
            diff.hash,
            diff.is_current,
            diff.mode,
            diff.difficulty
        ],
    )?;
    Ok(())
}

/// 批量插入同一版本的多个难度（先重置旧版本，再插入新版本）
pub fn insert_map_diffs_batch(conn: &Connection, key: &str, diffs: &[MapDiff]) -> Result<(), rusqlite::Error> {
    if diffs.is_empty() {
        return Ok(());
    }

    // 先将该 key 下所有记录的 isCurrent 改为 false（标记为旧版本）
    conn.execute(
        "UPDATE MapDiff SET isCurrent = 0 WHERE key = ?1",
        [key],
    )?;

    // 批量插入新记录（都标记为当前版本）
    for diff in diffs {
        conn.execute(
            "INSERT OR REPLACE INTO MapDiff (key, hash, isCurrent, mode, difficulty)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                diff.key,
                diff.hash,
                true, // 新版本都是当前版本
                diff.mode,
                diff.difficulty
            ],
        )?;
    }
    Ok(())
}

pub fn get_map_diffs_by_key(conn: &Connection, key: &str) -> Result<Vec<MapDiff>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT key, hash, isCurrent, mode, difficulty FROM MapDiff WHERE key = ?1"
    )?;

    let diffs = stmt.query_map([key], |row| {
        Ok(MapDiff {
            key: row.get(0)?,
            hash: row.get(1)?,
            is_current: row.get(2)?,
            mode: row.get(3)?,
            difficulty: row.get(4)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(diffs)
}

// ============ PlayerInfo CRUD ============

pub fn insert_player(conn: &Connection, player: &PlayerInfo) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO PlayerInfo (id, name, avatarURL, liveURL, alias)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            player.id,
            player.name,
            player.avatar_url,
            player.live_url,
            player.alias
        ],
    )?;
    Ok(())
}

pub fn get_player_by_id(conn: &Connection, id: &str) -> Result<Option<PlayerInfo>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, avatarURL, liveURL, alias FROM PlayerInfo WHERE id = ?1"
    )?;

    let result = stmt.query_row([id], |row| {
        Ok(PlayerInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            avatar_url: row.get(2)?,
            live_url: row.get(3)?,
            alias: row.get(4)?,
        })
    });

    match result {
        Ok(player) => Ok(Some(player)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn get_all_players(conn: &Connection) -> Result<Vec<PlayerInfo>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, avatarURL, liveURL, alias FROM PlayerInfo"
    )?;

    let players = stmt.query_map([], |row| {
        Ok(PlayerInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            avatar_url: row.get(2)?,
            live_url: row.get(3)?,
            alias: row.get(4)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(players)
}

pub fn delete_player(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
    // 先删除该玩家的所有成绩
    conn.execute(
        "DELETE FROM PlayerScore WHERE playerId = ?1",
        [id],
    )?;

    // 再删除玩家信息
    conn.execute(
        "DELETE FROM PlayerInfo WHERE id = ?1",
        [id],
    )?;
    Ok(())
}

pub fn update_player(conn: &Connection, player: &PlayerInfo) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE PlayerInfo SET name = ?2, avatarURL = ?3, liveURL = ?4, alias = ?5 WHERE id = ?1",
        rusqlite::params![
            player.id,
            player.name,
            player.avatar_url,
            player.live_url,
            player.alias
        ],
    )?;
    Ok(())
}

// ============ PlayerScore CRUD ============

pub fn insert_score(conn: &Connection, score: &PlayerScore) -> Result<bool, rusqlite::Error> {
    // 检查是否已存在
    let existing = get_score_by_pk(conn, &score.player_id, &score.hash, &score.mode, &score.difficulty)?;

    match existing {
        Some(existing_score) => {
            // 如果新成绩时间更晚，更新记录
            if score.epoch_time > existing_score.epoch_time {
                conn.execute(
                    "UPDATE PlayerScore SET scoreId = ?2, leaderboardId = ?3, mapKey = ?4, accuracy = ?5, epochTime = ?6
                     WHERE playerId = ?1 AND hash = ?7 AND mode = ?8 AND difficulty = ?9",
                    rusqlite::params![
                        score.player_id,
                        score.score_id,
                        score.leaderboard_id,
                        score.map_key,
                        score.accuracy,
                        score.epoch_time,
                        score.hash,
                        score.mode,
                        score.difficulty
                    ],
                )?;
                Ok(true) // 更新成功
            } else {
                Ok(false) // 不需要更新
            }
        }
        None => {
            // 插入新记录
            conn.execute(
                "INSERT INTO PlayerScore (playerId, scoreId, leaderboardId, mapKey, hash, mode, difficulty, accuracy, epochTime)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    score.player_id,
                    score.score_id,
                    score.leaderboard_id,
                    score.map_key,
                    score.hash,
                    score.mode,
                    score.difficulty,
                    score.accuracy,
                    score.epoch_time
                ],
            )?;
            Ok(true) // 插入成功
        }
    }
}

pub fn get_score_by_pk(conn: &Connection, player_id: &str, hash: &str, mode: &str, difficulty: &str) -> Result<Option<PlayerScore>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT playerId, scoreId, leaderboardId, mapKey, hash, mode, difficulty, accuracy, epochTime
         FROM PlayerScore WHERE playerId = ?1 AND hash = ?2 AND mode = ?3 AND difficulty = ?4"
    )?;

    let result = stmt.query_row(rusqlite::params![player_id, hash, mode, difficulty], |row| {
        Ok(PlayerScore {
            player_id: row.get(0)?,
            score_id: row.get(1)?,
            leaderboard_id: row.get(2)?,
            map_key: row.get(3)?,
            hash: row.get(4)?,
            mode: row.get(5)?,
            difficulty: row.get(6)?,
            accuracy: row.get(7)?,
            epoch_time: row.get(8)?,
        })
    });

    match result {
        Ok(score) => Ok(Some(score)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn get_scores_by_player(conn: &Connection, player_id: &str) -> Result<Vec<PlayerScore>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT playerId, scoreId, leaderboardId, mapKey, hash, mode, difficulty, accuracy, epochTime
         FROM PlayerScore WHERE playerId = ?1 ORDER BY epochTime DESC"
    )?;

    let scores = stmt.query_map([player_id], |row| {
        Ok(PlayerScore {
            player_id: row.get(0)?,
            score_id: row.get(1)?,
            leaderboard_id: row.get(2)?,
            map_key: row.get(3)?,
            hash: row.get(4)?,
            mode: row.get(5)?,
            difficulty: row.get(6)?,
            accuracy: row.get(7)?,
            epoch_time: row.get(8)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(scores)
}

pub fn get_score_count_by_player(conn: &Connection, player_id: &str) -> Result<i64, rusqlite::Error> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM PlayerScore WHERE playerId = ?1",
        [player_id],
        |row| row.get(0),
    )?;
    Ok(count)
}

/// 获取玩家最新成绩的 epoch_time
pub fn get_latest_score_time(conn: &Connection, player_id: &str) -> Result<Option<i64>, rusqlite::Error> {
    let result: Option<i64> = conn.query_row(
        "SELECT MAX(epochTime) FROM PlayerScore WHERE playerId = ?1",
        [player_id],
        |row| row.get(0),
    )?;
    Ok(result)
}

// ============ Playlist CRUD ============

pub fn insert_playlist(conn: &Connection, playlist: &Playlist) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO Playlist (playlistId, name, description, playlistImage, ownerId, ownerName, totalMaps, createdAt, songsChangedAt, downloadURL)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            playlist.playlist_id,
            playlist.name,
            playlist.description,
            playlist.playlist_image,
            playlist.owner_id,
            playlist.owner_name,
            playlist.total_maps,
            playlist.created_at,
            playlist.songs_changed_at,
            playlist.download_url
        ],
    )?;
    Ok(())
}

pub fn get_playlist_by_id(conn: &Connection, playlist_id: i64) -> Result<Option<Playlist>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT playlistId, name, description, playlistImage, ownerId, ownerName, totalMaps, createdAt, songsChangedAt, downloadURL FROM Playlist WHERE playlistId = ?1"
    )?;

    let result = stmt.query_row([playlist_id], |row| {
        Ok(Playlist {
            playlist_id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            playlist_image: row.get(3)?,
            owner_id: row.get(4)?,
            owner_name: row.get(5)?,
            total_maps: row.get(6)?,
            created_at: row.get(7)?,
            songs_changed_at: row.get(8)?,
            download_url: row.get(9)?,
        })
    });

    match result {
        Ok(playlist) => Ok(Some(playlist)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn get_all_playlists(conn: &Connection) -> Result<Vec<Playlist>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT playlistId, name, description, playlistImage, ownerId, ownerName, totalMaps, createdAt, songsChangedAt, downloadURL FROM Playlist"
    )?;

    let playlists = stmt.query_map([], |row| {
        Ok(Playlist {
            playlist_id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            playlist_image: row.get(3)?,
            owner_id: row.get(4)?,
            owner_name: row.get(5)?,
            total_maps: row.get(6)?,
            created_at: row.get(7)?,
            songs_changed_at: row.get(8)?,
            download_url: row.get(9)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(playlists)
}

pub fn delete_playlist(conn: &Connection, playlist_id: i64) -> Result<(), rusqlite::Error> {
    // 先删除歌单与谱面的关联
    conn.execute(
        "DELETE FROM PlaylistMap WHERE playlistId = ?1",
        [playlist_id],
    )?;

    // 再删除歌单元数据
    conn.execute(
        "DELETE FROM Playlist WHERE playlistId = ?1",
        [playlist_id],
    )?;
    Ok(())
}

/// 仅删除歌单的谱面关联（用于更新）
pub fn delete_playlist_maps(conn: &Connection, playlist_id: i64) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE FROM PlaylistMap WHERE playlistId = ?1",
        [playlist_id],
    )?;
    Ok(())
}

pub fn insert_playlist_map(conn: &Connection, playlist_map: &PlaylistMap) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO PlaylistMap (playlistId, mapKey, mapOrder)
         VALUES (?1, ?2, ?3)",
        rusqlite::params![
            playlist_map.playlist_id,
            playlist_map.map_key,
            playlist_map.map_order
        ],
    )?;
    Ok(())
}

pub fn get_playlist_maps(conn: &Connection, playlist_id: i64) -> Result<Vec<PlaylistMap>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT pm.playlistId, pm.mapKey, pm.mapOrder,
                m.key, m.songName, m.songAuthorName, m.levelAuthorName, m.rating, m.coverURL, m.previewURL
         FROM PlaylistMap pm
         LEFT JOIN Map m ON pm.mapKey = m.key
         WHERE pm.playlistId = ?1
         ORDER BY pm.mapOrder"
    )?;

    let maps = stmt.query_map([playlist_id], |row| {
        let key: Option<String> = row.get(3)?;
        let map = if let Some(_) = key {
            Some(Map {
                key: row.get(3)?,
                song_name: row.get(4)?,
                song_author_name: row.get(5)?,
                level_author_name: row.get(6)?,
                rating: row.get(7)?,
                cover_url: row.get(8)?,
                preview_url: row.get(9)?,
            })
        } else {
            None
        };
        Ok(PlaylistMap {
            playlist_id: row.get(0)?,
            map_key: row.get(1)?,
            map_order: row.get(2)?,
            map,
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(maps)
}

/// 获取谱面所有版本的 hash 列表
pub fn get_map_hashes_by_key(conn: &Connection, key: &str) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT hash FROM MapDiff WHERE key = ?1"
    )?;

    let hashes = stmt.query_map([key], |row| {
        row.get::<_, String>(0)
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(hashes)
}

/// 获取指定 hash 对应的 key
pub fn get_map_key_by_hash(conn: &Connection, hash: &str) -> Result<Option<String>, rusqlite::Error> {
    let result: Result<String, rusqlite::Error> = conn.query_row(
        "SELECT DISTINCT key FROM MapDiff WHERE hash = ?1 LIMIT 1",
        [hash],
        |row| row.get(0),
    );

    match result {
        Ok(key) => Ok(Some(key)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

/// 获取玩家在指定 hash 列表中的成绩
pub fn get_scores_by_hashes_and_player(
    conn: &Connection,
    hashes: &[String],
    player_id: &str,
) -> Result<Vec<PlayerScore>, rusqlite::Error> {
    if hashes.is_empty() {
        return Ok(vec![]);
    }

    let placeholders = hashes.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT playerId, scoreId, leaderboardId, mapKey, hash, mode, difficulty, accuracy, epochTime
         FROM PlayerScore WHERE playerId = ? AND hash IN ({})",
        placeholders
    );

    let mut params: Vec<&dyn rusqlite::ToSql> = vec![&player_id];
    for hash in hashes {
        params.push(hash);
    }

    let mut stmt = conn.prepare(&sql)?;
    let scores = stmt.query_map(rusqlite::params_from_iter(params), |row| {
        Ok(PlayerScore {
            player_id: row.get(0)?,
            score_id: row.get(1)?,
            leaderboard_id: row.get(2)?,
            map_key: row.get(3)?,
            hash: row.get(4)?,
            mode: row.get(5)?,
            difficulty: row.get(6)?,
            accuracy: row.get(7)?,
            epoch_time: row.get(8)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(scores)
}

/// 获取歌单谱面分页（按 mapOrder 排序）
pub fn get_playlist_maps_paginated(
    conn: &Connection,
    playlist_id: i64,
    page: i32,
    page_size: i32,
) -> Result<(Vec<String>, i32), rusqlite::Error> {
    // 获取总数
    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM PlaylistMap WHERE playlistId = ?1",
        [playlist_id],
        |row| row.get(0),
    )?;

    let total_pages = ((total as f64) / (page_size as f64)).ceil() as i32;
    let offset = (page - 1) * page_size;

    // 获取指定页的 map keys
    let mut stmt = conn.prepare(
        "SELECT mapKey FROM PlaylistMap WHERE playlistId = ?1 ORDER BY mapOrder LIMIT ?2 OFFSET ?3"
    )?;

    let keys = stmt.query_map(rusqlite::params![playlist_id, page_size, offset], |row| {
        row.get::<_, String>(0)
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok((keys, total_pages))
}