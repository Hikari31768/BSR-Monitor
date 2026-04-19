use reqwest::Client;
use std::time::Duration;
use crate::api::error::ApiError;

const BL_BASE_URL: &str = "https://api.beatleader.xyz";
const BEATSAVER_BASE_URL: &str = "https://api.beatsaver.com";

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

pub struct ApiClient {
    client: Client,
    bl_base_url: String,
    beatsaver_base_url: String,
}

impl ApiClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .user_agent("BSR-Monitor/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            bl_base_url: BL_BASE_URL.to_string(),
            beatsaver_base_url: BEATSAVER_BASE_URL.to_string(),
        }
    }

    pub fn bl_url(&self, path: &str) -> String {
        format!("{}{}", self.bl_base_url, path)
    }

    pub fn beatsaver_url(&self, path: &str) -> String {
        format!("{}{}", self.beatsaver_base_url, path)
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub async fn request_with_retry<T, F>(&self, f: F, max_retries: u32) -> Result<T, ApiError>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, ApiError>> + Send>>,
    {
        let mut attempts = 0;

        loop {
            let result = f().await;

            match result {
                Ok(data) => return Ok(data),
                Err(ApiError::Http(e)) => {
                    if e.is_timeout() {
                        if attempts >= max_retries {
                            return Err(ApiError::Timeout);
                        }
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        attempts += 1;
                        continue;
                    }
                    if let Some(status) = e.status() {
                        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                            if attempts >= max_retries {
                                return Err(ApiError::RateLimited(60));
                            }
                            tokio::time::sleep(Duration::from_secs(5)).await;
                            attempts += 1;
                            continue;
                        }
                    }
                    return Err(ApiError::Http(e));
                }
                Err(e) => return Err(e),
            }
        }
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new()
    }
}