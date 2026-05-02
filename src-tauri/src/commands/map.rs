use tauri::AppHandle;
use tauri::Manager;
use tauri::Emitter;
use crate::api::BeatSaverApi;
use crate::api::BeatLeaderApi;
use crate::database::operations::{MapDiff};
use crate::database::operations as db;
use crate::commands::common::convert_map_detail;
use std::sync::Mutex;

/// 在后台调用 BeatLeader /leaderboards/hash/{hash} 补全星级和评级信息
/// 完成后通过 `map-stars-ready` 事件通知前端（携带 map key）
fn spawn_enrich_stars(app: &AppHandle, key: String, hash: String) {
    let app_clone = app.clone();
    tokio::spawn(async move {
        let api = match BeatLeaderApi::new() {
            Ok(a) => a,
            Err(_) => return,
        };

        let response = match api.get_leaderboards_by_hash(&hash).await {
            Ok(r) => r,
            Err(_) => return,
        };

        let state = match app_clone.try_state::<Mutex<rusqlite::Connection>>() {
            Some(s) => s,
            None => return,
        };
        let guard = match state.lock() {
            Ok(g) => g,
            Err(_) => return,
        };

        for entry in &response.leaderboards {
            // 过滤非常规模式
            if entry.difficulty.mode > 8 {
                continue;
            }

            let mode = &entry.difficulty.mode_name;
            let difficulty = &entry.difficulty.difficulty_name;
            let diff = &entry.difficulty;

            // 使用 BeatLeader 返回的 hash（与 PlayerScore 中的 hash 格式一致）
            let bl_hash = &response.song.hash;
            let _ = db::update_diff_ratings(&guard, &key, bl_hash, mode, difficulty,
                diff.stars, diff.pass_rating, diff.acc_rating, diff.tech_rating);
        }

        drop(guard);

        // 通知前端：该谱面的星级已更新，可以刷新成绩
        let _ = app_clone.emit("map-stars-ready", &key);
    });
}

/// 通过 key 获取谱面
#[tauri::command]
pub async fn get_map_by_key(app: AppHandle, key: String) -> Result<crate::database::operations::Map, String> {
    let api = BeatSaverApi::new().map_err(|e| e.to_string())?;

    // 先检查数据库
    let existing_map = {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::get_map_by_key(&guard, &key).map_err(|e| e.to_string())?
    };

    if let Some(map) = existing_map {
        // 即使数据库已有，也在后台补全 BeatLeader 星级
        let hash = {
            let state = app.try_state::<Mutex<rusqlite::Connection>>()
                .ok_or("Database not initialized")?;
            let guard = state.lock().map_err(|e| e.to_string())?;
            db::get_map_hash_by_key(&guard, &key).map_err(|e| e.to_string())?
        };
        if let Some(h) = hash {
            spawn_enrich_stars(&app, key.clone(), h);
        }
        return Ok(map);
    }

    // 调用 BeatSaver API 获取谱面
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

    let hash = map_detail.versions.first().map(|v| v.hash.clone());

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
            pass_rating: None,
            acc_rating: None,
            tech_rating: None,
        }).collect();

        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::insert_map_diffs_batch(&guard, &key, &diffs).map_err(|e| e.to_string())?;
    }

    // 在后台补全 BeatLeader 星级
    if let Some(h) = hash {
        spawn_enrich_stars(&app, key.clone(), h);
    }

    Ok(map)
}

/// 通过 hash 获取谱面
#[tauri::command]
pub async fn get_map_by_hash(app: AppHandle, hash: String) -> Result<crate::database::operations::Map, String> {
    let api = BeatSaverApi::new().map_err(|e| e.to_string())?;

    // 调用 BeatSaver API 获取谱面
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
            pass_rating: None,
            acc_rating: None,
            tech_rating: None,
        }).collect();

        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::insert_map_diffs_batch(&guard, &key, &diffs).map_err(|e| e.to_string())?;
    }

    // 在后台补全 BeatLeader 星级
    spawn_enrich_stars(&app, key.clone(), hash);

    Ok(map)
}
