use tauri::AppHandle;
use tauri::Manager;
use crate::api::BeatSaverApi;
use crate::database::operations::{Playlist, PlaylistMap, Map, MapDiff};
use crate::database::operations as db;
use std::sync::Mutex;
use uuid::Uuid;
use crate::progress::{ProgressPayload, emit_progress_event};

/// 添加歌单或增量更新（如已存在）- 支持进度反馈
#[tauri::command]
pub async fn add_playlist(app: AppHandle, playlist_id: i32) -> Result<Playlist, String> {
    let task_id = Uuid::new_v4().to_string();
    let api = BeatSaverApi::new().map_err(|e| e.to_string())?;

    // 检查是否已存在
    let existing = {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::get_playlist_by_id(&guard, playlist_id as i64).map_err(|e| e.to_string())?
    };

    if existing.is_some() {
        // 已存在 -> 增量更新
        update_existing_playlist(app, playlist_id, task_id, api).await
    } else {
        // 不存在 -> 全量添加
        add_new_playlist(app, playlist_id, task_id, api).await
    }
}

const PLAYLIST_PAGE_SIZE: i32 = 500;

/// 全量添加新歌单
async fn add_new_playlist(
    app: AppHandle,
    playlist_id: i32,
    task_id: String,
    api: BeatSaverApi,
) -> Result<Playlist, String> {
    // 发送初始进度
    let payload = ProgressPayload::new("add_playlist".to_string(), task_id.clone())
        .with_progress("fetching_playlist".to_string(), 5.0)
        .with_message("正在获取歌单信息...".to_string());
    emit_progress_event(&app, &payload).map_err(|e| e.to_string())?;

    // 拉取第一页
    let first_page = api.get_playlist(playlist_id, 0)
        .await
        .map_err(|e| e.to_string())?;

    println!("[Playlist] First page: {} maps, type: {:?}, API total: {}",
        first_page.maps.len(),
        first_page.playlist.playlist_type,
        first_page.playlist.stats.as_ref().map(|s| s.total_maps).unwrap_or(0));

    // 判断是否为 Search 类型
    let is_search_type = first_page.playlist.playlist_type == Some(crate::api::PlaylistType::Search);

    // 转换歌单元数据（先使用 API 返回的 total_maps）
    let playlist = convert_playlist_response(&first_page);

    // 发送进度 - 开始保存歌单信息
    let payload = ProgressPayload::new("add_playlist".to_string(), task_id.clone())
        .with_progress("saving_playlist_info".to_string(), 10.0)
        .with_message(format!("已获取歌单: {}", playlist.name));
    emit_progress_event(&app, &payload).map_err(|e| e.to_string())?;

    // 存入歌单
    {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::insert_playlist(&guard, &playlist).map_err(|e| e.to_string())?;
    }

    // 处理第一页的谱面
    let mut total_actual_count = first_page.maps.len();
    save_playlist_maps_with_progress(&app, playlist_id, &first_page, &task_id, 1, 1)?;

    // 根据实际返回数量判断是否需要继续拉取
    let mut current_page = 0;
    let mut current_page_count = first_page.maps.len();
    while current_page_count == PLAYLIST_PAGE_SIZE as usize {
        current_page += 1;
        println!("[Playlist] Fetching page {}...", current_page);

        let page_response = api.get_playlist(playlist_id, current_page)
            .await
            .map_err(|e| e.to_string())?;

        current_page_count = page_response.maps.len();
        println!("[Playlist] Page {} returned {} maps", current_page, current_page_count);

        if current_page_count == 0 {
            break;
        }

        total_actual_count += current_page_count;
        save_playlist_maps_with_progress(&app, playlist_id, &page_response, &task_id, current_page as u64 + 1, current_page as u64 + 1)?;
    }

    println!("[Playlist] Total fetched: {} maps, is_search: {}", total_actual_count, is_search_type);

    // 对于 Search 类型歌单，更新 total_maps 为实际数量
    if is_search_type {
        println!("[Playlist] Updating total_maps to {}", total_actual_count);
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::update_playlist_total_maps(&guard, playlist_id as i64, Some(total_actual_count as i32))
            .map_err(|e| e.to_string())?;
    }

    // 发送完成进度
    let final_total = if is_search_type { total_actual_count as i32 } else { playlist.total_maps.unwrap_or(0) };
    let payload = ProgressPayload::new("add_playlist".to_string(), task_id.clone())
        .with_progress("completed".to_string(), 100.0)
        .with_message(format!("完成！共处理 {} 首歌曲", final_total));
    emit_progress_event(&app, &payload).map_err(|e| e.to_string())?;

    // 返回最终歌单信息
    let final_playlist = {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::get_playlist_by_id(&guard, playlist_id as i64)
            .map_err(|e| e.to_string())?
            .unwrap_or(playlist)
    };

    Ok(final_playlist)
}

