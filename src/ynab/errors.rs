use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub(crate) struct ErrorResponse {
    pub(crate) error: ApiError,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ApiError {
    pub id: String,
    pub name: String,
    pub detail: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) {} - {}", self.id, self.name, self.detail)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("bad request: {0}")]
    BadRequest(ApiError),
    #[error("internal server error: {0}")]
    InternalServerError(ApiError),
    #[error("unauthorized: {0}")]
    Unauthorized(ApiError),
    #[error("rate limited: {0}")]
    RateLimited(ApiError),
    #[error("not found: {0}")]
    NotFound(ApiError),
    #[error("forbidden: {0}")]
    Forbidden(ApiError),
    #[error("conflict: {0}")]
    Conflict(ApiError),
    #[error("service unavailable: {0}")]
    ServiceUnavailable(ApiError),
    #[error("unknown error: {0}")]
    UnknownError(ApiError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    ParseError(#[from] url::ParseError),
    #[error("invalid rate limit configuration: {0}")]
    InvalidRateLimit(String),
}

impl Error {
    pub fn new_api_error(status: reqwest::StatusCode, api_error: ApiError) -> Self {
        match status {
            reqwest::StatusCode::BAD_REQUEST => Error::BadRequest(api_error),
            reqwest::StatusCode::INTERNAL_SERVER_ERROR => Error::InternalServerError(api_error),
            reqwest::StatusCode::UNAUTHORIZED => Error::Unauthorized(api_error),
            reqwest::StatusCode::TOO_MANY_REQUESTS => Error::RateLimited(api_error),
            reqwest::StatusCode::NOT_FOUND => Error::NotFound(api_error),
            reqwest::StatusCode::FORBIDDEN => Error::Forbidden(api_error),
            reqwest::StatusCode::CONFLICT => Error::Conflict(api_error),
            reqwest::StatusCode::SERVICE_UNAVAILABLE => Error::ServiceUnavailable(api_error),
            _ => Error::UnknownError(api_error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;

    fn api_error() -> ApiError {
        ApiError {
            id: "test".into(),
            name: "test_error".into(),
            detail: "something went
  wrong"
                .into(),
        }
    }

    #[test]
    fn maps_status_codes_to_error_variants() {
        let cases: &[(StatusCode, fn(&Error) -> bool)] = &[
            (StatusCode::BAD_REQUEST, |e| {
                matches!(e, Error::BadRequest(_))
            }),
            (StatusCode::UNAUTHORIZED, |e| {
                matches!(e, Error::Unauthorized(_))
            }),
            (StatusCode::FORBIDDEN, |e| matches!(e, Error::Forbidden(_))),
            (StatusCode::NOT_FOUND, |e| matches!(e, Error::NotFound(_))),
            (StatusCode::CONFLICT, |e| matches!(e, Error::Conflict(_))),
            (StatusCode::TOO_MANY_REQUESTS, |e| {
                matches!(e, Error::RateLimited(_))
            }),
            (StatusCode::INTERNAL_SERVER_ERROR, |e| {
                matches!(e, Error::InternalServerError(_))
            }),
            (StatusCode::SERVICE_UNAVAILABLE, |e| {
                matches!(e, Error::ServiceUnavailable(_))
            }),
            (StatusCode::IM_A_TEAPOT, |e| {
                matches!(e, Error::UnknownError(_))
            }),
        ];

        for (status, check) in cases {
            let err = Error::new_api_error(*status, api_error());
            assert!(check(&err), "wrong variant for status {status}");
        }
    }
}
