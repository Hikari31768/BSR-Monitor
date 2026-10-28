use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct TaskStatus {
    pub task_type: String,
    pub target_id: String,
    pub progress: f64,
    pub message: String,
    pub display_name: String,
}

pub struct TaskState {
    pub tasks: Mutex<HashMap<String, TaskStatus>>,
    pub player_slots: tokio::sync::Semaphore,
    pub playlist_slots: tokio::sync::Semaphore,
    pub map_info_slots: tokio::sync::Semaphore,
}

impl Default for TaskState {
    fn default() -> Self {
        Self::with_limits(&crate::settings::ConcurrencySettings::default())
    }
}

impl TaskState {
    pub fn with_limits(settings: &crate::settings::ConcurrencySettings) -> Self {
        Self {
            tasks: Mutex::new(HashMap::new()),
            player_slots: tokio::sync::Semaphore::new(settings.startup_players),
            playlist_slots: tokio::sync::Semaphore::new(settings.startup_playlists),
            map_info_slots: tokio::sync::Semaphore::new(settings.map_info),
        }
    }
}

pub struct TaskRegistration<'a> {
    state: &'a TaskState,
    task_id: String,
}

impl Drop for TaskRegistration<'_> {
    fn drop(&mut self) { self.state.remove(&self.task_id); }
}

impl TaskState {
    /// Claim the target atomically so startup/manual refresh cannot race.
    pub fn begin(&self, task_id: &str, task_type: &str, target_id: &str, display_name: &str)
        -> Result<TaskRegistration<'_>, String>
    {
        let mut tasks = self.tasks.lock().map_err(|e| e.to_string())?;
        if tasks.values().any(|s| s.target_id == target_id) {
            return Err(format!("{}正在处理中，请稍候", display_name));
        }
        tasks.insert(task_id.to_string(), TaskStatus {
            task_type: task_type.to_string(), target_id: target_id.to_string(),
            progress: 0.0, message: "等待刷新...".into(), display_name: display_name.to_string(),
        });
        Ok(TaskRegistration { state: self, task_id: task_id.to_string() })
    }
    pub fn register(&self, task_id: &str, task_type: &str, target_id: &str) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.insert(
            task_id.to_string(),
            TaskStatus {
                task_type: task_type.to_string(),
                target_id: target_id.to_string(),
                progress: 0.0,
                message: String::new(),
                display_name: String::new(),
            },
        );
    }

    /// 注册带显示名称的任务（用于后台任务）
    pub fn register_with_name(&self, task_id: &str, task_type: &str, target_id: &str, display_name: &str) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.insert(
            task_id.to_string(),
            TaskStatus {
                task_type: task_type.to_string(),
                target_id: target_id.to_string(),
                progress: 0.0,
                message: String::new(),
                display_name: display_name.to_string(),
            },
        );
    }

    pub fn update(&self, task_id: &str, progress: f64, message: &str) {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(status) = tasks.get_mut(task_id) {
            status.progress = progress;
            status.message = message.to_string();
        }
    }

    pub fn remove(&self, task_id: &str) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.remove(task_id);
    }

    pub fn get_all(&self) -> Vec<(String, TaskStatus)> {
        let tasks = self.tasks.lock().unwrap();
        tasks.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    pub fn has_active_for_target(&self, target_id: &str) -> bool {
        let tasks = self.tasks.lock().unwrap();
        tasks.values().any(|s| s.target_id == target_id)
    }
}

#[tauri::command]
pub fn get_active_tasks(state: State<TaskState>) -> Vec<(String, TaskStatus)> {
    state.get_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claims_prevent_duplicate_refresh_or_delete_and_are_released_on_drop() {
        let state = TaskState::default();
        let first = state.begin("first", "startup_player", "player-1", "Player").unwrap();
        assert!(state.begin("manual", "refresh_player_scores", "player-1", "Player").is_err());
        assert!(state.begin("delete", "delete_player", "player-1", "Player").is_err());
        assert!(state.begin("other", "startup_player", "player-2", "Other").is_ok());
        drop(first);
        assert!(state.get_all().is_empty());
        assert!(state.begin("manual", "refresh_player_scores", "player-1", "Player").is_ok());
    }
}
