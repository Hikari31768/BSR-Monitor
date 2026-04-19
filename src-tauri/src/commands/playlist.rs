use tauri::AppHandle;
use tauri::Manager;
use crate::api::BeatSaverApi;
use crate::database::operations::{Playlist, PlaylistMap, Map, MapDiff};
use crate::database::operations as db;
use std::sync::Mutex;
use uuid::Uuid;
use crate::progress::{ProgressPayload, emit_progress_event};

/// 添加歌单 - 支持进度反馈
#[tauri::command]
pub async fn add_playlist(app: AppHandle, playlist_id: i32) -> Result<Playlist, String> {
    let task_id = Uuid::new_v4().to_string();
    let api = BeatSaverApi::new();

    // 发送初始进度
    let payload = ProgressPayload::new("add_playlist".to_string(), task_id.clone())
        .with_progress("fetching_playlist".to_string(), 5.0)
        .with_message("正在获取歌单信息...".to_string());
    emit_progress_event(&app, &payload).map_err(|e| e.to_string())?;

    // 拉取第一页
    let first_page = api.get_playlist(playlist_id, 0)
        .await
        .map_err(|e| e.to_string())?;

    // 转换歌单元数据
    let playlist = convert_playlist_response(&first_page);

    // 发送进度 - 开始保存歌单信息
    let payload = ProgressPayload::new("add_playlist".to_string(), task_id.clone())
        .with_progress("saving_playlist_info".to_string(), 10.0)
        .with_message("正在保存歌单信息...".to_string());
    emit_progress_event(&app, &payload).map_err(|e| e.to_string())?;

    // 存入歌单
    {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::insert_playlist(&guard, &playlist).map_err(|e| e.to_string())?;
    }

    // 处理第一页的谱面
    save_playlist_maps_with_progress(&app, playlist_id, &first_page, &task_id, 1, 1)?;

    // 如果有更多页面，继续拉取
    let total_maps = first_page.playlist.stats.as_ref().map(|s| s.total_maps).unwrap_or(0);
    let total_pages = if total_maps > 0 { (total_maps + 49) / 50 } else { 1 };

    if total_pages > 1 {
        // 发送进度 - 开始处理后续页面
        let payload = ProgressPayload::new("add_playlist".to_string(), task_id.clone())
            .with_count("processing_pages".to_string(), 1, total_pages as u64)
            .with_message(format!("开始处理 {} 页谱面...", total_pages));
        emit_progress_event(&app, &payload).map_err(|e| e.to_string())?;

        for page in 1..total_pages {
            let page_response = api.get_playlist(playlist_id, page as i32)
                .await
                .map_err(|e| e.to_string())?;

            save_playlist_maps_with_progress(&app, playlist_id, &page_response, &task_id, page as u64 + 1, total_pages as u64)?;
        }
    }

    // 发送完成进度
    let payload = ProgressPayload::new("add_playlist".to_string(), task_id.clone())
        .with_progress("completed".to_string(), 100.0)
        .with_message(format!("完成！共处理 {} 首歌曲", total_maps));
    emit_progress_event(&app, &payload).map_err(|e| e.to_string())?;

    Ok(playlist)
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
        // 存入谱面信息
        let map = convert_map_detail(&map_with_order.map);
        {
            let state = app.try_state::<Mutex<rusqlite::Connection>>()
                .ok_or("Database not initialized")?;
            let guard = state.lock().map_err(|e| e.to_string())?;
            db::insert_map(&guard, &map).map_err(|e| e.to_string())?;
        }

        // 存入难度信息
        if let Some(version) = map_with_order.map.versions.first() {
            for diff in &version.diffs {
                let map_diff = MapDiff {
                    key: map_with_order.map.id.clone(),
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

        // 存入关联关系
        let playlist_map = PlaylistMap {
            playlist_id: playlist_id as i64,
            map_key: map_with_order.map.id.clone(),
            map_order: map_with_order.order,
        };
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::insert_playlist_map(&guard, &playlist_map).map_err(|e| e.to_string())?;

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