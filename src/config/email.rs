use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use once_cell::sync::Lazy;
use std::env;

// Email configuration
pub static SMTP_HOST: Lazy<String> = Lazy::new(|| {
    env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string())
});

pub static SMTP_PORT: Lazy<u16> = Lazy::new(|| {
    env::var("SMTP_PORT")
        .unwrap_or_else(|_| "587".to_string())
        .parse()
        .unwrap_or(587)
});

pub static SMTP_USERNAME: Lazy<String> = Lazy::new(|| {
    env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set")
});

pub static SMTP_PASSWORD: Lazy<String> = Lazy::new(|| {
    env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set")
});

pub static FROM_EMAIL: Lazy<String> = Lazy::new(|| {
    env::var("FROM_EMAIL").unwrap_or_else(|| SMTP_USERNAME.clone())
});

pub static FROM_NAME: Lazy<String> = Lazy::new(|| {
    env::var("FROM_NAME").unwrap_or_else(|| "Axum Auth System".to_string())
});

// Create SMTP transport
pub fn create_smtp_transport() -> AsyncSmtpTransport<Tokio1Executor> {
    let credentials = Credentials::new(SMTP_USERNAME.clone(), SMTP_PASSWORD.clone());

    AsyncSmtpTransport::<Tokio1Executor>::relay(&SMTP_HOST)
        .unwrap()
        .port(*SMTP_PORT)
        .credentials(credentials)
        .build()
}

// Send email function
pub async fn send_email(
    to_email: &str,
    to_name: &str,
    subject: &str,
    body: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let email = Message::builder()
        .from(format!("{} <{}>", *FROM_NAME, *FROM_EMAIL).parse()?)
        .to(format!("{} <{}>", to_name, to_email).parse()?)
        .subject(subject)
        .body(body.to_string())?;

    let mailer = create_smtp_transport();
    mailer.send(email).await?;

    Ok(())
}

// Send OTP email specifically
pub async fn send_otp_email(
    to_email: &str,
    to_name: &str,
    otp: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let subject = "Password Reset OTP - Axum Auth";
    let body = format!(
        "Hello {},

You have requested to reset your password. Please use the following OTP code:

OTP: {}

This code will expire in 10 minutes for security reasons.

If you did not request this password reset, please ignore this email.

Best regards,
Axum Auth Team",
        to_name, otp
    );

    send_email(to_email, to_name, subject, &body).await
}
