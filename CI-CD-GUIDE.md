# 🚀 CI/CD and Docker Deployment Guide

This document explains the complete CI/CD pipeline and Docker deployment setup for the Axum authentication service.

## 📁 Overview

The project now includes:

- **Multi-stage Dockerfile** for lean production builds
- **Docker Compose** for local development and production
- **GitHub Actions CI/CD** with automated testing and deployment
- **Nginx reverse proxy** with rate limiting and security headers
- **VPS deployment automation** with health checks

## 🐳 Docker Setup

### Development

```bash
# Start all services for development
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### Production

```bash
# Start with production configuration
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# Scale the application (if needed)
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d --scale app=3
```

## 🔧 Environment Configuration

### Required Environment Variables

Create a `.env` file with the following variables:

```bash
# Database
DATABASE_URL=postgresql://username:password@host:5432/database
DB_NAME=axum_auth
DB_USER=postgres
DB_PASSWORD=your_secure_password

# Redis
REDIS_URL=redis://:password@host:6379
REDIS_PASSWORD=your_redis_password
REDIS_PORT=6379

# JWT
JWT_SECRET=your_very_secure_jwt_secret_key_here

# SMTP Email Configuration
SMTP_HOST=your.smtp.server.com
SMTP_PORT=587
SMTP_USERNAME=your_email@domain.com
SMTP_PASSWORD=your_email_password
FROM_EMAIL=noreply@yourdomain.com
FROM_NAME=Your App Name

# Application
APP_PORT=3001
FRONTEND_LOGIN_URL=https://yourfrontend.com/login
RUST_LOG=info
```

## 🏗️ CI/CD Pipeline

### Pipeline Stages

1. **Test and Build**

   - Code formatting check (`cargo fmt`)
   - Linting with Clippy (`cargo clippy`)
   - Unit and integration tests (`cargo test`)
   - Release build (`cargo build --release`)

2. **Security Audit**

   - Dependency vulnerability scanning (`cargo audit`)

3. **Docker Build**

   - Multi-stage build for lean images
   - Push to GitHub Container Registry
   - Multi-platform builds (AMD64, ARM64)

4. **Security Audit**

   - Dependency vulnerability scanning (`cargo audit`)

5. **Deploy to VPS**
   - Docker image built directly on VPS (no registry needed)
   - Multi-stage build for lean images
   - Automated deployment with health checks
   - Rollback capability

### GitHub Secrets Required

Configure these secrets in your GitHub repository:

```
HOST_IP           # Your VPS IP address
USERNAME          # SSH username
PRIVATE_KEY       # Private SSH key content
PORT              # SSH port (default: 22)
```

## 🖥️ VPS Setup

### Initial VPS Configuration

1. **Run the setup script on your VPS:**

```bash
# Download and run the setup script
curl -sSL https://raw.githubusercontent.com/your-username/your-repo/main/deploy/setup-vps.sh | bash
```

2. **Or manually clone and run:**

```bash
git clone https://github.com/your-username/your-repo.git
cd your-repo
chmod +x deploy/setup-vps.sh
./deploy/setup-vps.sh
```

### What the setup script does:

- ✅ Installs Docker and Docker Compose
- ✅ Creates application directory (`/opt/axum-app`)
- ✅ Clones your repository
- ✅ Sets up firewall rules
- ✅ Creates systemd service for auto-restart
- ✅ Prepares SSL certificate directory

### Manual VPS Configuration

If you prefer manual setup:

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# Create app directory
sudo mkdir -p /opt/axum-app
sudo chown $USER:$USER /opt/axum-app
cd /opt/axum-app

# Clone repository
git clone https://github.com/your-username/your-repo.git .

# Configure environment
cp .env.example .env
nano .env  # Edit with your settings
```

## 🔐 Security Features

### Nginx Security Headers

- `X-Frame-Options: SAMEORIGIN`
- `X-XSS-Protection: 1; mode=block`
- `X-Content-Type-Options: nosniff`
- `Content-Security-Policy`

### Rate Limiting

- API endpoints: 10 requests/second
- Auth endpoints: 5 requests/second
- Configurable burst limits

### Container Security

- Non-root user in containers
- Minimal base images (Debian Slim)
- Resource limits in production
- Health checks for all services

## 📊 Monitoring and Health Checks

