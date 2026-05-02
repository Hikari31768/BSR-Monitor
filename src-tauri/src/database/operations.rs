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
    pub stars_rating: Option<f32>,
    pub stars_fetched_at: Option<i64>,
    pub pass_rating: Option<f32>,
    pub acc_rating: Option<f32>,
    pub tech_rating: Option<f32>,
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

// 歌单来源
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PlaylistSource {
    Beatsaver,
    Mapper,
    Local,
}

impl rusqlite::types::ToSql for PlaylistSource {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        let s = match self {
            PlaylistSource::Beatsaver => "beatsaver",
            PlaylistSource::Mapper => "mapper",
            PlaylistSource::Local => "local",
        };
        Ok(rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Text(s.to_string())))
    }
}

impl rusqlite::types::FromSql for PlaylistSource {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value.as_str()? {
            "beatsaver" => Ok(PlaylistSource::Beatsaver),
            "mapper" => Ok(PlaylistSource::Mapper),
            "local" => Ok(PlaylistSource::Local),
            _ => Err(rusqlite::types::FromSqlError::Other(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid playlist source",
            )))),
        }
    }
}

// Playlist 表数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Playlist {
    pub source: PlaylistSource,
    pub identifier: String,
    pub name: String,
    pub description: Option<String>,
    pub playlist_image: Option<String>,
    pub owner_id: Option<String>,
    pub owner_name: Option<String>,
    pub total_maps: Option<i32>,
    pub created_at: Option<String>,
    pub songs_changed_at: Option<String>,
    pub download_url: Option<String>,
    pub local_path: Option<String>,
    pub last_bplist_fetched_at: Option<i64>,
    pub updated_at: Option<i64>,
}

