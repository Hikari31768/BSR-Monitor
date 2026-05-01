use tauri::AppHandle;
use tauri::Manager;
use tauri::State;
use crate::api::BeatSaverApi;
use crate::database::operations::{Playlist, PlaylistMap, MapDiff};
use crate::database::operations as db;
use crate::commands::common::convert_map_detail;
use std::sync::Mutex;
use uuid::Uuid;
use crate::progress::{ProgressPayload, TaskType, emit_progress_event};
use crate::task_state::TaskState;
use serde::Serialize;

/// Response from add_playlist command
#[derive(Serialize)]
pub struct AddPlaylistResult {
    pub task_id: String,
    pub playlist: Playlist,
}

/// 添加歌单或增量更新（如已存在）- 支持进度反馈
#[tauri::command]
pub async fn add_playlist(
    app: AppHandle,
    task_state: State<'_, TaskState>,
    playlist_id: i32,
    task_id: Option<String>,
) -> Result<AddPlaylistResult, String> {
    let task_id = task_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let api = BeatSaverApi::new().map_err(|e| e.to_string())?;

    task_state.register(&task_id, "add_playlist", &playlist_id.to_string());

    // 检查是否已存在
    let existing = {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::get_playlist_by_id(&guard, playlist_id as i64).map_err(|e| e.to_string())?
    };

    let delete_first = existing.is_some();
    let result = process_playlist(&app, &task_state, playlist_id, task_id.clone(), delete_first, api).await;

    // 无论成功失败都移除任务记录
    task_state.remove(&task_id);

    result.map(|playlist| AddPlaylistResult { task_id, playlist })
}

const PLAYLIST_PAGE_SIZE: i32 = 500;

