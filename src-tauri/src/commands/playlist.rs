use tauri::AppHandle;
use tauri::Manager;
use tauri::State;
use crate::api::BeatSaverApi;
use crate::database::operations::{Playlist, PlaylistSource, BplistFile, BplistSong, MapDiff, Map};
use crate::database::operations as db;
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

/// 添加歌单 - 支持三种来源
#[tauri::command]
pub async fn add_playlist(
    app: AppHandle,
    task_state: State<'_, TaskState>,
    source: String,
    identifier: String,
    task_id: Option<String>,
) -> Result<AddPlaylistResult, String> {
    let task_id = task_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let api = BeatSaverApi::new().map_err(|e| e.to_string())?;

    let playlist_source = match source.as_str() {
        "beatsaver" => PlaylistSource::Beatsaver,
        "mapper" => PlaylistSource::Mapper,
        "local" => PlaylistSource::Local,
        _ => return Err(format!("Invalid playlist source: {}", source)),
    };

    // 检查是否已存在
    let existing = {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::get_playlist(&guard, &playlist_source, &identifier).map_err(|e| e.to_string())?
    };

    if existing.is_some() {
        return Err(format!("Playlist already exists: {}/{}", source, identifier));
    }

    task_state.register(&task_id, "add_playlist", &format!("{}/{}", source, identifier));

    let result = process_playlist(&app, &task_state, &playlist_source, &identifier, task_id.clone(), api).await;

    task_state.remove(&task_id);

    result.map(|playlist| AddPlaylistResult { task_id, playlist })
}

/// 刷新歌单
#[tauri::command]
pub async fn refresh_playlist(
    app: AppHandle,
    task_state: State<'_, TaskState>,
    source: String,
    identifier: String,
    task_id: Option<String>,
) -> Result<AddPlaylistResult, String> {
    let task_id = task_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let api = BeatSaverApi::new().map_err(|e| e.to_string())?;

    let playlist_source = match source.as_str() {
        "beatsaver" => PlaylistSource::Beatsaver,
        "mapper" => PlaylistSource::Mapper,
        "local" => PlaylistSource::Local,
        _ => return Err(format!("Invalid playlist source: {}", source)),
    };

    task_state.register(&task_id, "refresh_playlist", &format!("{}/{}", source, identifier));

    let result = process_playlist(&app, &task_state, &playlist_source, &identifier, task_id.clone(), api).await;

    task_state.remove(&task_id);

    result.map(|playlist| AddPlaylistResult { task_id, playlist })
}

/// 统一处理歌单（添加或更新）
async fn process_playlist(
    app: &AppHandle,
    task_state: &TaskState,
    source: &PlaylistSource,
    identifier: &str,
    task_id: String,
    api: BeatSaverApi,
) -> Result<Playlist, String> {
    let is_update = {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::get_playlist(&guard, source, identifier).map_err(|e| e.to_string())?.is_some()
    };

    let action_prefix = if is_update { "更新" } else { "正在添加" };

    // 发送初始进度
    let payload = ProgressPayload::new(TaskType::AddPlaylist, task_id.clone())
        .with_progress("fetching_bplist".to_string(), 2.0)
        .with_message(format!("{}正在获取歌单信息...", action_prefix));
    emit_progress_event(app, &payload).map_err(|e| e.to_string())?;
    task_state.update(&task_id, 2.0, &format!("{}正在获取歌单信息...", action_prefix));

    // 1. 获取 .bplist 内容并保存到 Playlists 目录
    let (bplist_content, bplist_path) = fetch_and_save_bplist(app, source, identifier, &api, &task_id).await?;

    // 发送进度
    let payload = ProgressPayload::new(TaskType::AddPlaylist, task_id.clone())
        .with_progress("parsing_bplist".to_string(), 10.0)
        .with_message("正在解析歌单文件...".to_string());
    emit_progress_event(app, &payload).map_err(|e| e.to_string())?;
    task_state.update(&task_id, 10.0, "正在解析歌单文件...");

    // 2. 解析 .bplist
    let bplist: BplistFile = serde_json::from_str(&bplist_content)
        .map_err(|e| format!("Failed to parse bplist: {}", e))?;

    let song_count = bplist.songs.len();

    // 3. 保存歌单元信息到 Playlist 表
    let playlist = Playlist {
        source: source.clone(),
        identifier: identifier.to_string(),
        name: bplist.playlist_title.clone(),
        description: None,
        playlist_image: bplist.image.clone().filter(|s| !s.is_empty()),
        owner_id: match source {
            PlaylistSource::Mapper => Some(identifier.to_string()),
            _ => None,
        },
        owner_name: Some(bplist.playlist_author.clone()),
        total_maps: Some(song_count as i32),
        created_at: None,
        songs_changed_at: None,
        download_url: bplist.custom_data.as_ref().and_then(|c| c.sync_url.clone()),
        local_path: Some(bplist_path.clone()),
        last_bplist_fetched_at: Some(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64),
    };

    {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        db::insert_playlist(&guard, &playlist).map_err(|e| e.to_string())?;
    }

    let payload = ProgressPayload::new(TaskType::AddPlaylist, task_id.clone())
        .with_progress("saving_playlist_info".to_string(), 15.0)
        .with_message(format!("已获取歌单: {}", playlist.name));
    emit_progress_event(app, &payload).map_err(|e| e.to_string())?;
    task_state.update(&task_id, 15.0, &format!("已获取歌单: {}", playlist.name));

    // 4. 后台任务：对每个 hash 调用 BeatLeader API 获取详细信息
    let hashes: Vec<String> = bplist.songs.iter().map(|s| s.hash.clone()).collect();
    spawn_rating_fetch(app, &hashes, &playlist.name);

    // 发送完成进度
    let msg = format!("{}完成！共 {} 首歌曲", if is_update { "更新" } else { "添加" }, song_count);
    let payload = ProgressPayload::new(TaskType::AddPlaylist, task_id.clone())
        .with_progress("completed".to_string(), 100.0)
        .with_message(msg.clone());
    emit_progress_event(app, &payload).map_err(|e| e.to_string())?;
    task_state.update(&task_id, 100.0, &msg);

    Ok(playlist)
}

