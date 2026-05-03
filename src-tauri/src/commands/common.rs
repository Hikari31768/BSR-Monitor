use crate::api::MapDetail;
use crate::database::operations::Map;
use tauri::Manager;
use std::path::PathBuf;

/// 将 API MapDetail 转换为数据库 Map 结构
pub fn convert_map_detail(map_detail: &MapDetail) -> Map {
    let (cover_url, preview_url) = map_detail.versions.first()
        .map(|v| (Some(v.cover_url.clone()), Some(v.preview_url.clone())))
        .unwrap_or((None, None));

    Map {
        key: map_detail.id.clone(),
        song_name: map_detail.metadata.song_name.clone(),
        song_author_name: map_detail.metadata.song_author_name.clone(),
        level_author_name: map_detail.metadata.level_author_name.clone(),
        rating: map_detail.stats.as_ref().map(|s| s.score),
        cover_url,
        preview_url,
    }
}

/// 获取应用版本号
#[tauri::command]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 打开应用数据目录
#[tauri::command]
pub fn open_data_dir(app: tauri::AppHandle) -> Result<(), String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    open::that(dir).map_err(|e| e.to_string())
}

/// 用户偏好数据结构
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct UserPrefs {
    pub score_table: ScoreTablePrefs,
    pub volume: f32,
    pub starred_player_id: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ScoreTablePrefs {
    pub view_mode: String,
    pub selected_playlist_key: Option<String>,
    pub map_sort: String,
    pub diff_sort: String,
    pub selected_player_ids: Vec<String>,
}

impl Default for UserPrefs {
    fn default() -> Self {
        Self {
            score_table: ScoreTablePrefs {
                view_mode: "single".to_string(),
                selected_playlist_key: None,
                map_sort: "asc".to_string(),
                diff_sort: "desc".to_string(),
                selected_player_ids: Vec::new(),
            },
            volume: 1.0,
            starred_player_id: None,
        }
    }
}

fn get_prefs_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path().app_data_dir()
        .map(|dir| dir.join("user_prefs.json"))
        .map_err(|e| e.to_string())
}

/// 加载用户偏好
#[tauri::command]
pub fn load_user_prefs(app: tauri::AppHandle) -> Result<UserPrefs, String> {
    let path = get_prefs_path(&app)?;

    if !path.exists() {
        return Ok(UserPrefs::default());
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取偏好文件失败: {}", e))?;

    let prefs: UserPrefs = serde_json::from_str(&content)
        .map_err(|e| format!("解析偏好文件失败: {}", e))?;

    Ok(prefs)
}

/// 保存用户偏好
#[tauri::command]
pub fn save_user_prefs(app: tauri::AppHandle, prefs: UserPrefs) -> Result<(), String> {
    let path = get_prefs_path(&app)?;

    // 确保目录存在
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }

    let content = serde_json::to_string_pretty(&prefs)
        .map_err(|e| format!("序列化偏好失败: {}", e))?;

    std::fs::write(&path, content)
        .map_err(|e| format!("写入偏好文件失败: {}", e))?;

    Ok(())
}
