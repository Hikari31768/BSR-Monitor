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
    pub updated_at: Option<i64>,
}

// PlayerScore 表数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerScore {
    pub player_id: String,
    pub score_id: i64,
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

impl PlaylistSource {
    pub fn as_str(&self) -> &'static str {
        match self { Self::Beatsaver => "beatsaver", Self::Mapper => "mapper", Self::Local => "local" }
    }
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

/// 从 BeatLeader leaderboard ID 中解析 map_key、difficulty、mode。
///
/// 格式: `{key}{版本x+}{难度}{模式}`
/// - 版本段: 空=首次上传, `x`=第二次, `xx`=第三次...
/// - 难度: 1=Easy, 3=Normal, 5=Hard, 7=Expert, 9=ExpertPlus (1位数字)
/// - 模式: 0=Legacy, 1=Standard, 2=OneSaber, 3=NoArrows, 4=90Degree, 5=360Degree, 6=Lightshow, 7=Lawless
///
/// 示例: `"46f3cxxx91"` → key="46f3c", difficulty=9, mode=1
///       `"36303x71"` → key="36303", difficulty=7, mode=1
///       `"171f891"` → key="171f8", difficulty=9, mode=1
///
/// 注意：版本段为空且模式值超过 1 位时，key 解析不准确（BL 上游格式限制，无法避免）。
pub fn parse_leaderboard_id(leaderboard_id: &str) -> (Option<String>, i32, i32) {
    if leaderboard_id.len() < 3 {
        return (None, 0, 0);
    }

    if let Some(pos) = leaderboard_id.find('x') {
        let key = &leaderboard_id[..pos];
        // 去掉版本段的所有 x，剩余纯数字部分：{难度}{模式}
        let suffix: String = leaderboard_id[pos..].chars().filter(|c| *c != 'x').collect();
        if suffix.len() < 2 || key.is_empty() {
            return (None, 0, 0);
        }
        let difficulty = suffix[0..1].parse::<i32>().unwrap_or(0);
        let mode = suffix[1..].parse::<i32>().unwrap_or(0);
        (Some(key.to_string()), difficulty, mode)
    } else {
        // 无版本段：末尾 1 位模式 + 倒数第 2 位难度 + 剩余 key
        let len = leaderboard_id.len();
        let mode = leaderboard_id[len - 1..].parse::<i32>().unwrap_or(0);
        let difficulty = leaderboard_id[len - 2..len - 1]
            .parse::<i32>()
            .unwrap_or(0);
        let key = &leaderboard_id[..len - 2];
        if key.is_empty() {
            return (None, 0, 0);
        }
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

/// 获取谱面的首个 hash（用于 BeatLeader 查询）
pub fn get_map_hash_by_key(conn: &Connection, key: &str) -> Result<Option<String>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT hash FROM MapDiff WHERE key = ?1 AND metadataKnown = 1 ORDER BY isCurrent DESC LIMIT 1")?;
    let result = stmt.query_row([key], |row| row.get(0));
    match result {
        Ok(hash) => Ok(Some(hash)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

/// 批量插入同一版本的多个难度（先重置旧版本，再插入新版本）
pub fn insert_map_diffs_batch(conn: &Connection, key: &str, diffs: &[MapDiff]) -> Result<(), rusqlite::Error> {
    if diffs.is_empty() {
        return Ok(());
    }

    let tx = conn.unchecked_transaction()?;

    // 先将该 key 下所有记录的 isCurrent 改为 false（标记为旧版本）
    tx.execute(
        "UPDATE MapDiff SET isCurrent = 0 WHERE key = ?1",
        [key],
    )?;

    // 批量插入新记录（都标记为当前版本，hash 统一转小写）
    for diff in diffs {
        let normalized_hash = diff.hash.to_lowercase();
        tx.execute(
            "INSERT INTO MapDiff (key, hash, isCurrent, mode, difficulty, starsRating, starsFetchedAt, passRating, accRating, techRating, metadataKnown)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 1)
             ON CONFLICT(hash, mode, difficulty) DO UPDATE SET
                 key = excluded.key, isCurrent = excluded.isCurrent, metadataKnown = 1,
                 starsRating = COALESCE(excluded.starsRating, MapDiff.starsRating),
                 starsFetchedAt = COALESCE(excluded.starsFetchedAt, MapDiff.starsFetchedAt),
                 passRating = COALESCE(excluded.passRating, MapDiff.passRating),
                 accRating = COALESCE(excluded.accRating, MapDiff.accRating),
                 techRating = COALESCE(excluded.techRating, MapDiff.techRating)",
            rusqlite::params![
                diff.key,
                normalized_hash,
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
    tx.commit()?;
    Ok(())
}


fn read_diff(row: &rusqlite::Row<'_>) -> rusqlite::Result<MapDiff> {
    Ok(MapDiff {
        key: row.get(0)?, hash: row.get(1)?, is_current: row.get(2)?,
        mode: row.get(3)?, difficulty: row.get(4)?, stars_rating: row.get(5)?,
        stars_fetched_at: row.get(6)?, pass_rating: row.get(7)?,
        acc_rating: row.get(8)?, tech_rating: row.get(9)?,
    })
}

pub fn get_map_diffs_by_key(conn: &Connection, key: &str) -> rusqlite::Result<Vec<MapDiff>> {
    get_map_diffs_for_keys(conn, &[key.to_string()])
}

pub fn get_map_diffs_for_keys(conn: &Connection, keys: &[String]) -> rusqlite::Result<Vec<MapDiff>> {
    let mut diffs = Vec::new();
    for keys in keys.chunks(400) {
        let slots = vec!["?"; keys.len()].join(",");
        let mut stmt = conn.prepare(&format!(
            "SELECT key, hash, isCurrent, mode, difficulty, starsRating, starsFetchedAt,
                    passRating, accRating, techRating FROM MapDiff WHERE key IN ({slots})"
        ))?;
        diffs.extend(stmt.query_map(rusqlite::params_from_iter(keys.iter()), read_diff)?
            .collect::<rusqlite::Result<Vec<_>>>()?);
    }
    Ok(diffs)
}

/// One batch resolves playlist hashes to downloaded map metadata.
pub fn get_maps_by_hashes(conn: &Connection, hashes: &[String]) -> rusqlite::Result<Vec<(String, Map)>> {
    let mut maps = Vec::new();
    for hashes in hashes.chunks(400) {
        let hashes: Vec<_> = hashes.iter().map(|h| h.to_lowercase()).collect();
        let slots = vec!["?"; hashes.len()].join(",");
        let mut stmt = conn.prepare(&format!(
            "SELECT DISTINCT d.hash, m.key, m.songName, m.songAuthorName, m.levelAuthorName,
                             m.rating, m.coverURL, m.previewURL
             FROM MapDiff d JOIN Map m ON m.key = d.key WHERE d.hash IN ({slots})"
        ))?;
        maps.extend(stmt.query_map(rusqlite::params_from_iter(hashes.iter()), |r| Ok((
            r.get(0)?,
            Map { key: r.get(1)?, song_name: r.get(2)?, song_author_name: r.get(3)?,
                  level_author_name: r.get(4)?, rating: r.get(5)?, cover_url: r.get(6)?,
                  preview_url: r.get(7)? },
        )))?.collect::<rusqlite::Result<Vec<_>>>()?);
    }
    Ok(maps)
}


/// 更新指定 MapDiff 记录的评级信息（按 hash + mode + difficulty 匹配）
pub fn update_diff_ratings(
    conn: &Connection,
    key: &str,
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

    let updated = conn.execute(
        "UPDATE MapDiff SET
            key = ?4,
            isCurrent = CASE WHEN EXISTS (
                SELECT 1 FROM MapDiff c WHERE c.key = ?4 AND c.hash = LOWER(?1) AND c.isCurrent = 1
            ) THEN 1 ELSE isCurrent END,
            metadataKnown = 1,
            starsRating = ?5,
            starsFetchedAt = ?6,
            passRating = ?7,
            accRating = ?8,
            techRating = ?9
         WHERE hash = LOWER(?1) AND mode = ?2 AND difficulty = ?3",
        rusqlite::params![
            hash, mode, difficulty,
            key,
            stars,
            now,
            pass_rating,
            acc_rating,
            tech_rating,
        ],
    )?;

    // 如果没有匹配到现有记录，INSERT 一条（BeatLeader 可能有 BeatSaver 没有的难度模式）
    if updated == 0 {
        let normalized_hash = hash.to_lowercase();
        conn.execute(
            "INSERT OR IGNORE INTO MapDiff (key, hash, isCurrent, mode, difficulty, starsRating, starsFetchedAt, passRating, accRating, techRating, metadataKnown)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 1)",
            rusqlite::params![
                key, normalized_hash, true, mode, difficulty,
                stars,
                now,
                pass_rating,
                acc_rating,
                tech_rating,
            ],
        )?;
    }

    Ok(())
}

/// 找出指定 hash 列表中还有难度未拉取评级的 hash（去重）
pub fn get_hashes_needing_ratings(conn: &Connection, hashes: &[String]) -> Result<Vec<String>, rusqlite::Error> {
    if hashes.is_empty() {
        return Ok(vec![]);
    }

    // MapDiff 中 hash 统一存储为小写，需规范化输入后再查询
    let lower_hashes: Vec<String> = hashes.iter().map(|h| h.to_lowercase()).collect();
    let placeholders = lower_hashes.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT DISTINCT hash FROM MapDiff WHERE hash IN ({}) AND starsFetchedAt IS NULL",
        placeholders
    );

    let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
    for hash in &lower_hashes {
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

    // MapDiff 中 hash 统一存储为小写，需规范化输入后再查询
    let lower_hashes: Vec<String> = hashes.iter().map(|h| h.to_lowercase()).collect();
    let placeholders = lower_hashes.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT DISTINCT hash FROM MapDiff WHERE metadataKnown = 1 AND hash IN ({})",
        placeholders
    );

    let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
    for hash in &lower_hashes {
        params.push(hash);
    }

    let mut stmt = conn.prepare(&sql)?;
    let result = stmt.query_map(rusqlite::params_from_iter(params), |row| {
        row.get::<_, String>(0)
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(result)
}

/// 更新指定 key 的 previewURL
pub fn update_map_preview_url(conn: &Connection, key: &str, preview_url: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE Map SET previewURL = ?2 WHERE key = ?1",
        rusqlite::params![key, preview_url],
    )?;
    Ok(())
}

// ============ PlayerInfo CRUD ============

/// Profile refreshes preserve the last successful score-sync timestamp.
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
        "SELECT id, name, avatarURL, liveURL, alias, updatedAt FROM PlayerInfo WHERE id = ?1"
    )?;

    let result = stmt.query_row([id], |row| {
        Ok(PlayerInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            avatar_url: row.get(2)?,
            live_url: row.get(3)?,
            alias: row.get(4)?,
            updated_at: row.get(5)?,
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
        "SELECT id, name, avatarURL, liveURL, alias, updatedAt FROM PlayerInfo"
    )?;

    let players = stmt.query_map([], |row| {
        Ok(PlayerInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            avatar_url: row.get(2)?,
            live_url: row.get(3)?,
            alias: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(players)
}

/// 批量获取所有玩家及其成绩数量
pub fn get_players_with_score_counts(conn: &Connection) -> Result<Vec<(PlayerInfo, i64)>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.name, p.avatarURL, p.liveURL, p.alias, p.updatedAt, COUNT(ps.playerId) as score_count
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
            updated_at: row.get(5)?,
        };
        let score_count: i64 = row.get(6)?;
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

pub fn mark_player_updated(conn: &Connection, id: &str) -> rusqlite::Result<i64> {
    let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default().as_secs() as i64;
    if conn.execute("UPDATE PlayerInfo SET updatedAt = ?2,
        scoreSyncEpoch = (SELECT MAX(epochTime) FROM PlayerScore WHERE playerId = ?1) WHERE id = ?1",
        rusqlite::params![id, timestamp])? == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    Ok(timestamp)
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

// ============ Playlist CRUD ============

// Shared map fields are projected for callers, but stored only in MapDiff.
const SCORE_SELECT: &str =
    "SELECT s.playerId, d.key, d.hash, d.mode, d.difficulty, s.accuracy, s.epochTime, s.scoreId
     FROM PlayerScore s JOIN MapDiff d ON d.id = s.mapDiffId";

fn read_score(row: &rusqlite::Row<'_>) -> rusqlite::Result<PlayerScore> {
    Ok(PlayerScore {
        player_id: row.get(0)?, map_key: row.get(1)?, hash: row.get(2)?,
        mode: row.get(3)?, difficulty: row.get(4)?,
        accuracy: row.get(5)?, epoch_time: row.get(6)?, score_id: row.get(7)?,
    })
}

/// A page is atomic, including creation of previously unknown difficulties.
pub fn insert_scores_batch(conn: &Connection, scores: &[PlayerScore]) -> rusqlite::Result<usize> {
    let tx = conn.unchecked_transaction()?;
    let mut changed = 0;
    {
        let mut diff = tx.prepare_cached(
            "INSERT INTO MapDiff (key, hash, mode, difficulty) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(hash, mode, difficulty) DO UPDATE SET
                 key = COALESCE(MapDiff.key, excluded.key)"
        )?;
        let mut score_stmt = tx.prepare_cached(
            "INSERT INTO PlayerScore (playerId, mapDiffId, accuracy, epochTime, scoreId)
             SELECT ?1, id, ?5, ?6, ?7 FROM MapDiff
             WHERE hash = ?2 AND mode = ?3 AND difficulty = ?4
             ON CONFLICT(playerId, mapDiffId) DO UPDATE SET
                 accuracy = excluded.accuracy, epochTime = excluded.epochTime, scoreId = excluded.scoreId
             WHERE excluded.epochTime > PlayerScore.epochTime
                OR (excluded.epochTime = PlayerScore.epochTime AND PlayerScore.scoreId = 0 AND excluded.scoreId > 0)"
        )?;
        for score in scores {
            let hash = score.hash.to_lowercase();
            diff.execute(rusqlite::params![score.map_key, hash, score.mode, score.difficulty])?;
            changed += score_stmt.execute(rusqlite::params![
                score.player_id, hash, score.mode, score.difficulty, score.accuracy, score.epoch_time, score.score_id,
            ])?;
        }
    }
    tx.commit()?;
    Ok(changed)
}

pub fn get_scores_by_player(conn: &Connection, player_id: &str) -> rusqlite::Result<Vec<PlayerScore>> {
    let mut stmt = conn.prepare(&format!("{SCORE_SELECT} WHERE s.playerId = ?1 ORDER BY s.epochTime DESC"))?;
    let scores = stmt.query_map([player_id], read_score)?.collect();
    scores
}

pub fn get_score_count_by_player(conn: &Connection, player_id: &str) -> rusqlite::Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM PlayerScore WHERE playerId = ?1", [player_id], |r| r.get(0))
}

pub fn get_latest_score_time(conn: &Connection, player_id: &str) -> rusqlite::Result<Option<i64>> {
    conn.query_row("SELECT MAX(epochTime) FROM PlayerScore WHERE playerId = ?1", [player_id], |r| r.get(0))
}

/// Only completed syncs advance this checkpoint; partially saved pages must be retried.
pub fn get_score_sync_time(conn: &Connection, player_id: &str) -> rusqlite::Result<Option<i64>> {
    conn.query_row("SELECT scoreSyncEpoch FROM PlayerInfo WHERE id = ?1", [player_id], |r| r.get(0))
}

/// Batch maps and selected players, keeping parameter counts below SQLite's limit.
pub fn get_scores_for_maps(
    conn: &Connection, keys: &[String], player_ids: &[String],
) -> rusqlite::Result<Vec<PlayerScore>> {
    let mut scores = Vec::new();
    for keys in keys.chunks(400) {
        for players in player_ids.chunks(400) {
            let key_slots = vec!["?"; keys.len()].join(",");
            let player_slots = vec!["?"; players.len()].join(",");
            // Start with this page's difficulties instead of scanning every
            // selected player's complete score history on each query.
            let sql = format!(
                "SELECT s.playerId, d.key, d.hash, d.mode, d.difficulty, s.accuracy, s.epochTime, s.scoreId
                 FROM MapDiff d CROSS JOIN PlayerScore s ON s.mapDiffId = d.id
                 WHERE d.key IN ({key_slots}) AND s.playerId IN ({player_slots})"
            );
            let mut stmt = conn.prepare(&sql)?;
            scores.extend(stmt.query_map(
                rusqlite::params_from_iter(keys.iter().chain(players.iter())), read_score,
            )?.collect::<rusqlite::Result<Vec<_>>>()?);
        }
    }
    Ok(scores)
}

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

/// 从 .bplist 文件中读取谱面 hash 列表（保持原始顺序，统一转小写）
pub fn get_playlist_hashes_from_bplist(bplist_path: &str) -> Result<Vec<String>, String> {
    let content = std::fs::read_to_string(bplist_path)
        .map_err(|e| format!("Failed to read bplist file: {}", e))?;
    let bplist: BplistFile = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse bplist: {}", e))?;
    Ok(bplist.songs.iter().map(|s| s.hash.to_lowercase()).collect())
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

    // 先对整个列表反转，再分页
    let ordered_hashes: Vec<String> = if reverse {
        all_hashes.into_iter().rev().collect()
    } else {
        all_hashes
    };

    let offset = ((page - 1) * page_size) as usize;
    let end = (offset + page_size as usize).min(total);

    let page_hashes: Vec<String> = if offset >= total {
        vec![]
    } else {
        ordered_hashes[offset..end].to_vec()
    };

    Ok((page_hashes, total_pages))
}
