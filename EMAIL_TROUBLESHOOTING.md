# Email Testing and Troubleshooting Guide

## Current Error Analysis

**Error:** `InvalidMessage(InvalidContentType)`

This error typically occurs due to:

1. SMTP server doesn't accept the message format
2. Content-Type header issues
3. Authentication problems
4. TLS/SSL configuration issues

## Solutions Applied

### 1. Improved SMTP Configuration

- Added STARTTLS support for port 587
- Added proper timeout handling
- Added explicit ContentType headers

### 2. Debug Information

The system now shows:

- SMTP Host and Port
- Username being used
- Detailed error messages

### 3. Alternative Configurations

#### For Gmail (Working Example)

```env
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your_email@gmail.com
SMTP_PASSWORD=your_app_password  # Use App Password, not regular password
FROM_EMAIL=your_email@gmail.com
FROM_NAME=Axum Auth System
```

#### For Custom Servers (Current Setup)

```env
SMTP_HOST=mail.bowofade.com
SMTP_PORT=587
SMTP_USERNAME="noreply@bowofade.com"
SMTP_PASSWORD="Noreply@Pass@1996"
FROM_EMAIL="noreply@bowofade.com"
FROM_NAME="Axum Auth System"
```

## Troubleshooting Steps

### 1. Test SMTP Server Manually

```bash
# Test with telnet
telnet mail.bowofade.com 587

# Test with openssl for STARTTLS
openssl s_client -connect mail.bowofade.com:587 -starttls smtp
```

### 2. Check SMTP Server Requirements

- Does the server require specific authentication?
- Does it need PLAIN authentication vs LOGIN?
- Are there IP restrictions?
- Does it require specific headers?

### 3. Alternative Ports

Try different ports if 587 doesn't work:

- Port 25 (Plain SMTP)
- Port 465 (SSL/TLS)
- Port 2525 (Alternative)

### 4. Email Provider Settings

#### Popular SMTP Settings

```env
# Gmail
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587

# Outlook/Hotmail
SMTP_HOST=smtp-mail.outlook.com
SMTP_PORT=587

# Yahoo
SMTP_HOST=smtp.mail.yahoo.com
SMTP_PORT=587

# SendGrid
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
```

## Testing Email Functionality

### 1. Test with Known Working Provider

First, test with Gmail to verify the code works:

```env
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your_gmail@gmail.com
SMTP_PASSWORD=your_app_password
FROM_EMAIL=your_gmail@gmail.com
FROM_NAME=Test System
```

### 2. Test Forgot Password Endpoint

```bash
curl -X POST http://localhost:3003/auth/forgot-password \
  -H "Content-Type: application/json" \
  -d '{"email": "your_test_email@gmail.com"}'
```

### 3. Check Server Logs

Look for these debug messages:

- `🔧 DEBUG: Creating SMTP transport for...`
- `🔧 DEBUG: Attempting to send email to...`
- `✅ Email sent successfully` or `❌ Failed to send email`

## Next Steps

1. **Try Gmail first** to verify the email system works
2. **Contact your hosting provider** about SMTP requirements for mail.bowofade.com
3. **Check if authentication method needs to be different**
4. **Verify the email credentials are correct**

## Alternative: Use Email Service Provider

Consider using dedicated email services:

- **SendGrid** (Free tier: 100 emails/day)
- **Mailgun** (Free tier: 5,000 emails/month)
- **Amazon SES** (Pay per use)
- **Postmark** (Free tier: 100 emails/month)

These providers are more reliable and have better documentation.
