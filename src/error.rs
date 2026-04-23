/// All errors that can occur when using the Repology client.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// An HTTP-level error (connection refused, timeout, TLS failure, etc.)
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// The server returned a non-success status code.
    #[error("API returned HTTP {status}: {body}")]
    Api {
        status: reqwest::StatusCode,
        body: String,
    },

    /// Failed to deserialize the API response.
    #[error("failed to deserialize response: {source}")]
    Deserialize {
        #[source]
        source: serde_json::Error,
        /// The raw response body that failed to deserialize.
        body: String,
    },

    /// An error constructing a URL.
    #[error("invalid URL: {0}")]
    Url(#[from] url::ParseError),

    /// The client was not configured correctly.
    #[error("client configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, Error>;
