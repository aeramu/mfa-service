use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub redis_url: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_password: String,
    pub smtp_use_tls: bool,
    pub from_address: String,
    pub jwt_private_key_path: String,
    pub jwt_public_key_path: String,
    pub jwt_expiration_hours: u64,
    pub otp_expiration_minutes: u64,
    pub otp_length: usize,
    pub rate_limit_reset_seconds: u64,
    pub max_verify_attempts: u32,
    pub rate_limit_base_delay_seconds: u64,
    pub rate_limit_multiplier: f64,
}

impl Config {
    pub fn from_env() -> Self {
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .expect("PORT must be a number");

        let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");

        let smtp_host = env::var("SMTP_HOST").expect("SMTP_HOST must be set");
        let smtp_port = env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse()
            .expect("SMTP_PORT must be a number");
        let smtp_user = env::var("SMTP_USER").unwrap_or_default();
        let smtp_password = env::var("SMTP_PASSWORD").unwrap_or_default();
        let smtp_use_tls = env::var("SMTP_USE_TLS")
            .map(|s| s == "true" || s == "1")
            .unwrap_or(false); // Default to false for local Mailpit dev
        let from_address = env::var("FROM_ADDRESS")
            .or_else(|_| env::var("SMTP_FROM"))
            .unwrap_or_else(|_| "noreply@example.com".to_string());
        let jwt_private_key_path = env::var("JWT_PRIVATE_KEY_PATH").unwrap_or_else(|_| "keys/private.pem".to_string());
        let jwt_public_key_path = env::var("JWT_PUBLIC_KEY_PATH").unwrap_or_else(|_| "keys/public.pem".to_string());
        let jwt_expiration_hours = env::var("JWT_EXPIRATION_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse()
            .expect("JWT_EXPIRATION_HOURS must be a number");
        let otp_expiration_minutes = env::var("OTP_EXPIRATION_MINUTES")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .expect("OTP_EXPIRATION_MINUTES must be a number");
        let otp_length = env::var("OTP_LENGTH")
            .unwrap_or_else(|_| "6".to_string())
            .parse()
            .expect("OTP_LENGTH must be a number");

        let rate_limit_reset_seconds = env::var("RATE_LIMIT_RESET_HOURS")
            .map(|h| h.parse::<u64>().expect("RATE_LIMIT_RESET_HOURS must be a number") * 3600)
            .unwrap_or(86400); // Default to 24 hours (86400 seconds)

        let max_verify_attempts = env::var("MAX_VERIFY_ATTEMPTS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .expect("MAX_VERIFY_ATTEMPTS must be a number");

        let rate_limit_base_delay_seconds = env::var("RATE_LIMIT_BASE_DELAY_SECONDS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .expect("RATE_LIMIT_BASE_DELAY_SECONDS must be a number");

        let rate_limit_multiplier = env::var("RATE_LIMIT_MULTIPLIER")
            .unwrap_or_else(|_| "2.0".to_string())
            .parse()
            .expect("RATE_LIMIT_MULTIPLIER must be a number");

        Config {
            port,
            redis_url,
            smtp_host,
            smtp_port,
            smtp_user,
            smtp_password,
            smtp_use_tls,
            from_address,
            jwt_private_key_path,
            jwt_public_key_path,
            jwt_expiration_hours,
            otp_expiration_minutes,
            otp_length,
            rate_limit_reset_seconds,
            max_verify_attempts,
            rate_limit_base_delay_seconds,
            rate_limit_multiplier,
        }
    }
}
