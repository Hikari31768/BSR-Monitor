use tauri::AppHandle;
use tauri::Manager;
use crate::api::BeatSaverApi;
use crate::database::operations::{Map, MapDiff};
use crate::database::operations as db;
use std::sync::Mutex;

/// 通过 key 获取谱面
#[tauri::command]
pub async fn get_map_by_key(app: AppHandle, key: String) -> Result<Map, String> {
    let api = BeatSaverApi::new();

    // 先检查数据库
    {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        if let Some(map) = db::get_map_by_key(&guard, &key).map_err(|e| e.to_string())? {
            return Ok(map);
        }
    }

    // 调用 API 获取谱面
    let map_detail = api.get_map_by_key(&key)
        .await
        .map_err(|e| e.to_string())?;

    // 转换并存入数据库
    let map = convert_map_detail(&map_detail);

    // 存入 Map 表
    {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::insert_map(&guard, &map).map_err(|e| e.to_string())?;
    }

    // 存入 MapDiff 表
    if let Some(version) = map_detail.versions.first() {
        for diff in &version.diffs {
            let map_diff = MapDiff {
                key: key.clone(),
                hash: version.hash.clone(),
                is_current: true,
                mode: diff.characteristic.clone(),
                difficulty: diff.difficulty.clone(),
            };
            let state = app.try_state::<Mutex<rusqlite::Connection>>()
                .ok_or("Database not initialized")?;
            let guard = state.lock().map_err(|e| e.to_string())?;
            db::insert_map_diff(&guard, &map_diff).map_err(|e| e.to_string())?;
        }
    }

    Ok(map)
}

/// 通过 hash 获取谱面
#[tauri::command]
pub async fn get_map_by_hash(app: AppHandle, hash: String) -> Result<Map, String> {
    let api = BeatSaverApi::new();

    // 调用 API 获取谱面
    let map_detail = api.get_map_by_hash(&hash)
        .await
        .map_err(|e| e.to_string())?;

    // 转换并存入数据库
    let map = convert_map_detail(&map_detail);

    // 存入 Map 表
    {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::insert_map(&guard, &map).map_err(|e| e.to_string())?;
    }

    // 存入 MapDiff 表
    if let Some(version) = map_detail.versions.first() {
        for diff in &version.diffs {
            let map_diff = MapDiff {
                key: map_detail.id.clone(),
                hash: version.hash.clone(),
                is_current: true,
                mode: diff.characteristic.clone(),
                difficulty: diff.difficulty.clone(),
            };
            let state = app.try_state::<Mutex<rusqlite::Connection>>()
                .ok_or("Database not initialized")?;
            let guard = state.lock().map_err(|e| e.to_string())?;
            db::insert_map_diff(&guard, &map_diff).map_err(|e| e.to_string())?;
        }
    }

    Ok(map)
}

/// 将 API 响应转换为数据库结构
fn convert_map_detail(map_detail: &crate::api::MapDetail) -> Map {
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