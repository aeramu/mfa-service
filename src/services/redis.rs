use crate::error::AppError;
use deadpool_redis::{redis::AsyncCommands, Config as RedisConfig, Pool, Runtime};

#[derive(Clone)]
pub struct RedisService {
    pool: Pool,
}

impl RedisService {
    pub fn new(redis_url: &str) -> Self {
        let cfg = RedisConfig::from_url(redis_url);
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .expect("Failed to create Redis pool");
        Self { pool }
    }

    pub async fn check_rate_limit(&self, email: &str, reset_seconds: u64) -> Result<u64, AppError> {
        let mut conn = self.pool.get().await?;
        let attempts_key = format!("ratelimit:attempts:{}", email);
        let cooldown_key = format!("ratelimit:cooldown:{}", email);
        
        // 1. Check if currently in cooldown
        let cooldown_ttl: i64 = conn.ttl(&cooldown_key).await?;
        if cooldown_ttl > 0 {
            return Err(AppError::RateLimited { reset_in_seconds: cooldown_ttl as u64 });
        }
        
        // 2. Increment attempts counter
        let count: u32 = conn.incr(&attempts_key, 1).await?;
        
        // Reset the attempts counter memory after the configured reset time
        let _: () = conn.expire(&attempts_key, reset_seconds as i64).await?;
        
        // 3. Calculate new cooldown: 30s * 2^(count - 1)
        // count=1 -> 30s, count=2 -> 60s, count=3 -> 120s, etc.
        let base_delay = 30;
        let multiplier = 2_u32.pow((count.saturating_sub(1)).min(10)); // Cap shift at 10 to prevent huge lockouts
        let next_cooldown = base_delay * multiplier;
        
        // 4. Apply the cooldown lock
        let _: () = conn.set_ex(&cooldown_key, "1", next_cooldown as u64).await?;
        
        Ok(next_cooldown as u64)
    }

    pub async fn save_otp(&self, email: &str, code: &str, expiration_minutes: u64) -> Result<(), AppError> {
        let mut conn = self.pool.get().await?;
        let otp_key = format!("otp:{}", email);
        let _: () = conn.set_ex(&otp_key, code, expiration_minutes * 60).await?;
        Ok(())
    }

    pub async fn verify_otp(&self, email: &str, code: &str, max_verify_attempts: u32) -> Result<(), AppError> {
        let mut conn = self.pool.get().await?;
        
        // 1. Check verify rate limit: max attempts per minute
        let rate_limit_key = format!("ratelimit:verify:{}", email);
        let attempts: Option<u32> = conn.get(&rate_limit_key).await?;
        if let Some(attempts) = attempts {
            if attempts >= max_verify_attempts {
                let ttl: i64 = conn.ttl(&rate_limit_key).await?;
                return Err(AppError::RateLimited { reset_in_seconds: ttl.max(0) as u64 });
            }
        }
        
        // Increment verify attempts, expire after 60 seconds
        let count: u32 = conn.incr(&rate_limit_key, 1).await?;
        if count == 1 {
            let _: () = conn.expire(&rate_limit_key, 60).await?;
        }

        // 2. Fetch and verify OTP
        let otp_key = format!("otp:{}", email);
        let stored_code: Option<String> = conn.get(&otp_key).await?;
        
        match stored_code {
            Some(stored) if stored == code => {
                // Verification successful, delete the OTP to prevent reuse
                let _: () = conn.del(&otp_key).await?;
                // Reset the verify rate limit on success
                let _: () = conn.del(&rate_limit_key).await?;
                Ok(())
            }
            Some(_) => Err(AppError::InvalidOtp),
            None => Err(AppError::OtpExpired),
        }
    }
}