/// 获取并保存 .bplist 文件
async fn fetch_and_save_bplist(
    app: &AppHandle,
    source: &PlaylistSource,
    identifier: &str,
    api: &BeatSaverApi,
    _task_id: &str,
) -> Result<(String, String), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let playlists_dir = app_data_dir.join("Playlists");
    std::fs::create_dir_all(&playlists_dir).map_err(|e| e.to_string())?;

    let (bplist_content, filename) = match source {
        PlaylistSource::Beatsaver => {
            // GET /playlists/id/{id} → downloadURL
            let page = api.get_playlist(identifier.parse().map_err(|e| format!("Invalid playlist ID: {}", e))?, 0)
                .await
                .map_err(|e| format!("Failed to fetch playlist from BeatSaver: {}", e))?;

            let download_url = page.playlist.download_url
                .ok_or_else(|| "BeatSaver playlist has no downloadURL".to_string())?;

            let client = reqwest::Client::new();
            let response = client.get(&download_url).send().await
                .map_err(|e| format!("Failed to download bplist: {}", e))?;
            let content = response.text().await
                .map_err(|e| format!("Failed to read bplist content: {}", e))?;

            (content, format!("beatsaver_{}.bplist", identifier))
        }
        PlaylistSource::Mapper => {
            // GET /users/id/{id} → playlistUrl
            let user = api.get_user_by_id(identifier).await
                .map_err(|e| format!("Failed to fetch user from BeatSaver: {}", e))?;

            let playlist_url = user.playlist_url
                .ok_or_else(|| format!("User '{}' has no playlistUrl", identifier))?;

            let client = reqwest::Client::new();
            let response = client.get(&playlist_url).send().await
                .map_err(|e| format!("Failed to download bplist: {}", e))?;
            let content = response.text().await
                .map_err(|e| format!("Failed to read bplist content: {}", e))?;

            (content, format!("mapper_{}.bplist", identifier))
        }
        PlaylistSource::Local => {
            // identifier = 原始文件的绝对路径
            let original_path = identifier;
            let content = std::fs::read_to_string(original_path)
                .map_err(|e| format!("Failed to read local bplist file: {}", e))?;

            // 计算原始路径的 hash 作为文件名
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            original_path.hash(&mut hasher);
            let hash = hasher.finish();

            (content, format!("local_{:x}.bplist", hash))
        }
    };

    let target_path = playlists_dir.join(&filename);
    std::fs::write(&target_path, &bplist_content)
        .map_err(|e| format!("Failed to save bplist file: {}", e))?;

    Ok((bplist_content, target_path.to_string_lossy().to_string()))
}

