use serde::{Deserialize, Serialize};

// Map 表数据结构
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
pub struct MapDiff {
    pub key: String,
    pub hash: String,
    pub is_current: bool,
    pub mode: String,
    pub difficulty: String,
}

// PlayerInfo 表数据结构
#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub id: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub live_url: Option<String>,
    pub alias: Option<String>,
}

// PlayerScore 表数据结构
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistMap {
    pub playlist_id: i64,
    pub map_key: String,
    pub map_order: f32,
}

// 难度值转换
#[allow(dead_code)]
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

#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn parse_leaderboard_id(leaderboard_id: &str) -> (Option<String>, String, i32, i32) {
    // 格式: {key}{版本段}{难度}{模式}
    // 版本段: 空=首次上传，x=第二次，xx=第三次...
    // 难度: 1=Easy, 3=Normal, 5=Hard, 7=Expert, 9=ExpertPlus
    // 模式: 0=Legacy, 1=Standard, 2=OneSaber, 3=NoArrows, 4=90Degree, 5=360Degree, 6=Lightshow, 7=Lawless

    let x_count = leaderboard_id.matches('x').count();

    if x_count > 0 {
        // 存在版本段：以 x 为锚点分割
        let parts: Vec<&str> = leaderboard_id.split('x').collect();
        if parts.len() >= 2 {
            let key = parts[0].to_string();
            let suffix = parts.last().copied().unwrap_or("");
            if suffix.len() >= 2 {
                let difficulty = suffix[0..1].parse::<i32>().unwrap_or(0);
                let mode = suffix[1..].parse::<i32>().unwrap_or(0);
                return (Some(key), String::new(), difficulty, mode);
            }
        }
        return (None, String::new(), 0, 0);
    } else {
        // 无版本段：从末尾取模式值（1位或多位），再取难度值（1位）
        if leaderboard_id.len() >= 2 {
            let mode_start = leaderboard_id.len() - 1;
            // 尝试判断模式值位数
            let mode = leaderboard_id[mode_start..].parse::<i32>().unwrap_or(0);
            if mode > 7 {
                // 模式值可能超过1位（非常规模式）
                let difficulty = leaderboard_id[mode_start - 1..mode_start].parse::<i32>().unwrap_or(0);
                let key = &leaderboard_id[0..mode_start - 1];
                return (Some(key.to_string()), String::new(), difficulty, mode);
            } else {
                let difficulty = leaderboard_id[mode_start - 1..mode_start].parse::<i32>().unwrap_or(0);
                let key = &leaderboard_id[0..mode_start - 1];
                return (Some(key.to_string()), String::new(), difficulty, mode);
            }
        }
        return (None, String::new(), 0, 0);
    }
}