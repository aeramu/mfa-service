use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, Debug, ToSchema, Validate)]
pub struct GenerateOtpRequest {
    #[validate(email)]
    #[schema(example = "user@example.com")]
    pub email: String,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct GenerateOtpResponse {
    #[schema(example = "OTP sent successfully")]
    pub message: String,
    #[schema(example = 10)]
    pub expires_in_minutes: u64,
    #[schema(example = 30)]
    pub retry_after_seconds: u64,
}

#[derive(Deserialize, Debug, ToSchema, Validate)]
pub struct VerifyOtpRequest {
    #[validate(email)]
    #[schema(example = "user@example.com")]
    pub email: String,
    #[schema(example = "123456")]
    pub code: String,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct VerifyOtpResponse {
    #[schema(example = "OTP verified successfully")]
    pub message: String,
    #[schema(example = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    #[schema(example = "Invalid or expired OTP")]
    pub error: String,
}

#[derive(Serialize, ToSchema)]
pub struct RateLimitResponse {
    #[schema(example = "Too many requests")]
    pub error: String,
    #[schema(example = 30)]
    pub retry_after_seconds: u64,
}

#[derive(Serialize, ToSchema)]
pub struct ValidationErrorResponse {
    #[schema(example = "Validation failed")]
    pub error: String,
    #[schema(value_type = Object, example = json!({"email": [{"code": "email", "message": null, "params": {"value": "not-an-email"}}]}))]
    pub details: serde_json::Value,
}
