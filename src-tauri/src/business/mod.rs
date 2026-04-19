pub mod player;
pub mod score;
pub mod map;
pub mod playlist;
pub mod leaderboard;

#[cfg(test)]
mod tests;

// 业务错误类型
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BusinessError {
    #[error("API error: {0}")]
    ApiError(#[from] crate::api::ApiError),

    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("Player not found: {0}")]
    PlayerNotFound(String),

    #[error("Map not found: {0}")]
    MapNotFound(String),

    #[error("Playlist not found: {0}")]
    PlaylistNotFound(i64),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}