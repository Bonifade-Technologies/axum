use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use once_cell::sync::Lazy;
use std::env;

// Email configuration
pub static SMTP_HOST: Lazy<String> =
    Lazy::new(|| env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string()));

pub static SMTP_PORT: Lazy<u16> = Lazy::new(|| {
    env::var("SMTP_PORT")
        .unwrap_or_else(|_| "587".to_string())
        .parse()
        .unwrap_or(587)
});

pub static SMTP_USERNAME: Lazy<String> =
    Lazy::new(|| env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set"));

pub static SMTP_PASSWORD: Lazy<String> =
    Lazy::new(|| env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set"));

pub static FROM_EMAIL: Lazy<String> =
    Lazy::new(|| env::var("FROM_EMAIL").unwrap_or_else(|_| SMTP_USERNAME.clone()));

pub static FROM_NAME: Lazy<String> =
    Lazy::new(|| env::var("FROM_NAME").unwrap_or_else(|_| "Axum Auth System".to_string()));

// Create SMTP transport with better error handling and certificate flexibility
pub fn create_smtp_transport(
) -> Result<AsyncSmtpTransport<Tokio1Executor>, Box<dyn std::error::Error + Send + Sync>> {
    use lettre::transport::smtp::client::{Tls, TlsParameters};

    let credentials = Credentials::new(SMTP_USERNAME.clone(), SMTP_PASSWORD.clone());

    println!(
        "🔧 DEBUG: Creating SMTP transport for {}:{}",
        *SMTP_HOST, *SMTP_PORT
    );
    println!("🔧 DEBUG: Username: {}", *SMTP_USERNAME);

    // Create TLS parameters that accept self-signed certificates
    let tls_parameters = TlsParameters::builder(SMTP_HOST.clone())
        .dangerous_accept_invalid_certs(true)
        .dangerous_accept_invalid_hostnames(true)
        .build()?;

    // Try different configurations based on port
    let transport = if *SMTP_PORT == 465 {
        // SSL/TLS (port 465)
        AsyncSmtpTransport::<Tokio1Executor>::relay(&SMTP_HOST)?
            .port(*SMTP_PORT)
            .credentials(credentials)
            .tls(Tls::Wrapper(tls_parameters))
            .timeout(Some(std::time::Duration::from_secs(30)))
            .build()
    } else if *SMTP_PORT == 587 || *SMTP_PORT == 25 {
        // STARTTLS (port 587) or plain (port 25)
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&SMTP_HOST)?
            .port(*SMTP_PORT)
            .credentials(credentials)
            .tls(Tls::Required(tls_parameters))
            .timeout(Some(std::time::Duration::from_secs(30)))
            .build()
    } else {
        // Default relay for other ports
        AsyncSmtpTransport::<Tokio1Executor>::relay(&SMTP_HOST)?
            .port(*SMTP_PORT)
            .credentials(credentials)
            .timeout(Some(std::time::Duration::from_secs(30)))
            .build()
    };

    Ok(transport)
}

// Send email function with better error handling
pub async fn send_email(
    to_email: &str,
    to_name: &str,
    subject: &str,
    body: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🔧 DEBUG: Attempting to send email to: {}", to_email);
    println!("🔧 DEBUG: SMTP Host: {}:{}", *SMTP_HOST, *SMTP_PORT);
    println!("🔧 DEBUG: FROM: {} <{}>", *FROM_NAME, *FROM_EMAIL);

    // Use plain text content type to avoid InvalidContentType error
    let email = Message::builder()
        .from(format!("{} <{}>", *FROM_NAME, *FROM_EMAIL).parse()?)
        .to(format!("{} <{}>", to_name, to_email).parse()?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body.to_string())?;

    let mailer = create_smtp_transport()?;

    println!("🔧 DEBUG: Sending email via SMTP...");
    match mailer.send(email).await {
        Ok(response) => {
            println!("✅ Email sent successfully: {:?}", response);
            Ok(())
        }
        Err(e) => {
            println!("❌ Failed to send email: {:?}", e);
            Err(Box::new(e))
        }
    }
}

// Test SMTP connection
pub async fn test_smtp_connection() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🔧 Testing SMTP connection...");
    let _mailer = create_smtp_transport()?;

    // Try to test the connection (lettre doesn't have a direct test method, so we'll create the transport)
    println!("✅ SMTP transport created successfully");
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
