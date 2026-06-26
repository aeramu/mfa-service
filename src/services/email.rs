use crate::{config::Config, error::AppError};
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, Message,
    SmtpTransport, Transport, transport::smtp::client::{Tls, TlsParameters},
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
        let mut builder = SmtpTransport::relay(&config.smtp_host)
            .expect("Failed to create SMTP relay")
            .port(config.smtp_port);

        if !config.smtp_use_tls {
            builder = builder.tls(Tls::None);
        } else if config.smtp_port == 465 {
            let tls_params = TlsParameters::new(config.smtp_host.clone())
                .expect("Failed to create TLS parameters");
            builder = builder.tls(Tls::Wrapper(tls_params));
        } else {
            let tls_params = TlsParameters::new(config.smtp_host.clone())
                .expect("Failed to create TLS parameters");
            builder = builder.tls(Tls::Required(tls_params));
        }

        if !config.smtp_user.is_empty() {
            let creds = Credentials::new(config.smtp_user.clone(), config.smtp_password.clone());
            builder = builder.credentials(creds);
        }

        let mailer = builder.build();

        Self {
            mailer,
            from_address: config.from_address.clone(),
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