### Health Check Endpoint

```bash
curl http://your-domain.com/health
```

Response:

```json
{
  "success": true,
  "message": "Service is healthy",
  "data": {
    "status": "healthy",
    "timestamp": "2025-08-10T15:30:00Z",
    "services": {
      "redis": "healthy",
      "application": "healthy"
    },
    "version": "0.1.0"
  }
}
```

### Docker Health Checks

All services include health checks:

- **PostgreSQL**: `pg_isready`
- **Redis**: `redis-cli ping`
- **Application**: HTTP health endpoint
- **Nginx**: Default upstream health

### Monitoring Commands

```bash
# Check service status
docker-compose ps

# View application logs
docker-compose logs -f app

# Check resource usage
docker stats

# System service status
sudo systemctl status axum-app
```

## 🚦 Deployment Process

### Automatic Deployment (via GitHub Actions)

1. **Push to main branch**
2. **CI/CD pipeline runs automatically:**
   - Tests pass ✅
   - Security audit passes ✅
   - Docker image builds ✅
   - Deploys to VPS ✅
   - Health check passes ✅

### Manual Deployment

```bash
# On your VPS
cd /opt/axum-app

# Pull latest changes
git pull origin main

# Update and restart services
docker-compose -f docker-compose.yml -f docker-compose.prod.yml pull
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# Check health
curl -f http://localhost:3001/health
```

## 🔄 Rollback Process

If deployment fails:

```bash
# Check last working image
docker images

# Rollback to previous version
docker tag ghcr.io/your-username/your-repo:previous-sha ghcr.io/your-username/your-repo:latest
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# Or rollback git and redeploy
git reset --hard HEAD~1
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d --build
```

## 📈 Scaling

### Horizontal Scaling

```bash
# Scale application instances
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d --scale app=3

# Load balancer will distribute traffic automatically
```

### Resource Limits

Configured in `docker-compose.prod.yml`:

- **App**: 512MB RAM, 1 CPU
- **PostgreSQL**: 256MB RAM, 0.5 CPU
- **Redis**: 128MB RAM, 0.25 CPU

## 🐛 Troubleshooting

### Common Issues

1. **Port conflicts**

```bash
# Check what's using port 3001
sudo netstat -tulpn | grep 3001
sudo lsof -i :3001
```

2. **Permission issues**

```bash
# Fix Docker permissions
sudo usermod -aG docker $USER
newgrp docker
```

3. **Environment variables not loading**

```bash
# Check .env file
cat .env
# Restart services
docker-compose down && docker-compose up -d
```

4. **Database connection issues**

```bash
# Check PostgreSQL logs
docker-compose logs postgres

# Test connection
docker-compose exec postgres psql -U postgres -d axum_auth -c "SELECT 1;"
```

### Log Locations

```bash
# Application logs
docker-compose logs app

# Database logs
docker-compose logs postgres

# Redis logs
docker-compose logs redis

# Nginx logs
docker-compose logs nginx

# System logs
journalctl -u axum-app
```

## 🎯 Performance Optimization

### Docker Image Optimization

- Multi-stage builds reduce image size by ~90%
- Cargo dependency caching
- Minimal runtime dependencies
- Distroless base images for security

### Database Optimization

- Connection pooling
- Prepared statements
- Indexed queries
- Regular VACUUM operations

### Caching Strategy

- Redis for session storage
- Application-level caching
- Nginx static file caching
- CDN for static assets (recommended)

## 🔗 Additional Resources

- [Docker Best Practices](https://docs.docker.com/develop/dev-best-practices/)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Nginx Configuration Guide](https://nginx.org/en/docs/)
- [PostgreSQL Performance Tuning](https://wiki.postgresql.org/wiki/Performance_Optimization)
- [Redis Configuration](https://redis.io/topics/config)

---

## 📝 Quick Commands Reference

```bash
# Development
docker-compose up -d                    # Start all services
docker-compose logs -f app              # View app logs
docker-compose exec app sh              # Shell into app container

# Production
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d  # Start production
systemctl status axum-app               # Check service status
journalctl -u axum-app -f               # Follow service logs

# Maintenance
docker-compose pull                     # Update images
docker system prune -f                 # Clean up unused resources
docker-compose down --volumes          # Stop and remove volumes
```
