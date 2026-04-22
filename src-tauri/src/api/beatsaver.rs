use serde::{Deserialize, Serialize};
use crate::api::{ApiClient, ApiError};

const MAX_RETRIES: u32 = 3;

#[derive(Debug, Serialize, Deserialize)]
pub struct MapDetail {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub metadata: MapDetailMetadata,
    #[serde(default)]
    pub stats: Option<MapStats>,
    pub versions: Vec<MapVersion>,
    #[serde(default)]
    pub uploader: Option<UserDetail>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MapDetailMetadata {
    #[serde(rename = "songName", default)]
    pub song_name: String,
    #[serde(rename = "songAuthorName", default)]
    pub song_author_name: String,
    #[serde(rename = "levelAuthorName", default)]
    pub level_author_name: String,
    #[serde(default)]
    pub bpm: f32,
    #[serde(default)]
    pub duration: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MapStats {
    #[serde(default)]
    pub score: f32,
    #[serde(default)]
    pub downloads: i32,
    #[serde(default)]
    pub plays: i32,
    #[serde(default)]
    pub upvotes: i32,
    #[serde(default)]
    pub downvotes: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MapVersion {
    pub hash: String,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(rename = "coverURL", default)]
    pub cover_url: String,
    #[serde(rename = "previewURL", default)]
    pub preview_url: String,
    #[serde(rename = "downloadURL", default)]
    pub download_url: String,
    pub diffs: Vec<MapDifficulty>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MapDifficulty {
    pub characteristic: String,
    pub difficulty: String,
    #[serde(default)]
    pub nps: f32,
    #[serde(default)]
    pub stars: Option<f32>,
    #[serde(default)]
    pub notes: i32,
    #[serde(default)]
    pub bombs: i32,
    #[serde(default)]
    pub obstacles: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDetail {
    pub id: i32,
    pub name: String,
    #[serde(default)]
    pub avatar: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistPage {
    pub playlist: PlaylistFull,
    pub maps: Vec<MapDetailWithOrder>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum PlaylistType {
    Private,
    Public,
    System,
    Search,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistFull {
    #[serde(rename = "playlistId")]
    pub playlist_id: i32,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "playlistImage", default)]
    pub playlist_image: Option<String>,
    pub owner: UserDetail,
    #[serde(rename = "downloadURL", default)]
    pub download_url: Option<String>,
    #[serde(default)]
    pub stats: Option<PlaylistStats>,
    #[serde(rename = "createdAt", default)]
    pub created_at: Option<String>,
    #[serde(rename = "songsChangedAt", default)]
    pub songs_changed_at: Option<String>,
    #[serde(rename = "type", default)]
    pub playlist_type: Option<PlaylistType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistStats {
    #[serde(rename = "totalMaps", default)]
    pub total_maps: i32,
    #[serde(rename = "avgScore", default)]
    pub avg_score: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MapDetailWithOrder {
    pub map: MapDetail,
    pub order: f32,
}

pub struct BeatSaverApi {
    client: ApiClient,
}

impl BeatSaverApi {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            client: ApiClient::new()?,
        })
    }

    pub fn with_client(client: ApiClient) -> Self {
        Self { client }
    }

    async fn handle_response<T: for<'de> Deserialize<'de>>(response: reqwest::Response) -> Result<T, ApiError> {
        let status = response.status();
        if status == 404 {
            return Err(ApiError::NotFound("Resource not found".to_string()));
        }
        if !status.is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(ApiError::ApiStatus { status: status.as_u16(), message });
        }
        response.json::<T>().await.map_err(ApiError::from)
    }

    pub async fn get_map_by_key(&self, key: &str) -> Result<MapDetail, ApiError> {
        let url = self.client.beatsaver_url(format!("/maps/id/{key}").as_str());

        self.client.request_with_retry(|| {
            let url = url.clone();
            let client = self.client.client().clone();
            Box::pin(async move {
                let response = client.get(&url).send().await?;
                Self::handle_response(response).await
            })
        }, MAX_RETRIES).await
    }

    pub async fn get_map_by_hash(&self, hash: &str) -> Result<MapDetail, ApiError> {
        let url = self.client.beatsaver_url(format!("/maps/hash/{hash}").as_str());

        self.client.request_with_retry(|| {
            let url = url.clone();
            let client = self.client.client().clone();
            Box::pin(async move {
                let response = client.get(&url).send().await?;
                Self::handle_response(response).await
            })
        }, MAX_RETRIES).await
    }

    pub async fn get_playlist(&self, playlist_id: i32, page: i32) -> Result<PlaylistPage, ApiError> {
        let url = self.client.beatsaver_url(format!("/playlists/id/{playlist_id}/{page}").as_str());

        self.client.request_with_retry(|| {
            let url = url.clone();
            let client = self.client.client().clone();
            Box::pin(async move {
                let response = client.get(&url).send().await?;
                Self::handle_response(response).await
            })
        }, MAX_RETRIES).await
    }
}

impl Default for BeatSaverApi {
    fn default() -> Self {
        Self::new().expect("Failed to create default BeatSaverApi")
    }
}