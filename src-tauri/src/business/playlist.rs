use crate::api::{BeatSaverApi, PlaylistPage};
use crate::database::operations::{Playlist, PlaylistMap};
use crate::database::operations as db;
use crate::business::map::save_map_detail;
use crate::business::BusinessError;
use rusqlite::Connection;

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
    // 从第一页开始拉取
    let first_page = api.get_playlist(playlist_id, 0).await?;

    // 存入歌单元数据
    let playlist = convert_playlist_response(&first_page);
    db::insert_playlist(conn, &playlist)?;

    // 处理第一页的谱面
    save_playlist_maps(conn, playlist_id, &first_page)?;

    // 如果有更多页面，继续拉取
    let total_pages = first_page.playlist.stats.map(|s| s.total_maps).unwrap_or(0) / 50 + 1;
    for page in 1..total_pages {
        let page_response = api.get_playlist(playlist_id, page).await?;
        save_playlist_maps(conn, playlist_id, &page_response)?;
    }

    Ok(playlist)
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
        };
        db::insert_playlist_map(conn, &playlist_map)?;
    }
    Ok(())
}

/// 将 API 响应转换为数据库结构
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