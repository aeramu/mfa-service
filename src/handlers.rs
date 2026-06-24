use axum::{extract::State, Json};
use rand::Rng;
use std::sync::Arc;
use validator::Validate;

use crate::{
    error::AppError,
    models::{GenerateOtpRequest, GenerateOtpResponse, VerifyOtpRequest, VerifyOtpResponse},
    services::{email::EmailService, redis::RedisService},
    config::Config,
};

#[derive(Clone)]
pub struct AppState {
    pub redis_service: RedisService,
    pub email_service: EmailService,
    pub config: Config,
}

#[utoipa::path(
    post,
    path = "/api/v1/otp/generate",
    request_body = GenerateOtpRequest,
    responses(
        (status = 200, description = "OTP generated and sent", body = GenerateOtpResponse),
        (status = 400, description = "Validation failed (e.g., invalid email format)", body = ValidationErrorResponse),
        (status = 429, description = "Rate limited, includes retry_after_seconds", body = RateLimitResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn generate_otp(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<GenerateOtpRequest>,
) -> Result<Json<GenerateOtpResponse>, AppError> {
    payload.validate()?;

    // 1. Check Rate Limit and get cooldown for next request
    let retry_after_seconds = state
        .redis_service
        .check_rate_limit(&payload.email, state.config.rate_limit_reset_seconds)
        .await?;

    // 2. Generate Random OTP
    let otp = generate_random_otp(state.config.otp_length);

    // 3. Save to Redis
    state
        .redis_service
        .save_otp(&payload.email, &otp, state.config.otp_expiration_minutes)
        .await?;

    // 4. Send Email (running blocking operation in tokio runtime)
    let email = payload.email.clone();
    let email_service = state.email_service.clone();
    let expiration_minutes = state.config.otp_expiration_minutes;
    
    // lettre SmtpTransport sends synchronously. So we must use spawn_blocking.
    tokio::task::spawn_blocking(move || {
        email_service.send_otp(&email, &otp, expiration_minutes)
    })
    .await
    .map_err(|e| AppError::InternalError(e.into()))??;

    Ok(Json(GenerateOtpResponse {
        message: "OTP sent successfully".to_string(),
        expires_in_minutes: state.config.otp_expiration_minutes,
        retry_after_seconds,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/otp/verify",
    request_body = VerifyOtpRequest,
    responses(
        (status = 200, description = "OTP verified successfully", body = VerifyOtpResponse),
        (status = 400, description = "Validation failed (e.g., invalid email format)", body = ValidationErrorResponse),
        (status = 401, description = "Invalid or expired OTP", body = ErrorResponse),
        (status = 429, description = "Too many verification attempts, includes retry_after_seconds", body = RateLimitResponse)
    )
)]
pub async fn verify_otp(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VerifyOtpRequest>,
) -> Result<Json<VerifyOtpResponse>, AppError> {
    payload.validate()?;

    state
        .redis_service
        .verify_otp(&payload.email, &payload.code, state.config.max_verify_attempts)
        .await?;

    Ok(Json(VerifyOtpResponse {
        message: "OTP verified successfully".to_string(),
    }))
}

fn generate_random_otp(length: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| rng.gen_range(0..10).to_string())
        .collect()
}
