use tauri::AppHandle;
use tauri::Manager;
use crate::api::BeatSaverApi;
use crate::database::operations::{MapDiff};
use crate::database::operations as db;
use crate::commands::common::convert_map_detail;
use std::sync::Mutex;

/// 通过 key 获取谱面
#[tauri::command]
pub async fn get_map_by_key(app: AppHandle, key: String) -> Result<crate::database::operations::Map, String> {
    let api = BeatSaverApi::new().map_err(|e| e.to_string())?;

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

    // 存入 MapDiff 表（批量插入）
    if let Some(version) = map_detail.versions.first() {
        let diffs: Vec<MapDiff> = version.diffs.iter().map(|diff| MapDiff {
            key: key.clone(),
            hash: version.hash.clone(),
            is_current: true,
            mode: diff.characteristic.clone(),
            difficulty: diff.difficulty.clone(),
            stars_rating: diff.stars,
            stars_fetched_at: None,
        }).collect();

        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::insert_map_diffs_batch(&guard, &key, &diffs).map_err(|e| e.to_string())?;
    }

    Ok(map)
}

/// 通过 hash 获取谱面
#[tauri::command]
pub async fn get_map_by_hash(app: AppHandle, hash: String) -> Result<crate::database::operations::Map, String> {
    let api = BeatSaverApi::new().map_err(|e| e.to_string())?;

    // 调用 API 获取谱面
    let map_detail = api.get_map_by_hash(&hash)
        .await
        .map_err(|e| e.to_string())?;

    // 转换并存入数据库
    let map = convert_map_detail(&map_detail);
    let key = map_detail.id.clone();

    // 存入 Map 表
    {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::insert_map(&guard, &map).map_err(|e| e.to_string())?;
    }

    // 存入 MapDiff 表（批量插入）
    if let Some(version) = map_detail.versions.first() {
        let diffs: Vec<MapDiff> = version.diffs.iter().map(|diff| MapDiff {
            key: key.clone(),
            hash: version.hash.clone(),
            is_current: true,
            mode: diff.characteristic.clone(),
            difficulty: diff.difficulty.clone(),
            stars_rating: diff.stars,
            stars_fetched_at: None,
        }).collect();

        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::insert_map_diffs_batch(&guard, &key, &diffs).map_err(|e| e.to_string())?;
    }

    Ok(map)
}
