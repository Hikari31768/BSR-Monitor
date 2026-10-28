use std::{fs, path::{Path, PathBuf}, sync::Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

pub const MAX_CONCURRENCY: usize = 5;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ConcurrencySettings {
    pub startup_players: usize,
    pub startup_playlists: usize,
    pub map_info: usize,
    pub add_player_pages: usize,
}

impl Default for ConcurrencySettings {
    fn default() -> Self {
        Self { startup_players: 5, startup_playlists: 5, map_info: 5, add_player_pages: 5 }
    }
}

impl ConcurrencySettings {
    fn validate(&self) -> Result<(), String> {
        if [self.startup_players, self.startup_playlists, self.map_info, self.add_player_pages]
            .iter().any(|value| !(1..=MAX_CONCURRENCY).contains(value)) {
            return Err("并发数必须在 1 到 5 之间".into());
        }
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PendingDirectory {
    path: PathBuf,
    migration_id: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct StoredSettings {
    concurrency: ConcurrencySettings,
    data_directory: Option<PathBuf>,
    pending_directory: Option<PendingDirectory>,
}

pub struct SettingsState {
    config_path: PathBuf,
    default_directory: PathBuf,
    pub data_directory: PathBuf,
    pub concurrency: ConcurrencySettings,
    stored: Mutex<StoredSettings>,
    migration_error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    concurrency: ConcurrencySettings,
    data_directory: String,
    current_data_directory: String,
    default_data_directory: String,
    restart_required: bool,
    migration_error: Option<String>,
}

pub(crate) fn display_directory(path: &Path) -> String {
    let value = path.to_string_lossy();
    if let Some(unc) = value.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{unc}")
    } else {
        value.strip_prefix(r"\\?\").unwrap_or(&value).to_string()
    }
}

fn write_config(path: &Path, settings: &StoredSettings) -> Result<(), String> {
    let temporary = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let content = serde_json::to_vec_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&temporary, content).map_err(|e| format!("保存设置失败：{e}"))?;
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(format!("保存设置失败：{error}"));
    }
    Ok(())
}

impl SettingsState {
    pub fn load(default_directory: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&default_directory).map_err(|e| e.to_string())?;
        let default_directory = fs::canonicalize(default_directory).map_err(|e| e.to_string())?;
        let config_path = default_directory.join("settings.json");
        let mut stored: StoredSettings = if config_path.exists() {
            serde_json::from_slice(&fs::read(&config_path).map_err(|e| e.to_string())?)
                .map_err(|e| format!("读取设置失败：{e}"))?
        } else { StoredSettings::default() };
        stored.concurrency.validate()?;
        let mut data_directory = stored.data_directory.clone().unwrap_or_else(|| default_directory.clone());
        let mut migration_error = None;
        if let Some(pending) = stored.pending_directory.clone() {
            match migrate_data(&data_directory, &pending) {
                Ok(()) => {
                    let mut completed = stored.clone();
                    completed.data_directory = Some(pending.path.clone());
                    completed.pending_directory = None;
                    // A marker in the destination lets the next launch recover
                    // if the copy succeeded but saving this pointer failed.
                    write_config(&config_path, &completed)?;
                    stored = completed;
                    data_directory = pending.path;
                }
                Err(error) => migration_error = Some(format!("数据目录迁移失败，仍使用原目录：{error}")),
            }
        }
        fs::create_dir_all(&data_directory).map_err(|e| format!("打开数据目录失败：{e}"))?;
        Ok(Self {
            config_path, default_directory, data_directory,
            concurrency: stored.concurrency.clone(), stored: Mutex::new(stored), migration_error,
        })
    }

    fn view(&self, stored: &StoredSettings) -> SettingsView {
        SettingsView {
            concurrency: stored.concurrency.clone(),
            data_directory: display_directory(stored.pending_directory.as_ref().map(|p| p.path.as_path())
                .unwrap_or(&self.data_directory)),
            current_data_directory: display_directory(&self.data_directory),
            default_data_directory: display_directory(&self.default_directory),
            restart_required: stored.pending_directory.is_some() || stored.concurrency != self.concurrency,
            migration_error: self.migration_error.clone(),
        }
    }
}

pub fn data_directory(app: &AppHandle) -> PathBuf {
    app.state::<SettingsState>().data_directory.clone()
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<SettingsView, String> {
    let state = app.state::<SettingsState>();
    let stored = state.stored.lock().map_err(|e| e.to_string())?;
    Ok(state.view(&stored))
}

#[tauri::command]
pub fn save_settings(app: AppHandle, concurrency: ConcurrencySettings, data_directory: String) -> Result<SettingsView, String> {
    concurrency.validate()?;
    let state = app.state::<SettingsState>();
    let selected = PathBuf::from(data_directory.trim());
    if !selected.is_absolute() { return Err("请选择完整的数据目录路径".into()); }
    fs::create_dir_all(&selected).map_err(|e| format!("创建数据目录失败：{e}"))?;
    let selected = fs::canonicalize(selected).map_err(|e| e.to_string())?;
    let mut stored = state.stored.lock().map_err(|e| e.to_string())?;
    let mut next = stored.clone();
    next.concurrency = concurrency;
    if selected == state.data_directory {
        next.pending_directory = None;
    } else {
        if selected.starts_with(&state.data_directory) || state.data_directory.starts_with(&selected) {
            return Err("新目录不能与当前数据目录互相包含".into());
        }
        ensure_empty_directory(&selected)?;
        let probe = selected.join(format!(".bsr-write-{}", uuid::Uuid::new_v4()));
        fs::write(&probe, []).map_err(|e| format!("数据目录不可写：{e}"))?;
        fs::remove_file(probe).map_err(|e| e.to_string())?;
        next.pending_directory = Some(PendingDirectory {
            path: selected, migration_id: uuid::Uuid::new_v4().to_string(),
        });
    }
    write_config(&state.config_path, &next)?;
    *stored = next;
    Ok(state.view(&stored))
}

#[tauri::command]
pub fn restart_app(app: AppHandle) -> Result<(), String> {
    if !app.state::<crate::task_state::TaskState>().get_all().is_empty() {
        return Err("后台任务仍在运行，请等待全部完成后再重启".into());
    }
    app.restart();
}

fn ensure_empty_directory(directory: &Path) -> Result<(), String> {
    if fs::read_dir(directory).map_err(|e| e.to_string())?.next().is_some() {
        return Err("请选择空文件夹，避免覆盖已有数据".into());
    }
    Ok(())
}

fn copy_directory(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_symlink() { return Err("歌单缓存包含链接，请先移除链接再迁移".into()); }
        let target = destination.join(entry.file_name());
        if kind.is_dir() { copy_directory(&entry.path(), &target)?; }
        else { fs::copy(entry.path(), target).map_err(|e| e.to_string())?; }
    }
    Ok(())
}

fn migrate_data(source: &Path, pending: &PendingDirectory) -> Result<(), String> {
    let destination = &pending.path;
    let marker = ".bsr-monitor-migration";
    if fs::read_to_string(destination.join(marker)).ok().as_deref() == Some(&pending.migration_id) {
        return Ok(());
    }
    fs::create_dir_all(destination).map_err(|e| e.to_string())?;
    ensure_empty_directory(destination)?;
    let staging = destination.parent().ok_or("无效的目标目录")?
        .join(format!(".bsr-monitor-migrate-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&staging).map_err(|e| e.to_string())?;
    let result = (|| -> Result<(), String> {
        for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name();
            let name_text = name.to_string_lossy();
            if name_text == "Playlists" { copy_directory(&entry.path(), &staging.join(&name))?; }
            else if name_text == "user_prefs.json" || (name_text.starts_with("bsr_monitor.") && name_text.ends_with(".db") && name_text != "bsr_monitor.db") {
                fs::copy(entry.path(), staging.join(&name)).map_err(|e| e.to_string())?;
            }
        }
        let database = source.join("bsr_monitor.db");
        if database.exists() {
            let connection = rusqlite::Connection::open_with_flags(database, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                .map_err(|e| e.to_string())?;
            let target_database = staging.join("bsr_monitor.db");
            connection.execute("VACUUM INTO ?1", [target_database.to_string_lossy().as_ref()]).map_err(|e| e.to_string())?;
            let copied = rusqlite::Connection::open(target_database).map_err(|e| e.to_string())?;
            let has_playlists: bool = copied.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='Playlist')", [], |row| row.get(0))
                .map_err(|e| e.to_string())?;
            if has_playlists {
                let mut query = copied.prepare("SELECT source, identifier, localPath FROM Playlist WHERE localPath IS NOT NULL").map_err(|e| e.to_string())?;
                let paths = query.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)))
                    .map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
                let old_cache = fs::canonicalize(source.join("Playlists")).ok();
                for (source_name, identifier, path) in paths {
                    let canonical = fs::canonicalize(&path).ok();
                    if let (Some(old_cache), Some(canonical)) = (&old_cache, canonical) {
                        if let Ok(relative) = canonical.strip_prefix(old_cache) {
                            let new_path = destination.join("Playlists").join(relative);
                            copied.execute("UPDATE Playlist SET localPath=?1 WHERE source=?2 AND identifier=?3",
                                rusqlite::params![new_path.to_string_lossy(), source_name, identifier]).map_err(|e| e.to_string())?;
                        }
                    }
                }
            }
            let check: String = copied.query_row("PRAGMA integrity_check", [], |row| row.get(0)).map_err(|e| e.to_string())?;
            if check != "ok" { return Err(format!("数据库校验失败：{check}")); }
        }
        fs::write(staging.join(marker), &pending.migration_id).map_err(|e| e.to_string())?;
        // remove_dir only succeeds while the selected destination is still empty.
        fs::remove_dir(destination).map_err(|e| e.to_string())?;
        fs::rename(&staging, destination).map_err(|e| e.to_string())?;
        Ok(())
    })();
    if result.is_err() { let _ = fs::remove_dir_all(&staging); }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDirectory(PathBuf);
    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("bsr-settings-test-{}", uuid::Uuid::new_v4()));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for TestDirectory {
        fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
    }

    #[test]
    fn directory_migration_preserves_wal_scores_cache_paths_and_preferences() {
        let test = TestDirectory::new();
        let source = test.0.join("old");
        let target = test.0.join("new");
        fs::create_dir_all(source.join("Playlists")).unwrap();
        let cache = source.join("Playlists/test.bplist");
        fs::write(&cache, "playlist content").unwrap();
        fs::write(source.join("user_prefs.json"), "{\"volume\":0.4}").unwrap();
        fs::write(source.join("bsr_monitor.before-score-normalization.db"), "backup").unwrap();
        let original = rusqlite::Connection::open(source.join("bsr_monitor.db")).unwrap();
        original.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE Scores (accuracy REAL); INSERT INTO Scores VALUES (0.95); CREATE TABLE Playlist (source TEXT, identifier TEXT, localPath TEXT);").unwrap();
        original.execute("INSERT INTO Playlist VALUES ('local','original-path',?1)", [cache.to_string_lossy().as_ref()]).unwrap();
        let pending = PendingDirectory { path: target.clone(), migration_id: "migration-test".into() };
        migrate_data(&source, &pending).unwrap();
        let copied = rusqlite::Connection::open(target.join("bsr_monitor.db")).unwrap();
        assert_eq!(copied.query_row("SELECT accuracy FROM Scores", [], |row| row.get::<_, f64>(0)).unwrap(), 0.95);
        let (identifier, cache_path): (String, String) = copied.query_row("SELECT identifier, localPath FROM Playlist", [], |row| Ok((row.get(0)?, row.get(1)?))).unwrap();
        assert_eq!(identifier, "original-path");
        assert_eq!(PathBuf::from(&cache_path), target.join("Playlists/test.bplist"));
        assert_eq!(fs::read_to_string(cache_path).unwrap(), "playlist content");
        assert_eq!(fs::read_to_string(target.join("user_prefs.json")).unwrap(), "{\"volume\":0.4}");
        assert!(target.join("bsr_monitor.before-score-normalization.db").exists());
        assert!(source.join("bsr_monitor.db").exists());
        // A launch interrupted after directory replacement can finish safely.
        migrate_data(&source, &pending).unwrap();
    }

    #[test]
    fn migration_rejects_existing_data_and_load_keeps_the_original_directory() {
        let test = TestDirectory::new();
        let source = test.0.join("old");
        let target = test.0.join("new");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("important.txt"), "keep").unwrap();
        let settings = StoredSettings {
            pending_directory: Some(PendingDirectory { path: target.clone(), migration_id: "test".into() }),
            ..StoredSettings::default()
        };
        write_config(&source.join("settings.json"), &settings).unwrap();
        let loaded = SettingsState::load(source.clone()).unwrap();
        assert_eq!(loaded.data_directory, fs::canonicalize(source).unwrap());
        assert!(loaded.migration_error.is_some());
        assert_eq!(fs::read_to_string(target.join("important.txt")).unwrap(), "keep");
    }

    #[test]
    fn saved_limits_and_data_directory_survive_restart() {
        let test = TestDirectory::new();
        let source = test.0.join("old");
        let target = test.0.join("new");
        fs::create_dir_all(&source).unwrap();
        let settings = StoredSettings {
            concurrency: ConcurrencySettings { startup_players: 1, startup_playlists: 2, map_info: 3, add_player_pages: 4 },
            pending_directory: Some(PendingDirectory { path: target.clone(), migration_id: "saved-test".into() }),
            ..StoredSettings::default()
        };
        write_config(&source.join("settings.json"), &settings).unwrap();
        let loaded = SettingsState::load(source.clone()).unwrap();
        assert_eq!(loaded.concurrency, settings.concurrency);
        assert_eq!(loaded.data_directory, target);
        let reloaded = SettingsState::load(source).unwrap();
        assert_eq!(reloaded.data_directory, loaded.data_directory);
        assert!(reloaded.stored.lock().unwrap().pending_directory.is_none());
        assert!(ConcurrencySettings { startup_players: 0, ..ConcurrencySettings::default() }.validate().is_err());
        assert!(ConcurrencySettings { add_player_pages: 6, ..ConcurrencySettings::default() }.validate().is_err());
    }
}
