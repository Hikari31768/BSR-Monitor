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

#[derive(Default)]
pub struct TaskState {
    pub tasks: Mutex<HashMap<String, TaskStatus>>,
}

impl TaskState {
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
