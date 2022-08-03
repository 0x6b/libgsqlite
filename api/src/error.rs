use oauth2::{
    basic::BasicErrorResponseType, url::ParseError, RequestTokenError, StandardErrorResponse,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    FailedRequest(#[from] reqwest::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error(transparent)]
    Token(
        #[from]
        RequestTokenError<
            oauth2::reqwest::Error<reqwest::Error>,
            StandardErrorResponse<BasicErrorResponseType>,
        >,
    ),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error("Unexpected token")]
    UnexpectedToken,
    #[error("Unexpected response: {0}")]
    UnexpectedResponse(String),
    #[error("Invalid redirect URL")]
    InvalidRedirectUrl,
    #[error("Missing code")]
    CodeMissing,
    #[error("Invalid sheet ID")]
    InvalidSheetId,
}
