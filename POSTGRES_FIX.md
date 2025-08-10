# PostgreSQL Docker Configuration Fix

## Problem

The PostgreSQL container was showing "FATAL: role 'root' does not exist" errors because the environment configuration was mixed between local development and Docker deployment.

## Root Cause

1. The `.env` file was configured for external PostgreSQL server (`152.53.140.57`)
2. Docker containers were trying to use local PostgreSQL service
3. Environment variables weren't properly separated between local dev and Docker deployment

## Solution

### 1. Environment File Separation

- **`.env`** - For local development with external PostgreSQL
- **`.env.docker`** - For Docker deployment with containerized services

### 2. Docker Environment (`.env.docker`)

```bash
# Database Configuration (for Docker containers)
DB_HOST=postgres
DB_PORT=5432
DB_NAME=axum_auth
DB_USER=postgres
DB_PASSWORD=password
DATABASE_URL=postgres://postgres:password@postgres:5432/axum_auth

# Redis Configuration (for Docker containers)
REDIS_HOST=redis
REDIS_PORT=6379
REDIS_PASSWORD=password
REDIS_URL=redis://:password@redis:6379
```

### 3. CI/CD Pipeline Update

Updated `.github/workflows/ci-cd.yml` to:

- Copy `.env.docker` to `.env` during deployment
- Use single `docker-compose.yml` file (removed production override)
- Fixed health check endpoint from `:3003` to `:3003`

### 4. Docker Compose Fixes

- Fixed Redis health check (removed password from command)
- Ensured proper PostgreSQL user configuration
- Added proper network configuration

## Testing the Fix

### Local Testing (if Docker is available)

```bash
# Test configuration
cp .env.docker .env.test
docker compose --env-file .env.test config

# Run containers
docker compose --env-file .env.test up -d

# Check logs
docker compose logs postgres
docker compose logs redis
```

### VPS Deployment

The CI/CD pipeline will now:

1. Pull latest code
2. Copy `.env.docker` to `.env`
3. Build and start containers with correct configuration
4. Perform health check on correct port

## Expected Result

- PostgreSQL container starts with `postgres` user (not `root`)
- Application connects to `postgres://postgres:password@postgres:5432/axum_auth`
- Redis connects with password authentication
- Health checks pass on port 3003

## Verification Commands (on VPS)

```bash
# Check container status
docker ps

# Check PostgreSQL logs
docker logs axum_postgres

# Check Redis logs
docker logs axum_redis

# Check application logs
docker logs axum_app

# Test database connection
docker exec axum_postgres pg_isready -U postgres -d axum_auth
```
