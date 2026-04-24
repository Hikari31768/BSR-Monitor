use crate::api::MapDetail;
use crate::database::operations::Map;

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