/// 后台拉取评级数据
fn spawn_rating_fetch(
    app: &AppHandle,
    hashes: &[String],
    playlist_name: &str,
) {
    if hashes.is_empty() {
        return;
    }

    let rating_task_id = format!("rating-{}", Uuid::new_v4());
    let rating_display = format!("{} - 拉取评级", playlist_name);

    // Register via app state
    if let Some(ts) = app.try_state::<crate::task_state::TaskState>() {
        ts.register_with_name(&rating_task_id, "fetch_ratings", "rating", &rating_display);
    }

    let hashes = hashes.to_vec();
    let app_clone = app.clone();
    let rating_task_id_clone = rating_task_id.clone();

    tokio::spawn(async move {
        let ts = match app_clone.try_state::<crate::task_state::TaskState>() {
            Some(s) => s,
            None => return,
        };

        let total = hashes.len() as u64;
        ts.update(&rating_task_id_clone, 0.0, &format!("正在拉取评级 (0/{})", total));

        let api = match crate::api::BeatLeaderApi::new() {
            Ok(a) => a,
            Err(_) => return,
        };

        for (idx, hash) in hashes.iter().enumerate() {
            match api.get_leaderboards_by_hash(hash).await {
                Ok(response) => {
                    // 更新 Map 表
                    let key = response.song.id.trim_end_matches('x').to_string();
                    let map = Map {
                        key: key.clone(),
                        song_name: response.song.name.clone(),
                        song_author_name: response.song.author.clone(),
                        level_author_name: response.song.mapper.clone(),
                        rating: None,
                        cover_url: response.song.cover_image.clone(),
                        preview_url: None,
                    };

                    // 构建 MapDiff 列表
                    let diffs: Vec<MapDiff> = response.leaderboards.iter().map(|entry| {
                        let diff = &entry.difficulty;
                        let mode_name = if diff.mode_name.is_empty() {
                            "Standard".to_string()
                        } else {
                            diff.mode_name.clone()
                        };
                        let difficulty_name = if diff.difficulty_name.is_empty() {
                            db::difficulty_from_value(diff.value)
                        } else {
                            diff.difficulty_name.clone()
                        };

                        MapDiff {
                            key: key.clone(),
                            hash: hash.clone(),
                            is_current: true,
                            mode: mode_name,
                            difficulty: difficulty_name,
                            stars_rating: diff.stars,
                            stars_fetched_at: Some(std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs() as i64),
                            pass_rating: diff.pass_rating,
                            acc_rating: diff.acc_rating,
                            tech_rating: diff.tech_rating,
                        }
                    }).collect();

                    if let Some(state) = app_clone.try_state::<Mutex<rusqlite::Connection>>() {
                        if let Ok(guard) = state.lock() {
                            let _ = db::insert_map(&guard, &map);
                            if !diffs.is_empty() {
                                let _ = db::insert_map_diffs_batch(&guard, &key, &diffs);
                            }
                        }
                    }
                }
                Err(_) => continue,
            }

            let current = (idx + 1) as u64;
            let progress = (current as f64 / total as f64) * 100.0;
            ts.update(&rating_task_id_clone, progress, &format!("正在拉取评级 ({}/{})", current, total));

            let payload = ProgressPayload::new(TaskType::FetchRatings, rating_task_id_clone.clone())
                .with_progress("fetching_ratings".to_string(), progress)
                .with_message(format!("正在拉取评级 ({}/{})", current, total));
            let _ = emit_progress_event(&app_clone, &payload);
        }

        ts.update(&rating_task_id_clone, 100.0, "评级拉取完成");
        let payload = ProgressPayload::new(TaskType::FetchRatings, rating_task_id_clone.clone())
            .with_progress("completed".to_string(), 100.0)
            .with_message("评级拉取完成".to_string());
        let _ = emit_progress_event(&app_clone, &payload);

        if let Some(ts) = app_clone.try_state::<crate::task_state::TaskState>() {
            ts.remove(&rating_task_id_clone);
        }
    });
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
pub fn get_playlist(app: AppHandle, source: String, identifier: String) -> Result<Option<Playlist>, String> {
    let playlist_source = match source.as_str() {
        "beatsaver" => PlaylistSource::Beatsaver,
        "mapper" => PlaylistSource::Mapper,
        "local" => PlaylistSource::Local,
        _ => return Err(format!("Invalid playlist source: {}", source)),
    };

    let state = app.try_state::<Mutex<rusqlite::Connection>>()
        .ok_or("Database not initialized")?;
    let guard = state.lock().map_err(|e| e.to_string())?;
    db::get_playlist(&guard, &playlist_source, &identifier).map_err(|e| e.to_string())
}

/// 删除歌单
#[tauri::command]
pub fn delete_playlist(app: AppHandle, task_state: State<'_, TaskState>, source: String, identifier: String) -> Result<(), String> {
    let playlist_source = match source.as_str() {
        "beatsaver" => PlaylistSource::Beatsaver,
        "mapper" => PlaylistSource::Mapper,
        "local" => PlaylistSource::Local,
        _ => return Err(format!("Invalid playlist source: {}", source)),
    };

    if task_state.has_active_for_target(&format!("{}/{}", source, identifier)) {
        return Err("无法删除正在处理中的歌单".to_string());
    }

    // 获取歌单信息以获取 localPath
    let state = app.try_state::<Mutex<rusqlite::Connection>>()
        .ok_or("Database not initialized")?;
    let guard = state.lock().map_err(|e| e.to_string())?;
    let playlist = db::get_playlist(&guard, &playlist_source, &identifier).map_err(|e| e.to_string())?;

    if let Some(p) = playlist {
        // 删除数据库记录
        db::delete_playlist(&guard, &playlist_source, &identifier).map_err(|e| e.to_string())?;

        // 清理 Playlists 目录下的副本文件
        if let Some(local_path) = p.local_path {
            let _ = std::fs::remove_file(&local_path);
        }
    }

    Ok(())
}

/// 获取歌单中的谱面列表（从 .bplist 文件读取）
#[tauri::command]
pub fn get_playlist_maps(app: AppHandle, source: String, identifier: String) -> Result<Vec<BplistSong>, String> {
    let playlist_source = match source.as_str() {
        "beatsaver" => PlaylistSource::Beatsaver,
        "mapper" => PlaylistSource::Mapper,
        "local" => PlaylistSource::Local,
        _ => return Err(format!("Invalid playlist source: {}", source)),
    };

    let state = app.try_state::<Mutex<rusqlite::Connection>>()
        .ok_or("Database not initialized")?;
    let guard = state.lock().map_err(|e| e.to_string())?;

    // 读取 bplist 获取完整的 songs 信息
    let playlist = db::get_playlist(&guard, &playlist_source, &identifier)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Playlist not found".to_string())?;

    let bplist_path = playlist.local_path
        .ok_or_else(|| "Playlist has no local bplist path".to_string())?;

    let content = std::fs::read_to_string(&bplist_path)
        .map_err(|e| format!("Failed to read bplist file: {}", e))?;
    let bplist: BplistFile = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse bplist: {}", e))?;

    Ok(bplist.songs)
}

/// 手动刷新歌单全部谱面星级（覆盖已有数据）
#[tauri::command]
pub async fn refresh_playlist_ratings(
    app: AppHandle,
    task_state: State<'_, TaskState>,
    source: String,
    identifier: String,
) -> Result<String, String> {
    let playlist_source = match source.as_str() {
        "beatsaver" => PlaylistSource::Beatsaver,
        "mapper" => PlaylistSource::Mapper,
        "local" => PlaylistSource::Local,
        _ => return Err(format!("Invalid playlist source: {}", source)),
    };

    let task_id = format!("refresh-ratings-{}", Uuid::new_v4());
    let api = crate::api::BeatLeaderApi::new().map_err(|e| e.to_string())?;

    // 获取歌单名称和 hash 列表
    let (playlist_name, all_hashes) = {
        let state = app.try_state::<Mutex<rusqlite::Connection>>()
            .ok_or("Database not initialized")?;
        let guard = state.lock().map_err(|e| e.to_string())?;
        let playlist = db::get_playlist(&guard, &playlist_source, &identifier)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Playlist not found".to_string())?;

        let hashes = db::get_playlist_hashes(&guard, &playlist_source, &identifier)?;
        (playlist.name, hashes)
    };

    if all_hashes.is_empty() {
        task_state.remove(&task_id);
        return Err("歌单中没有谱面".to_string());
    }

    let display_name = format!("{} - 更新星级", playlist_name);
    task_state.register_with_name(&task_id, "refresh_ratings", &format!("{}/{}", source, identifier), &display_name);

    let total = all_hashes.len() as u64;
    task_state.update(&task_id, 0.0, &format!("正在拉取星级 (0/{})", total));

    for (idx, hash) in all_hashes.iter().enumerate() {
        match api.get_leaderboards_by_hash(hash).await {
            Ok(response) => {
                let state = app.try_state::<Mutex<rusqlite::Connection>>();
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
                                db::difficulty_from_value(diff.value)
                            } else {
                                diff.difficulty_name.clone()
                            };
                            let _ = db::update_diff_ratings(
                                &guard, &hash, &mode_name, &difficulty_name,
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
        task_state.update(&task_id, progress, &format!("正在拉取星级 ({}/{})", current, total));

        let payload = ProgressPayload::new(TaskType::RefreshRatings, task_id.clone())
            .with_progress("refreshing_ratings".to_string(), progress)
            .with_message(format!("正在拉取星级 ({}/{})", current, total));
        let _ = emit_progress_event(&app, &payload);
    }

    task_state.update(&task_id, 100.0, "星级更新完成");
    let payload = ProgressPayload::new(TaskType::RefreshRatings, task_id.clone())
        .with_progress("completed".to_string(), 100.0)
        .with_message("星级更新完成".to_string());
    let _ = emit_progress_event(&app, &payload);

    // 3秒后清理任务状态
    let task_id_clone = task_id.clone();
    let app_clone = app.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        let ts = match app_clone.try_state::<TaskState>() {
            Some(s) => s,
            None => return,
        };
        ts.remove(&task_id_clone);
    });

    Ok(task_id)
}