/// 增量更新已存在的歌单
async fn update_existing_playlist(
    app: AppHandle,
    playlist_id: i32,
    task_id: String,
    api: BeatSaverApi,
) -> Result<Playlist, String> {
    println!("[Playlist] Updating existing playlist {}", playlist_id);

    // 发送初始进度
    let payload = ProgressPayload::new("add_playlist".to_string(), task_id.clone())
        .with_progress("fetching_playlist".to_string(), 5.0)
        .with_message("正在获取最新歌单信息...".to_string());
    emit_progress_event(&app, &payload).map_err(|e| e.to_string())?;

    // 拉取第一页
    let first_page = api.get_playlist(playlist_id, 0)
        .await
        .map_err(|e| e.to_string())?;

    println!("[Playlist] First page: {} maps, type: {:?}",
        first_page.maps.len(),
        first_page.playlist.playlist_type);

    // 判断是否为 Search 类型
    let is_search_type = first_page.playlist.playlist_type == Some(crate::api::PlaylistType::Search);

    // 转换歌单元数据
    let playlist = convert_playlist_response(&first_page);

    // 发送进度
    let payload = ProgressPayload::new("add_playlist".to_string(), task_id.clone())
        .with_progress("saving_playlist_info".to_string(), 10.0)
        .with_message(format!("已获取歌单: {}", playlist.name));
    emit_progress_event(&app, &payload).map_err(|e| e.to_string())?;

    // 更新歌单信息
    {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::insert_playlist(&guard, &playlist).map_err(|e| e.to_string())?;
    }

    // 删除旧谱面关联（重新拉取）
    {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::delete_playlist_maps(&guard, playlist_id as i64).map_err(|e| e.to_string())?;
    }

    // 处理第一页的谱面
    let mut total_actual_count = first_page.maps.len();
    save_playlist_maps_with_progress(&app, playlist_id, &first_page, &task_id, 1, 1)?;

    // 根据实际返回数量判断是否需要继续拉取
    let mut current_page = 0;
    let mut current_page_count = first_page.maps.len();
    while current_page_count == PLAYLIST_PAGE_SIZE as usize {
        current_page += 1;
        println!("[Playlist] Fetching page {}...", current_page);

        let page_response = api.get_playlist(playlist_id, current_page)
            .await
            .map_err(|e| e.to_string())?;

        current_page_count = page_response.maps.len();
        println!("[Playlist] Page {} returned {} maps", current_page, current_page_count);

        if current_page_count == 0 {
            break;
        }

        total_actual_count += current_page_count;
        save_playlist_maps_with_progress(&app, playlist_id, &page_response, &task_id, current_page as u64 + 1, current_page as u64 + 1)?;
    }

    println!("[Playlist] Total fetched: {} maps, is_search: {}", total_actual_count, is_search_type);

    // 对于 Search 类型歌单，更新 total_maps 为实际数量
    if is_search_type {
        println!("[Playlist] Updating total_maps to {}", total_actual_count);
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::update_playlist_total_maps(&guard, playlist_id as i64, Some(total_actual_count as i32))
            .map_err(|e| e.to_string())?;
    }

    // 发送完成进度
    let final_total = if is_search_type { total_actual_count as i32 } else { playlist.total_maps.unwrap_or(0) };
    let payload = ProgressPayload::new("add_playlist".to_string(), task_id.clone())
        .with_progress("completed".to_string(), 100.0)
        .with_message(format!("更新完成！共处理 {} 首歌曲", final_total));
    emit_progress_event(&app, &payload).map_err(|e| e.to_string())?;

    // 返回最终歌单信息
    let final_playlist = {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::get_playlist_by_id(&guard, playlist_id as i64)
            .map_err(|e| e.to_string())?
            .unwrap_or(playlist)
    };

    Ok(final_playlist)
}

