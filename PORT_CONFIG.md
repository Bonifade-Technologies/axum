# Port Configuration for VPS Deployment

This document explains the port configuration used to avoid conflicts with existing services on the VPS.

## Port Mappings

### Default Ports (for local development)
- PostgreSQL: 5432
- Redis: 6379
- Application: 3000

### Production Ports (for VPS deployment)
- PostgreSQL: 5433 (mapped to container port 5432)
- Redis: 6380 (mapped to container port 6379)
- Application: 3001

## Environment Files

### `.env` - Local development
Uses default ports and external database configuration.

### `.env.docker` - Docker development
Uses ports 5433 and 6380 for testing Docker setup locally.

### `.env.production` - VPS production deployment
Uses ports 5433 and 6380 to avoid conflicts with existing PostgreSQL and Redis instances on the VPS.

## Docker Compose Configuration

The `docker-compose.yml` file uses environment variables with fallback defaults:

```yaml
# PostgreSQL
ports:
  - "${DB_PORT:-5433}:5432"

# Redis  
ports:
  - "${REDIS_PORT:-6380}:6379"
```

This allows flexibility:
- If `DB_PORT` environment variable is set, it uses that value
- Otherwise, it defaults to 5433 for production to avoid port 5432 conflicts

## VPS Deployment

The CI/CD pipeline automatically:
1. Copies `.env.production` to `.env` during deployment
2. Uses ports 5433 and 6380 for external access
3. Maps these to standard container ports (5432, 6379) internally

## Accessing Services on VPS

From the VPS host system:
- PostgreSQL: `localhost:5433`
- Redis: `localhost:6380`  
- Application: `localhost:3001`

From within Docker containers:
- PostgreSQL: `postgres:5432`
- Redis: `redis:6379`
- Application: `app:3001`

## Security Notes

Remember to:
1. Update passwords in `.env.production` before deployment
2. Ensure VPS firewall allows access to ports 5433, 6380, 3001 if needed
3. Consider using Docker networks for internal communication