// .bplist 文件解析结构体
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct BplistSong {
    pub key: Option<String>,
    #[serde(rename = "songName")]
    pub song_name: Option<String>,
    #[serde(rename = "levelAuthorName")]
    pub level_author_name: Option<String>,
    pub hash: String,
    #[serde(rename = "levelID")]
    pub levelid: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BplistCustomData {
    #[serde(rename = "syncURL")]
    pub sync_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BplistFile {
    #[serde(rename = "playlistTitle")]
    pub playlist_title: String,
    #[serde(rename = "playlistAuthor")]
    pub playlist_author: String,
    #[serde(rename = "customData")]
    pub custom_data: Option<BplistCustomData>,
    pub songs: Vec<BplistSong>,
    #[serde(default)]
    pub image: Option<String>,
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

// leaderboardId 解析
/// 解析 leaderboardId，返回 (key, difficulty, mode)
/// 格式: "keyxD{mode}" 或 "keyD{mode}"，失败返回 (None, 0, 0)
pub fn parse_leaderboard_id(leaderboard_id: &str) -> (Option<String>, i32, i32) {
    if leaderboard_id.is_empty() {
        return (None, 0, 0);
    }

    if let Some((left, suffix)) = leaderboard_id.rsplit_once('x') {
        if left.is_empty() || suffix.len() < 2 {
            return (None, 0, 0);
        }
        let difficulty = suffix[0..1].parse::<i32>().unwrap_or(0);
        let mode = suffix[1..].parse::<i32>().unwrap_or(0);
        (Some(left.to_string()), difficulty, mode)
    } else {
        if leaderboard_id.len() < 2 {
            return (None, 0, 0);
        }
        let mode_start = leaderboard_id.len() - 1;
        let mode = leaderboard_id[mode_start..].parse::<i32>().unwrap_or(0);
        let difficulty = leaderboard_id[mode_start - 1..mode_start].parse::<i32>().unwrap_or(0);
        let key = &leaderboard_id[0..mode_start - 1];
        (Some(key.to_string()), difficulty, mode)
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
            "INSERT INTO MapDiff (key, hash, isCurrent, mode, difficulty, starsRating, starsFetchedAt, passRating, accRating, techRating)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(key, hash, mode, difficulty) DO UPDATE SET isCurrent = excluded.isCurrent",
            rusqlite::params![
                diff.key,
                diff.hash,
                true,
                diff.mode,
                diff.difficulty,
                diff.stars_rating,
                diff.stars_fetched_at,
                diff.pass_rating,
                diff.acc_rating,
                diff.tech_rating,
            ],
        )?;
    }
    Ok(())
}


pub fn get_map_diffs_by_key(conn: &Connection, key: &str) -> Result<Vec<MapDiff>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT key, hash, isCurrent, mode, difficulty, starsRating, starsFetchedAt, passRating, accRating, techRating FROM MapDiff WHERE key = ?1"
    )?;

    let diffs = stmt.query_map([key], |row| {
        Ok(MapDiff {
            key: row.get(0)?,
            hash: row.get(1)?,
            is_current: row.get(2)?,
            mode: row.get(3)?,
            difficulty: row.get(4)?,
            stars_rating: row.get(5)?,
            stars_fetched_at: row.get(6)?,
            pass_rating: row.get(7)?,
            acc_rating: row.get(8)?,
            tech_rating: row.get(9)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(diffs)
}

/// 通过 hash 查找 MapDiff 记录（用于歌单模式通过 hash 反查 key）
pub fn get_map_diffs_by_key_for_hash(conn: &Connection, hash: &str) -> Result<Vec<MapDiff>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT key, hash, isCurrent, mode, difficulty, starsRating, starsFetchedAt, passRating, accRating, techRating FROM MapDiff WHERE hash = ?1"
    )?;

    let diffs = stmt.query_map([hash], |row| {
        Ok(MapDiff {
            key: row.get(0)?,
            hash: row.get(1)?,
            is_current: row.get(2)?,
            mode: row.get(3)?,
            difficulty: row.get(4)?,
            stars_rating: row.get(5)?,
            stars_fetched_at: row.get(6)?,
            pass_rating: row.get(7)?,
            acc_rating: row.get(8)?,
            tech_rating: row.get(9)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(diffs)
}

/// 找出指定 hash 列表中还有难度未拉取评级的 hash（去重）
pub fn get_hashes_needing_ratings(conn: &Connection, hashes: &[String]) -> Result<Vec<String>, rusqlite::Error> {
    if hashes.is_empty() {
        return Ok(vec![]);
    }

    let placeholders = hashes.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT DISTINCT hash FROM MapDiff WHERE hash IN ({}) AND starsFetchedAt IS NULL",
        placeholders
    );

    let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
    for hash in hashes {
        params.push(hash);
    }

    let mut stmt = conn.prepare(&sql)?;
    let result = stmt.query_map(rusqlite::params_from_iter(params), |row| {
        row.get::<_, String>(0)
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(result)
}

/// 找出指定 hash 列表中已在数据库中存在的 hash（用于增量补全时跳过已收录的谱面）
pub fn get_existing_hashes(conn: &Connection, hashes: &[String]) -> Result<Vec<String>, rusqlite::Error> {
    if hashes.is_empty() {
        return Ok(vec![]);
    }

    let placeholders = hashes.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT DISTINCT hash FROM MapDiff WHERE hash IN ({})",
        placeholders
    );

    let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
    for hash in hashes {
        params.push(hash);
    }

    let mut stmt = conn.prepare(&sql)?;
    let result = stmt.query_map(rusqlite::params_from_iter(params), |row| {
        row.get::<_, String>(0)
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(result)
}

/// 更新指定难度的所有评级（按 hash + mode + difficulty 定位）
pub fn update_diff_ratings(
    conn: &Connection,
    hash: &str,
    mode: &str,
    difficulty: &str,
    stars: Option<f32>,
    pass_rating: Option<f32>,
    acc_rating: Option<f32>,
    tech_rating: Option<f32>,
) -> Result<(), rusqlite::Error> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    conn.execute(
        "UPDATE MapDiff SET starsRating = ?1, starsFetchedAt = ?2, passRating = ?3, accRating = ?4, techRating = ?5 WHERE hash = ?6 AND mode = ?7 AND difficulty = ?8",
        rusqlite::params![stars, now, pass_rating, acc_rating, tech_rating, hash, mode, difficulty],
    )?;

    Ok(())
}

// ============ PlayerInfo CRUD ============

pub fn insert_player(conn: &Connection, player: &PlayerInfo) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO PlayerInfo (id, name, avatarURL, liveURL, alias)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET
           name = excluded.name,
           avatarURL = excluded.avatarURL,
           liveURL = COALESCE(excluded.liveURL, PlayerInfo.liveURL),
           alias = COALESCE(excluded.alias, PlayerInfo.alias)",
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

/// 批量获取所有玩家及其成绩数量
pub fn get_players_with_score_counts(conn: &Connection) -> Result<Vec<(PlayerInfo, i64)>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.name, p.avatarURL, p.liveURL, p.alias, COUNT(ps.playerId) as score_count
         FROM PlayerInfo p
         LEFT JOIN PlayerScore ps ON p.id = ps.playerId
         GROUP BY p.id"
    )?;

    let players = stmt.query_map([], |row| {
        let player = PlayerInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            avatar_url: row.get(2)?,
            live_url: row.get(3)?,
            alias: row.get(4)?,
        };
        let score_count: i64 = row.get(5)?;
        Ok((player, score_count))
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(players)
}

pub fn delete_player(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("BEGIN", []).map_err(|e| e.to_string())?;
    if let Err(e) = conn.execute(
        "DELETE FROM PlayerScore WHERE playerId = ?1",
        [id],
    ) {
        let _ = conn.execute("ROLLBACK", []);
        return Err(e.to_string());
    }
    if let Err(e) = conn.execute(
        "DELETE FROM PlayerInfo WHERE id = ?1",
        [id],
    ) {
        let _ = conn.execute("ROLLBACK", []);
        return Err(e.to_string());
    }
    conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
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

/// 原子化插入：仅当记录不存在或新成绩时间更晚时才更新/插入
pub fn insert_score(conn: &Connection, score: &PlayerScore) -> Result<bool, rusqlite::Error> {
    let updated = conn.execute(
        "UPDATE PlayerScore SET scoreId = ?2, leaderboardId = ?3, mapKey = ?4, accuracy = ?5, epochTime = ?6
         WHERE playerId = ?1 AND hash = ?7 AND mode = ?8 AND difficulty = ?9
         AND epochTime < ?10",
        rusqlite::params![
            score.player_id,
            score.score_id,
            score.leaderboard_id,
            score.map_key,
            score.accuracy,
            score.epoch_time,
            score.hash,
            score.mode,
            score.difficulty,
            score.epoch_time,
        ],
    )?;

    if updated > 0 {
        return Ok(true);
    }

    let inserted = conn.execute(
        "INSERT INTO PlayerScore (playerId, scoreId, leaderboardId, mapKey, hash, mode, difficulty, accuracy, epochTime)
         SELECT ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9
         WHERE NOT EXISTS (
             SELECT 1 FROM PlayerScore
             WHERE playerId = ?1 AND hash = ?5 AND mode = ?6 AND difficulty = ?7
         )",
        rusqlite::params![
            score.player_id,
            score.score_id,
            score.leaderboard_id,
            score.map_key,
            score.hash,
            score.mode,
            score.difficulty,
            score.accuracy,
            score.epoch_time,
        ],
    )?;

    Ok(inserted > 0)
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
        "INSERT OR REPLACE INTO Playlist (source, identifier, name, description, playlistImage, ownerId, ownerName, totalMaps, createdAt, songsChangedAt, downloadURL, localPath, lastBplistFetchedAt, updatedAt)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        rusqlite::params![
            playlist.source,
            playlist.identifier,
            playlist.name,
            playlist.description,
            playlist.playlist_image,
            playlist.owner_id,
            playlist.owner_name,
            playlist.total_maps,
            playlist.created_at,
            playlist.songs_changed_at,
            playlist.download_url,
            playlist.local_path,
            playlist.last_bplist_fetched_at,
            playlist.updated_at,
        ],
    )?;
    Ok(())
}

pub fn get_playlist(conn: &Connection, source: &PlaylistSource, identifier: &str) -> Result<Option<Playlist>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT source, identifier, name, description, playlistImage, ownerId, ownerName, totalMaps, createdAt, songsChangedAt, downloadURL, localPath, lastBplistFetchedAt, updatedAt FROM Playlist WHERE source = ?1 AND identifier = ?2"
    )?;

    let result = stmt.query_row(rusqlite::params![source, identifier], |row| {
        Ok(Playlist {
            source: row.get(0)?,
            identifier: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            playlist_image: row.get(4)?,
            owner_id: row.get(5)?,
            owner_name: row.get(6)?,
            total_maps: row.get(7)?,
            created_at: row.get(8)?,
            songs_changed_at: row.get(9)?,
            download_url: row.get(10)?,
            local_path: row.get(11)?,
            last_bplist_fetched_at: row.get(12)?,
            updated_at: row.get(13)?,
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
        "SELECT source, identifier, name, description, playlistImage, ownerId, ownerName, totalMaps, createdAt, songsChangedAt, downloadURL, localPath, lastBplistFetchedAt, updatedAt FROM Playlist"
    )?;

    let playlists = stmt.query_map([], |row| {
        Ok(Playlist {
            source: row.get(0)?,
            identifier: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            playlist_image: row.get(4)?,
            owner_id: row.get(5)?,
            owner_name: row.get(6)?,
            total_maps: row.get(7)?,
            created_at: row.get(8)?,
            songs_changed_at: row.get(9)?,
            download_url: row.get(10)?,
            local_path: row.get(11)?,
            last_bplist_fetched_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(playlists)
}

pub fn delete_playlist(conn: &Connection, source: &PlaylistSource, identifier: &str) -> Result<(), String> {
    conn.execute(
        "DELETE FROM Playlist WHERE source = ?1 AND identifier = ?2",
        rusqlite::params![source, identifier],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

/// 从 .bplist 文件中读取谱面 hash 列表（保持原始顺序）
pub fn get_playlist_hashes_from_bplist(bplist_path: &str) -> Result<Vec<String>, String> {
    let content = std::fs::read_to_string(bplist_path)
        .map_err(|e| format!("Failed to read bplist file: {}", e))?;
    let bplist: BplistFile = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse bplist: {}", e))?;
    Ok(bplist.songs.iter().map(|s| s.hash.clone()).collect())
}

/// 获取歌单中所有谱面的 hash 列表（从 .bplist 文件读取）
pub fn get_playlist_hashes(conn: &Connection, source: &PlaylistSource, identifier: &str) -> Result<Vec<String>, String> {
    let playlist = get_playlist(conn, source, identifier)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Playlist not found: {:?}/{}", source, identifier))?;

    let bplist_path = match &playlist.local_path {
        Some(path) => path.clone(),
        None => return Err("Playlist has no local bplist path".to_string()),
    };

    get_playlist_hashes_from_bplist(&bplist_path)
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

/// 获取歌单谱面分页（按 .bplist 中 songs 顺序）
pub fn get_playlist_hashes_paginated(
    conn: &Connection,
    source: &PlaylistSource,
    identifier: &str,
    page: i32,
    page_size: i32,
    reverse: bool,
) -> Result<(Vec<String>, i32), String> {
    let playlist = get_playlist(conn, source, identifier)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Playlist not found".to_string())?;

    let bplist_path = playlist.local_path
        .ok_or_else(|| "Playlist has no local bplist path".to_string())?;

    let all_hashes = get_playlist_hashes_from_bplist(&bplist_path)?;
    let total = all_hashes.len();
    let total_pages = ((total as f64) / (page_size as f64)).ceil() as i32;

    let offset = ((page - 1) * page_size) as usize;
    let end = (offset + page_size as usize).min(total);

    let mut page_hashes: Vec<String>;
    if offset >= total {
        page_hashes = vec![];
    } else {
        page_hashes = all_hashes[offset..end].to_vec();
    }

    if reverse {
        page_hashes.reverse();
    }

    Ok((page_hashes, total_pages))
}
