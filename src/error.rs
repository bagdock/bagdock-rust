use thiserror::Error;

pub type Result<T> = std::result::Result<T, BagdockError>;

#[derive(Error, Debug)]
pub enum BagdockError {
    #[error("authentication error: {0}")]
    Authentication(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("API error {status}: {code} — {message}")]
    Api {
        status: u16,
        code: String,
        message: String,
    },

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("rate limit exceeded")]
    RateLimit,
}
