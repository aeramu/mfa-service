use axum::{extract::State, Json};
use rand::Rng;
use std::sync::Arc;
use validator::Validate;
use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
use chrono::{Utc, Duration};

use crate::{
    error::AppError,
    models::{GenerateOtpRequest, GenerateOtpResponse, VerifyOtpRequest, VerifyOtpResponse, JwtClaims},
    services::{email::EmailService, redis::RedisService},
    config::Config,
};

#[derive(Clone)]
pub struct AppState {
    pub redis_service: RedisService,
    pub email_service: EmailService,
    pub config: Config,
    pub jwt_private_key: Vec<u8>,
    pub jwt_public_key: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/otp/public-key",
    responses(
        (status = 200, description = "Returns the RS256 Public Key in PEM format", body = String)
    )
)]
pub async fn get_public_key(State(state): State<Arc<AppState>>) -> String {
    state.jwt_public_key.clone()
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
        .check_rate_limit(
            &payload.email,
            state.config.rate_limit_reset_seconds,
            state.config.rate_limit_base_delay_seconds,
            state.config.rate_limit_multiplier,
        )
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

    let is_bypass_code = state
        .config
        .otp_bypass_code
        .as_deref()
        .is_some_and(|code| code == payload.code);

    if is_bypass_code {
        tracing::warn!(email = %payload.email, "OTP verification bypass used");
    } else {
        state
            .redis_service
            .verify_otp(&payload.email, &payload.code, state.config.max_verify_attempts)
            .await?;
    }

    let expiration = Utc::now() + Duration::hours(state.config.jwt_expiration_hours as i64);
    let claims = JwtClaims {
        sub: payload.email.clone(),
        exp: expiration.timestamp() as usize,
    };
    
    let key = EncodingKey::from_rsa_pem(&state.jwt_private_key)?;
    let header = Header::new(Algorithm::RS256);
    let token = encode(&header, &claims, &key)?;

    Ok(Json(VerifyOtpResponse {
        message: "OTP verified successfully".to_string(),
        token,
    }))
}

fn generate_random_otp(length: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| rng.gen_range(0..10).to_string())
        .collect()
}