/// 统一处理歌单（添加或更新）
async fn process_playlist(
    app: &AppHandle,
    task_state: &TaskState,
    playlist_id: i32,
    task_id: String,
    delete_first: bool,
    api: BeatSaverApi,
) -> Result<Playlist, String> {
    let message_prefix = if delete_first { "更新" } else { "" };
    let progress_key = if delete_first { "updating" } else { "adding" };

    // 发送初始进度
    let payload = ProgressPayload::new(TaskType::AddPlaylist, task_id.clone())
        .with_progress(format!("{}_playlist", progress_key), 2.0)
        .with_message(format!("{}{}正在获取歌单信息...", message_prefix, if delete_first { "" } else { "正在" }));
    emit_progress_event(app, &payload).map_err(|e| e.to_string())?;
    task_state.update(&task_id, 2.0, &format!("{}{}正在获取歌单信息...", message_prefix, if delete_first { "" } else { "正在" }));

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
    let msg = format!("已获取歌单: {}", playlist.name);
    let payload = ProgressPayload::new(TaskType::AddPlaylist, task_id.clone())
        .with_progress("saving_playlist_info".to_string(), 5.0)
        .with_message(msg.clone());
    emit_progress_event(app, &payload).map_err(|e| e.to_string())?;
    task_state.update(&task_id, 5.0, &msg);

    // 存入歌单
    {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::insert_playlist(&guard, &playlist).map_err(|e| e.to_string())?;
    }

    // 如果是更新，先删除旧谱面关联
    if delete_first {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::delete_playlist_maps(&guard, playlist_id as i64).map_err(|e| e.to_string())?;
    }

    // 处理第一页的谱面
    let mut total_actual_count = first_page.maps.len();
    save_playlist_maps_with_progress(app, task_state, &task_id, playlist_id, &first_page, 1, 1)?;

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
        save_playlist_maps_with_progress(app, task_state, &task_id, playlist_id, &page_response, current_page as u64 + 1, current_page as u64 + 1)?;
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

    // 后台拉取评级数据（stars, passRating, accRating, techRating）
    println!("[Playlist] Spawning background rating fetch...");
    let rating_task_id = format!("rating-{}", Uuid::new_v4());
    let rating_display = format!("{} - 拉取评级", playlist.name);
    task_state.register_with_name(&rating_task_id, "fetch_ratings", &playlist_id.to_string(), &rating_display);

    // 收集需要拉取评级的 hash
    let all_hashes: Vec<String> = {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::get_playlist_maps(&guard, playlist_id as i64)
            .map_err(|e| e.to_string())?
            .into_iter()
            .filter_map(|pm| pm.map)
            .flat_map(|m| {
                db::get_map_diffs_by_key(&guard, &m.key)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|d| d.hash)
            })
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    };

    // 在后台任务中拉取评级
    let app_clone = app.clone();
    let rating_task_id_clone = rating_task_id.clone();
    tokio::spawn(async move {
        if all_hashes.is_empty() {
            return;
        }

        // 通过 app 获取 TaskState，避免生命周期问题
        let task_state = match app_clone.try_state::<crate::task_state::TaskState>() {
            Some(s) => s,
            None => return,
        };

        let total = all_hashes.len() as u64;
        task_state.update(&rating_task_id_clone, 0.0, &format!("正在拉取评级 (0/{})", total));

        let api = match crate::api::BeatLeaderApi::new() {
            Ok(a) => a,
            Err(_) => return,
        };

        for (idx, hash) in all_hashes.iter().enumerate() {
            match api.get_leaderboards_by_hash(hash).await {
                Ok(response) => {
                    let state = app_clone.try_state::<Mutex<rusqlite::Connection>>();
                    if let Some(state) = state {
                        if let Ok(guard) = state.lock() {
                            for entry in &response.leaderboards {
                                let diff = &entry.difficulty;
                                let mode_name = if diff.mode_name.is_empty() {
                                    "Standard".to_string()
                                } else {
                                    diff.mode_name.clone()
                                };
                                let difficulty_name = if diff.difficulty_name.is_empty() {
                                    crate::database::operations::difficulty_from_value(diff.value)
                                } else {
                                    diff.difficulty_name.clone()
                                };
                                let _ = db::update_diff_ratings(
                                    &guard, hash, &mode_name, &difficulty_name,
                                    diff.stars,
                                    diff.pass_rating,
                                    diff.acc_rating,
                                    diff.tech_rating,
                                );
                            }
                        }
                    }
                }
                Err(_) => continue,
            }

            let current = (idx + 1) as u64;
            let progress = (current as f64 / total as f64) * 100.0;
            task_state.update(&rating_task_id_clone, progress, &format!("正在拉取评级 ({}/{})", current, total));

            // 同时发射进度事件给前端
            let payload = ProgressPayload::new(TaskType::FetchRatings, rating_task_id_clone.clone())
                .with_progress("fetching_ratings".to_string(), progress)
                .with_message(format!("正在拉取评级 ({}/{})", current, total));
            let _ = emit_progress_event(&app_clone, &payload);
        }

        task_state.update(&rating_task_id_clone, 100.0, "评级拉取完成");
        let payload = ProgressPayload::new(TaskType::FetchRatings, rating_task_id_clone.clone())
            .with_progress("completed".to_string(), 100.0)
            .with_message("评级拉取完成".to_string());
        let _ = emit_progress_event(&app_clone, &payload);

        // 清理任务状态（前端已通过 completed 事件自动隐藏，此处清理后端状态以防重启恢复）
        task_state.remove(&rating_task_id_clone);
    });

    // 发送完成进度
    let final_total = if is_search_type { total_actual_count as i32 } else { playlist.total_maps.unwrap_or(0) };
    let msg = format!("{}完成！共处理 {} 首歌曲", if delete_first { "更新" } else { "" }, final_total);
    let payload = ProgressPayload::new(TaskType::AddPlaylist, task_id.clone())
        .with_progress("completed".to_string(), 100.0)
        .with_message(msg.clone());
    emit_progress_event(app, &payload).map_err(|e| e.to_string())?;
    task_state.update(&task_id, 100.0, &msg);

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
    task_state: &TaskState,
    task_id: &str,
    playlist_id: i32,
    page: &crate::api::PlaylistPage,
    current_page: u64,
    total_pages: u64
) -> Result<(), String> {
    // 发送当前页进度（从8%开始）
    let payload = ProgressPayload::new(TaskType::AddPlaylist, task_id.to_string())
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
                stars_rating: diff.stars,
                stars_fetched_at: None,
                pass_rating: None,
                acc_rating: None,
                tech_rating: None,
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
        // 进度区间：8%-100%（谱面处理占92%）
        if idx % 10 == 0 || idx == page.maps.len() - 1 {
            let maps_in_current_page = page.maps.len();
            let base_progress = 8.0;
            let progress_range = 92.0;
            let current_progress = base_progress +
                (current_page as f64 - 1.0) / total_pages as f64 * progress_range +
                (idx as f64 + 1.0) / maps_in_current_page as f64 * (progress_range / total_pages as f64);
            let progress = current_progress.min(100.0);

            let payload = ProgressPayload::new(TaskType::AddPlaylist, task_id.to_string())
                .with_progress("processing_maps".to_string(), progress)
                .with_message(format!("正在拉取第 {current_page} 页，处理 {} / {} 谱面...", idx + 1, maps_in_current_page));
            emit_progress_event(app, &payload).map_err(|e| e.to_string())?;
            task_state.update(task_id, progress, &format!("正在拉取第 {current_page} 页，处理 {} / {} 谱面...", idx + 1, maps_in_current_page));
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
pub fn delete_playlist(app: AppHandle, task_state: State<'_, TaskState>, playlist_id: i64) -> Result<(), String> {
    // 检查是否有正在进行的添加任务
    if task_state.has_active_for_target(&playlist_id.to_string()) {
        return Err("无法删除正在处理中的歌单".to_string());
    }
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
