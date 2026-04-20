use crate::api::{BeatSaverApi, PlaylistPage, PlaylistType};
use crate::database::operations::{Playlist, PlaylistMap};
use crate::database::operations as db;
use crate::business::map::save_map_detail;
use crate::business::BusinessError;
use rusqlite::Connection;

const PLAYLIST_PAGE_SIZE: i32 = 500;

/// 添加歌单
///
/// 1. 调用 API 获取歌单信息
/// 2. 存入数据库
/// 3. 批量填充谱面信息
pub async fn add_playlist(
    conn: &Connection,
    api: &BeatSaverApi,
    playlist_id: i32,
) -> Result<Playlist, BusinessError> {
    // 先存入歌单元数据（使用 API 返回的 total_maps）
    let first_page = api.get_playlist(playlist_id, 0).await?;
    println!("[Playlist] First page: {} maps, type: {:?}, API total: {}",
        first_page.maps.len(),
        first_page.playlist.playlist_type,
        first_page.playlist.stats.as_ref().map(|s| s.total_maps).unwrap_or(0));

    let initial_playlist = convert_playlist_response(&first_page);
    db::insert_playlist(conn, &initial_playlist)?;

    // 处理第一页的谱面
    let mut total_actual_count = first_page.maps.len();
    save_playlist_maps(conn, playlist_id, &first_page)?;

    // 判断是否为 Search 类型
    let is_search_type = first_page.playlist.playlist_type == Some(PlaylistType::Search);

    // 根据实际返回数量判断是否需要继续拉取
    // 如果当前页满页（500条），可能还有更多数据
    let mut current_page = 0;
    let mut current_page_count = first_page.maps.len();
    while current_page_count == PLAYLIST_PAGE_SIZE as usize {
        current_page += 1;
        println!("[Playlist] Fetching page {}...", current_page);
        let page_response = api.get_playlist(playlist_id, current_page).await?;

        current_page_count = page_response.maps.len();
        println!("[Playlist] Page {} returned {} maps", current_page, current_page_count);

        if current_page_count == 0 {
            break;
        }

        total_actual_count += current_page_count;
        save_playlist_maps(conn, playlist_id, &page_response)?;
    }

    println!("[Playlist] Total fetched: {} maps, is_search: {}", total_actual_count, is_search_type);

    // 对于 Search 类型歌单，更新 total_maps 为实际数量
    if is_search_type {
        println!("[Playlist] Updating total_maps to {}", total_actual_count);
        db::update_playlist_total_maps(conn, playlist_id as i64, Some(total_actual_count as i32))?;
    }

    // 返回最终的歌单信息
    let final_playlist = db::get_playlist_by_id(conn, playlist_id as i64)?
        .unwrap_or(initial_playlist);

    Ok(final_playlist)
}

/// 获取歌单列表
pub fn get_playlist_list(conn: &Connection) -> Result<Vec<Playlist>, BusinessError> {
    db::get_all_playlists(conn).map_err(BusinessError::from)
}

/// 获取单个歌单
pub fn get_playlist(conn: &Connection, playlist_id: i64) -> Result<Option<Playlist>, BusinessError> {
    db::get_playlist_by_id(conn, playlist_id).map_err(BusinessError::from)
}

/// 删除歌单
pub fn delete_playlist(conn: &Connection, playlist_id: i64) -> Result<(), BusinessError> {
    db::delete_playlist(conn, playlist_id).map_err(BusinessError::from)
}

/// 获取歌单中的谱面列表
pub fn get_playlist_maps(conn: &Connection, playlist_id: i64) -> Result<Vec<PlaylistMap>, BusinessError> {
    db::get_playlist_maps(conn, playlist_id).map_err(BusinessError::from)
}

/// 处理歌单页面中的谱面
fn save_playlist_maps(conn: &Connection, playlist_id: i32, page: &PlaylistPage) -> Result<(), BusinessError> {
    for map_with_order in &page.maps {
        // 存入谱面信息
        save_map_detail(conn, &map_with_order.map)?;

        // 存入关联关系
        let playlist_map = PlaylistMap {
            playlist_id: playlist_id as i64,
            map_key: map_with_order.map.id.clone(),
            map_order: map_with_order.order,
            map: None,
        };
        db::insert_playlist_map(conn, &playlist_map)?;
    }
    Ok(())
}

/// 将 API 响应转换为数据库结构（使用 API 返回的 total_maps）
fn convert_playlist_response(page: &PlaylistPage) -> Playlist {
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