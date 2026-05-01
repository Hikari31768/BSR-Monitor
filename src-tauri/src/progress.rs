use tauri::{AppHandle, Emitter};
use serde::Serialize;
use std::fmt;

/// Task type enum for progress events
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    AddPlayer,
    AddPlaylist,
    FetchRatings,
    StartupRefresh,
}

impl fmt::Display for TaskType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskType::AddPlayer => write!(f, "add_player"),
            TaskType::AddPlaylist => write!(f, "add_playlist"),
            TaskType::FetchRatings => write!(f, "fetch_ratings"),
            TaskType::StartupRefresh => write!(f, "startup_refresh"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressPayload {
    pub task_type: TaskType,
    pub task_id: String,
    pub stage: String,
    pub progress: f64,
    pub total: Option<u64>,
    pub current: Option<u64>,
    pub message: Option<String>,
    pub player_name: Option<String>,
    pub player_avatar_url: Option<String>,
}

impl ProgressPayload {
    pub fn new(task_type: TaskType, task_id: String) -> Self {
        Self {
            task_type,
            task_id,
            stage: String::new(),
            progress: 0.0,
            total: None,
            current: None,
            message: None,
            player_name: None,
            player_avatar_url: None,
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

    pub fn with_player_name(mut self, name: String) -> Self {
        self.player_name = Some(name);
        self
    }

    pub fn with_player_avatar_url(mut self, avatar_url: String) -> Self {
        self.player_avatar_url = Some(avatar_url);
        self
    }
}

pub fn emit_progress_event(app: &AppHandle, payload: &ProgressPayload) -> Result<(), Box<dyn std::error::Error>> {
    app.emit("progress-update", payload)?;
    Ok(())
}