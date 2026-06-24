use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub redis_url: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub smtp_from: String,
    pub otp_expiration_minutes: u64,
    pub otp_length: usize,
    pub rate_limit_reset_seconds: u64,
    pub max_verify_attempts: u32,
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
        let smtp_pass = env::var("SMTP_PASSWORD").unwrap_or_default();
        let smtp_from = env::var("SMTP_FROM").expect("SMTP_FROM must be set");

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

        Config {
            port,
            redis_url,
            smtp_host,
            smtp_port,
            smtp_user,
            smtp_pass,
            smtp_from,
            otp_expiration_minutes,
            otp_length,
            rate_limit_reset_seconds,
            max_verify_attempts,
        }
    }
}
