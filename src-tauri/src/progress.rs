use tauri::{AppHandle, Emitter};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ProgressPayload {
    pub task_type: String,       // "add_player", "add_playlist"
    pub task_id: String,         // 唯一的任务ID
    pub stage: String,           // 当前阶段描述
    pub progress: f64,           // 进度百分比 (0.0 - 100.0)
    pub total: Option<u64>,      // 总数（如果有）
    pub current: Option<u64>,    // 当前完成数（如果有）
    pub message: Option<String>, // 状态消息
}

impl ProgressPayload {
    pub fn new(task_type: String, task_id: String) -> Self {
        Self {
            task_type,
            task_id,
            stage: "initial".to_string(),
            progress: 0.0,
            total: None,
            current: None,
            message: None,
        }
    }

    pub fn with_progress(mut self, stage: String, progress: f64) -> Self {
        self.stage = stage;
        self.progress = progress;
        self
    }

    pub fn with_count(mut self, stage: String, current: u64, total: u64) -> Self {
        self.stage = stage;
        self.current = Some(current);
        self.total = Some(total);
        self.progress = if total > 0 { (current as f64 / total as f64) * 100.0 } else { 0.0 };
        self
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }
}

pub fn emit_progress_event(app: &AppHandle, payload: &ProgressPayload) -> Result<(), Box<dyn std::error::Error>> {
    app.emit("progress-update", payload)?;
    Ok(())
}