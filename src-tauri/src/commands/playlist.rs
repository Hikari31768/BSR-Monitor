use tauri::AppHandle;
use tauri::Manager;
use crate::api::BeatSaverApi;
use crate::database::operations::{Playlist, PlaylistMap, Map, MapDiff};
use crate::database::operations as db;
use std::sync::Mutex;

/// 添加歌单
#[tauri::command]
pub async fn add_playlist(app: AppHandle, playlist_id: i32) -> Result<Playlist, String> {
    let api = BeatSaverApi::new();

    // 拉取第一页
    let first_page = api.get_playlist(playlist_id, 0)
        .await
        .map_err(|e| e.to_string())?;

    // 转换歌单元数据
    let playlist = convert_playlist_response(&first_page);

    // 存入歌单
    {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::insert_playlist(&guard, &playlist).map_err(|e| e.to_string())?;
    }

    // 处理第一页的谱面
    save_playlist_maps(&app, playlist_id, &first_page)?;

    // 如果有更多页面，继续拉取
    let total_pages = first_page.playlist.stats.map(|s| s.total_maps).unwrap_or(0) / 50 + 1;
    for page in 1..total_pages {
        let page_response = api.get_playlist(playlist_id, page)
            .await
            .map_err(|e| e.to_string())?;
        save_playlist_maps(&app, playlist_id, &page_response)?;
    }

    Ok(playlist)
}

/// 处理歌单页面中的谱面
fn save_playlist_maps(app: &AppHandle, playlist_id: i32, page: &crate::api::PlaylistPage) -> Result<(), String> {
    for map_with_order in &page.maps {
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