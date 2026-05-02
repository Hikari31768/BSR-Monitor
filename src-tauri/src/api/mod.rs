pub mod client;
pub mod error;
pub mod beatleader;
pub mod beatsaver;
pub mod types;

#[cfg(test)]
mod tests;

pub use client::ApiClient;
pub use error::ApiError;
pub use beatleader::{BeatLeaderApi, PlayerResponse, CompactScoreResponse, CompactScoreResponseWithMetadata, CompactScore, CompactLeaderboard, LeaderboardResponse, LeaderboardsByHashResponse};
pub use beatsaver::{BeatSaverApi, MapDetail, MapDetailMetadata, MapStats, MapVersion, MapDifficulty, PlaylistPage, PlaylistFull, PlaylistStats, PlaylistType, MapDetailWithOrder, UserProfileResponse};