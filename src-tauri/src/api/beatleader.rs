use serde::{Deserialize, Serialize};
use crate::api::{ApiClient, ApiError};
use crate::api::types::Metadata;

const MAX_RETRIES: u32 = 3;

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerResponse {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub avatar: String,
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub alias: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompactScoreResponse {
    pub score: CompactScore,
    pub leaderboard: CompactLeaderboard,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompactScoreResponseWithMetadata {
    pub metadata: Metadata,
    pub data: Vec<CompactScoreResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompactScore {
    pub id: i32,
    #[serde(default)]
    pub base_score: i32,
    #[serde(default)]
    pub modified_score: i32,
    pub accuracy: f32,
    #[serde(rename = "epochTime")]
    pub epoch_time: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompactLeaderboard {
    pub id: String,
    #[serde(rename = "songHash", default)]
    pub song_hash: String,
    #[serde(rename = "modeName", default)]
    pub mode_name: String,
    #[serde(default)]
    pub difficulty: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaderboardResponse {
    pub id: String,
    pub song: SongResponse,
    pub difficulty: DifficultyResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SongResponse {
    pub hash: String,
    pub name: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub mapper: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DifficultyResponse {
    pub value: i32,
    #[serde(default)]
    pub mode: i32,
    #[serde(default)]
    pub mode_name: String,
    #[serde(default)]
    pub difficulty_name: String,
}

pub struct BeatLeaderApi {
    client: ApiClient,
}

impl BeatLeaderApi {
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

    pub async fn get_player(&self, player_id: &str) -> Result<PlayerResponse, ApiError> {
        let url = self.client.bl_url(format!("/player/{player_id}").as_str());

        self.client.request_with_retry(|| {
            let url = url.clone();
            let client = self.client.client().clone();
            Box::pin(async move {
                let response = client.get(&url).send().await?;
                Self::handle_response(response).await
            })
        }, MAX_RETRIES).await
    }

    pub async fn get_player_scores(
        &self,
        player_id: &str,
        page: i32,
        count: i32,
    ) -> Result<CompactScoreResponseWithMetadata, ApiError> {
        let url = self.client.bl_url(format!("/player/{player_id}/scores/compact").as_str());

        self.client.request_with_retry(|| {
            let url = url.clone();
            let client = self.client.client().clone();
            let page = page;
            let count = count;
            Box::pin(async move {
                let response = client
                    .post(&url)
                    .query(&[
                        ("page", page.to_string()),
                        ("count", count.to_string()),
                        ("sortBy", "date".to_string()),
                    ])
                    .send()
                    .await?;
                Self::handle_response(response).await
            })
        }, MAX_RETRIES).await
    }

    pub async fn get_leaderboard(&self, leaderboard_id: &str) -> Result<LeaderboardResponse, ApiError> {
        let url = self.client.bl_url(format!("/leaderboard/{leaderboard_id}").as_str());

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

impl Default for BeatLeaderApi {
    fn default() -> Self {
        Self::new().expect("Failed to create default BeatLeaderApi")
    }
}