use chrono::{DateTime, Utc};
use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::{
        authentication::Credentials,
        client::{Tls, TlsParameters},
    },
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use std::env;
use std::path::Path;
use tera::{Context, Tera};

pub async fn send_html_email(
    to_email: &str,
    to_name: &str,
    subject: &str,
    html_body: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get SMTP configuration from environment (use same logic as existing email service)
    let smtp_host = env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string());
    let smtp_port: u16 = env::var("SMTP_PORT")
        .unwrap_or_else(|_| "587".to_string())
        .parse()
        .unwrap_or(587);
    let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
    let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
    let from_email = env::var("FROM_EMAIL").unwrap_or_else(|_| smtp_username.clone());
    let from_name = env::var("FROM_NAME").unwrap_or_else(|_| "SecureAuth".to_string());

    println!("🔧 HTML Email SMTP Configuration:");
    println!("   Server: {}:{}", smtp_host, smtp_port);
    println!("   Username: {}", smtp_username);
    println!("   From: {} <{}>", from_name, from_email);

    // Create email message
    let email = Message::builder()
        .from(format!("{} <{}>", from_name, from_email).parse::<Mailbox>()?)
        .to(format!("{} <{}>", to_name, to_email).parse::<Mailbox>()?)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html_body.to_string())?;

    // Create SMTP transport using the same robust configuration as the existing email service
    let credentials = Credentials::new(smtp_username, smtp_password);

    println!("🔧 Creating robust SMTP transport for HTML email");

    // Create TLS parameters that accept self-signed certificates (same as existing service)
    let tls_parameters = TlsParameters::builder(smtp_host.clone())
        .dangerous_accept_invalid_certs(true)
        .dangerous_accept_invalid_hostnames(true)
        .build()?;

    // Use the same transport logic as the existing email service
    let transport = if smtp_port == 465 {
        // SSL/TLS (port 465)
        println!("🔧 Using SSL/TLS for port 465");
        AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_host)?
            .port(smtp_port)
            .credentials(credentials)
            .tls(Tls::Wrapper(tls_parameters))
            .timeout(Some(std::time::Duration::from_secs(30)))
            .build()
    } else if smtp_port == 587 || smtp_port == 25 {
        // STARTTLS (port 587) or plain (port 25)
        println!("🔧 Using STARTTLS for port {} (HTML email)", smtp_port);
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)?
            .port(smtp_port)
            .credentials(credentials)
            .tls(Tls::Required(tls_parameters))
            .timeout(Some(std::time::Duration::from_secs(30)))
            .build()
    } else {
        // Default relay for other ports
        println!("🔧 Using default relay for port {}", smtp_port);
        AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_host)?
            .port(smtp_port)
            .credentials(credentials)
            .timeout(Some(std::time::Duration::from_secs(30)))
            .build()
    };

    // Send email
    println!("📤 Sending HTML email to: {}", to_email);
    match transport.send(email).await {
        Ok(response) => {
            println!(
                "✅ HTML email sent successfully to {}: {:?}",
                to_email, response
            );
            Ok(())
        }
        Err(e) => {
            println!("❌ Failed to send HTML email to {}: {:?}", to_email, e);
            println!("🔍 Debug info:");
            println!("   SMTP Server: {}:{}", smtp_host, smtp_port);
            println!("   TLS Parameters: Self-signed certs accepted");
            Err(Box::new(e))
        }
    }
}

pub fn render_password_reset_success_email(
    name: &str,
    email: &str,
    reset_time: &str,
    login_url: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Read the HTML template
    let template_path = "src/views/password_reset_success.html";

    if !Path::new(template_path).exists() {
        return Err(format!("Template file not found: {}", template_path).into());
    }

    let tera = Tera::new("src/views/*.html")?;
    let mut context = Context::new();

    // Add template variables
    context.insert("name", name);
    context.insert("email", email);
    context.insert("reset_time", reset_time);
    context.insert("login_url", login_url);

    // Add current date and time
    let now: DateTime<Utc> = Utc::now();
    context.insert("current_date", &now.format("%B %d, %Y").to_string());
    context.insert("current_time", &now.format("%I:%M %p UTC").to_string());

    let rendered = tera.render("password_reset_success.html", &context)?;
    Ok(rendered)
}

pub async fn send_password_reset_success_email(
    to_email: &str,
    to_name: &str,
    reset_time: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let login_url = env::var("FRONTEND_LOGIN_URL")
        .unwrap_or_else(|_| "http://localhost:3000/login".to_string());

    let html_body = render_password_reset_success_email(to_name, to_email, reset_time, &login_url)?;

    send_html_email(
        to_email,
        to_name,
        "Password Reset Successful - SecureAuth",
        &html_body,
    )
    .await
}
