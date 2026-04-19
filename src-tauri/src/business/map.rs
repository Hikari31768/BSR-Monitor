use crate::api::{BeatSaverApi, MapDetail, MapVersion};
use crate::database::operations::{Map, MapDiff};
use crate::database::operations as db;
use crate::business::BusinessError;
use rusqlite::Connection;

/// 通过 key 获取谱面信息
///
/// 1. 调用 API 获取谱面
/// 2. 存入数据库
/// 3. 返回谱面详情
pub async fn get_map_by_key(
    conn: &Connection,
    api: &BeatSaverApi,
    key: &str,
) -> Result<Map, BusinessError> {
    // 先检查数据库
    if let Some(map) = db::get_map_by_key(conn, key)? {
        return Ok(map);
    }

    // 调用 API 获取谱面
    let map_detail = api.get_map_by_key(key).await?;

    // 存入数据库
    save_map_detail(conn, &map_detail)?;

    // 返回转换后的 Map 结构
    Ok(convert_map_detail(&map_detail))
}

/// 通过 hash 获取谱面信息
pub async fn get_map_by_hash(
    conn: &Connection,
    api: &BeatSaverApi,
    hash: &str,
) -> Result<Map, BusinessError> {
    // 调用 API 获取谱面
    let map_detail = api.get_map_by_hash(hash).await?;

    // 存入数据库
    save_map_detail(conn, &map_detail)?;

    // 返回转换后的 Map 结构
    Ok(convert_map_detail(&map_detail))
}

/// 从数据库获取谱面（不调用 API）
pub fn get_map_from_db(conn: &Connection, key: &str) -> Result<Option<Map>, BusinessError> {
    db::get_map_by_key(conn, key).map_err(BusinessError::from)
}

/// 获取谱面的难度列表
pub fn get_map_diffs(conn: &Connection, key: &str) -> Result<Vec<MapDiff>, BusinessError> {
    db::get_map_diffs_by_key(conn, key).map_err(BusinessError::from)
}

/// 将 API 响应的谱面详情存入数据库（公开函数，供其他模块调用）
pub fn save_map_detail(conn: &Connection, map_detail: &MapDetail) -> Result<(), BusinessError> {
    // 存入 Map 表
    let map = convert_map_detail(map_detail);
    db::insert_map(conn, &map)?;

    // 存入 MapDiff 表
    // BeatSaver versions 数组只包含当前版本，取 versions[0]
    if let Some(version) = map_detail.versions.first() {
        save_map_version(conn, &map_detail.id, version, true)?;
    }

    Ok(())
}

/// 保存谱面版本信息
fn save_map_version(conn: &Connection, key: &str, version: &MapVersion, is_current: bool) -> Result<(), BusinessError> {
    let diffs: Vec<MapDiff> = version.diffs.iter().map(|diff| MapDiff {
        key: key.to_string(),
        hash: version.hash.clone(),
        is_current,
        mode: diff.characteristic.clone(),
        difficulty: diff.difficulty.clone(),
    }).collect();

    db::insert_map_diffs_batch(conn, key, &diffs)?;
    Ok(())
}

/// 将 API 响应转换为数据库结构
fn convert_map_detail(map_detail: &MapDetail) -> Map {
    // 获取封面和预览链接（取第一个版本）
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