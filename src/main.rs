mod config;
mod error;
mod handlers;
mod models;
mod services;

use axum::{
    http::{header, HeaderValue},
    routing::{post, get},
    Router,
};
use std::sync::Arc;
use tower_http::{
    cors::CorsLayer,
    set_header::SetResponseHeaderLayer,
};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use config::Config;
use handlers::{generate_otp, verify_otp, get_public_key, AppState};
use services::{email::EmailService, redis::RedisService};

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::generate_otp,
        handlers::verify_otp,
        handlers::get_public_key
    ),
    components(
        schemas(
            models::GenerateOtpRequest, 
            models::GenerateOtpResponse, 
            models::VerifyOtpRequest, 
            models::VerifyOtpResponse,
            models::ErrorResponse,
            models::RateLimitResponse,
            models::ValidationErrorResponse
        )
    ),
    tags(
        (name = "OTP", description = "Email OTP generation and verification endpoints")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing (logging)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mfa_service=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables (ignore error if .env is missing in production)
    let _ = dotenvy::dotenv();

    // Load configuration
    let config = Config::from_env();

    // Initialize services
    let redis_service = RedisService::new(&config.redis_url);
    let email_service = EmailService::new(&config);

    // Load RSA Keys
    let jwt_private_key = tokio::fs::read(&config.jwt_private_key_path)
        .await
        .expect("Failed to read JWT_PRIVATE_KEY_PATH");
    let jwt_public_key = tokio::fs::read_to_string(&config.jwt_public_key_path)
        .await
        .expect("Failed to read JWT_PUBLIC_KEY_PATH");

    // Create application state
    let state = Arc::new(AppState {
        redis_service,
        email_service,
        config: config.clone(),
        jwt_private_key,
        jwt_public_key,
    });

    // Configure permissive CORS for all frontends
    let cors = CorsLayer::permissive();

    // Build our application with routes
    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/v1/otp/generate", post(generate_otp))
        .route("/api/v1/otp/verify", post(verify_otp))
        .route("/api/v1/otp/public-key", get(get_public_key))
        .with_state(state)
        .layer(cors)
        .layer(SetResponseHeaderLayer::overriding(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'"),
        ));

    // Start the server
    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("Server listening on {}", addr);
    
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    
    tracing::info!("Shutdown signal received, starting graceful shutdown");
}