/// 带进度的保存歌单谱面
fn save_playlist_maps_with_progress(
    app: &AppHandle,
    playlist_id: i32,
    page: &crate::api::PlaylistPage,
    task_id: &str,
    current_page: u64,
    total_pages: u64
) -> Result<(), String> {
    // 发送当前页进度
    let payload = ProgressPayload::new("add_playlist".to_string(), task_id.to_string())
        .with_count("processing_page".to_string(), current_page, total_pages)
        .with_message(format!("正在处理第 {} 页谱面...", current_page));
    emit_progress_event(app, &payload).map_err(|e| e.to_string())?;

    for (idx, map_with_order) in page.maps.iter().enumerate() {
        let map = convert_map_detail(&map_with_order.map);

        let diffs: Vec<MapDiff> = map_with_order.map.versions.first()
            .map(|version| version.diffs.iter().map(|diff| MapDiff {
                key: map_with_order.map.id.clone(),
                hash: version.hash.clone(),
                is_current: true,
                mode: diff.characteristic.clone(),
                difficulty: diff.difficulty.clone(),
            }).collect())
            .unwrap_or_default();

        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::save_playlist_map_entry(
            &guard,
            playlist_id as i64,
            &map,
            &diffs,
            map_with_order.order,
        ).map_err(|e| e.to_string())?;

        // 每处理完几个谱面更新一次进度（避免频繁更新）
        if idx % 10 == 0 || idx == page.maps.len() - 1 {
            let maps_in_current_page = page.maps.len();
            let current_progress = (current_page as f64 - 1.0) / total_pages as f64 * 100.0 +
                                   (idx as f64 + 1.0) / maps_in_current_page as f64 * (100.0 / total_pages as f64);

            let payload = ProgressPayload::new("add_playlist".to_string(), task_id.to_string())
                .with_progress("processing_maps".to_string(), current_progress.min(100.0))
                .with_message(format!("第 {} 页：正在处理第 {} 个谱面 (共 {} 个)...", current_page, idx + 1, maps_in_current_page));
            emit_progress_event(app, &payload).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// 获取歌单列表
#[tauri::command]
pub fn get_playlist_list(app: AppHandle) -> Result<Vec<Playlist>, String> {
    let state = app.try_state::<Mutex<rusqlite::Connection>>()
        .ok_or("Database not initialized")?;
    let guard = state.lock().map_err(|e| e.to_string())?;
    db::get_all_playlists(&guard).map_err(|e| e.to_string())
}

/// 获取单个歌单详情
#[tauri::command]
pub fn get_playlist(app: AppHandle, playlist_id: i64) -> Result<Option<Playlist>, String> {
    let state = app.try_state::<Mutex<rusqlite::Connection>>()
        .ok_or("Database not initialized")?;
    let guard = state.lock().map_err(|e| e.to_string())?;
    db::get_playlist_by_id(&guard, playlist_id).map_err(|e| e.to_string())
}

/// 删除歌单
#[tauri::command]
pub fn delete_playlist(app: AppHandle, playlist_id: i64) -> Result<(), String> {
    let state = app.try_state::<Mutex<rusqlite::Connection>>()
        .ok_or("Database not initialized")?;
    let guard = state.lock().map_err(|e| e.to_string())?;
    db::delete_playlist(&guard, playlist_id).map_err(|e| e.to_string())
}

/// 获取歌单中的谱面列表
#[tauri::command]
pub fn get_playlist_maps(app: AppHandle, playlist_id: i64) -> Result<Vec<PlaylistMap>, String> {
    let state = app.try_state::<Mutex<rusqlite::Connection>>()
        .ok_or("Database not initialized")?;
    let guard = state.lock().map_err(|e| e.to_string())?;
    db::get_playlist_maps(&guard, playlist_id).map_err(|e| e.to_string())
}

/// 将 API 响应转换为数据库结构
fn convert_playlist_response(page: &crate::api::PlaylistPage) -> Playlist {
    Playlist {
        playlist_id: page.playlist.playlist_id as i64,
        name: page.playlist.name.clone(),
        description: page.playlist.description.clone(),
        playlist_image: page.playlist.playlist_image.clone(),
        owner_id: Some(page.playlist.owner.id as i64),
        owner_name: Some(page.playlist.owner.name.clone()),
        total_maps: page.playlist.stats.as_ref().map(|s| s.total_maps),
        created_at: page.playlist.created_at.clone(),
        songs_changed_at: page.playlist.songs_changed_at.clone(),
        download_url: page.playlist.download_url.clone(),
    }
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
