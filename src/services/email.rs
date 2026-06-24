use crate::{config::Config, error::AppError};
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, Message,
    SmtpTransport, Transport,
};
use askama::Template;

#[derive(Template)]
#[template(path = "otp.html")]
struct OtpEmailTemplate<'a> {
    otp_code: &'a str,
    expiration_minutes: u64,
}

#[derive(Clone)]
pub struct EmailService {
    mailer: SmtpTransport,
    from_address: String,
}

impl EmailService {
    pub fn new(config: &Config) -> Self {
        let creds = Credentials::new(config.smtp_user.clone(), config.smtp_pass.clone());
        
        let mailer = if config.smtp_host == "127.0.0.1" || config.smtp_host == "localhost" {
            // Local Mailpit/MailHog (no TLS)
            SmtpTransport::builder_dangerous(&config.smtp_host)
                .port(config.smtp_port)
                .build()
        } else {
            // Production (TLS)
            SmtpTransport::relay(&config.smtp_host)
                .expect("Failed to create SMTP relay")
                .port(config.smtp_port)
                .credentials(creds)
                .build()
        };

        Self {
            mailer,
            from_address: config.smtp_from.clone(),
        }
    }

    pub fn send_otp(&self, to_email: &str, otp_code: &str, expiration_minutes: u64) -> Result<(), AppError> {
        let template = OtpEmailTemplate {
            otp_code,
            expiration_minutes,
        };
        let html_body = template.render().map_err(|e| AppError::EmailError(format!("Template error: {}", e)))?;

        let email = Message::builder()
            .from(self.from_address.parse().expect("Invalid from address"))
            .to(to_email.parse().map_err(|_| AppError::EmailError("Invalid email".into()))?)
            .subject("Your One-Time Password")
            .header(ContentType::TEXT_HTML)
            .body(html_body)
            .map_err(|e| AppError::EmailError(e.to_string()))?;

        self.mailer
            .send(&email)
            .map_err(|e| AppError::EmailError(e.to_string()))?;

        Ok(())
    }
}
