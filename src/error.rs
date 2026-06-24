use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Internal server error")]
    InternalError(#[from] anyhow::Error),

    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("Redis pool error")]
    RedisPoolError(#[from] deadpool_redis::PoolError),

    #[error("Email delivery failed: {0}")]
    EmailError(String),

    #[error("Invalid OTP or email")]
    InvalidOtp,

    #[error("OTP expired")]
    OtpExpired,

    #[error("Rate limited. Please try again in {reset_in_seconds} seconds.")]
    RateLimited { reset_in_seconds: u64 },

    #[error("Validation error: {0}")]
    ValidationError(#[from] validator::ValidationErrors),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Special case for RateLimited to return the retry_after_seconds field in JSON
        if let AppError::RateLimited { reset_in_seconds } = self {
            let body = Json(json!({
                "error": "Too many requests",
                "retry_after_seconds": reset_in_seconds
            }));
            return (StatusCode::TOO_MANY_REQUESTS, body).into_response();
        }

        let (status, error_message) = match self {
            AppError::InvalidOtp => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::OtpExpired => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::RateLimited { .. } => unreachable!(),
            AppError::ValidationError(ref e) => {
                let body = Json(json!({
                    "error": "Validation failed",
                    "details": e
                }));
                return (StatusCode::BAD_REQUEST, body).into_response();
            }
            AppError::EmailError(_) => {
                tracing::error!("Email error: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to send email".to_string())
            }
            AppError::InternalError(e) => {
                tracing::error!("Internal error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::RedisError(e) => {
                tracing::error!("Redis error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::RedisPoolError(e) => {
                tracing::error!("Redis pool error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}
