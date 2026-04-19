use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Failed to parse JSON response: {0}")]
    JsonParse(String),

    #[error("API returned error status {status}: {message}")]
    ApiStatus { status: u16, message: String },

    #[error("Rate limited, please retry after {0} seconds")]
    RateLimited(u64),

    #[error("Network timeout")]
    Timeout,

    #[error("Resource not found: {0}")]
    NotFound(String),
}

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        ApiError::JsonParse(e.to_string())
    }
}