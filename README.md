# Rust Axum Authentication API

A robust authentication system built with Rust, Axum web framework, Sea-ORM, and Redis for session management.

## Features

- **JWT Authentication** - Secure token-based authentication
- **Redis Caching** - Fast user session management with Redis-first lookups
- **Database Integration** - PostgreSQL with Sea-ORM
- **Validation** - Request validation with custom email uniqueness checks
- **Middleware Protection** - Route protection with JWT middleware
- **Structured Error Responses** - Field-specific error messages in JSON format

## Tech Stack

- **Web Framework**: Axum
- **Database**: PostgreSQL with Sea-ORM
- **Cache**: Redis
- **Authentication**: JWT (jsonwebtoken)
- **Password Hashing**: bcrypt
- **Validation**: validator crate
- **Migration**: Sea-ORM migration tools

## Project Structure

```
src/
├── main.rs                 # Application entry point
├── lib.rs                  # Library root
├── config/                 # Configuration modules
│   ├── database.rs         # Database connection setup
│   ├── redis.rs           # Redis client configuration
│   └── mod.rs
├── controllers/            # Request handlers
│   ├── auth_controller.rs  # Authentication endpoints
│   ├── users.rs           # User management
│   └── mod.rs
├── database/               # Database entities
│   ├── users.rs           # User entity model
│   ├── prelude.rs         # Common database imports
│   └── mod.rs
├── dtos/                   # Data Transfer Objects
│   ├── auth_dto.rs        # Authentication DTOs with validation
│   └── mod.rs
├── extractors/             # Custom extractors
│   ├── json_extractor.rs  # Validated JSON extractor
│   └── mod.rs
├── middlewares/            # Custom middleware
│   ├── auth_middleware.rs # JWT authentication middleware
│   └── mod.rs
├── models/                 # Business logic models
│   └── mod.rs
├── resources/              # API resource transformers
│   ├── user_resource.rs   # User response formatting
│   └── mod.rs
├── routes/                 # Route definitions
│   ├── auth.rs            # Authentication routes
│   ├── users.rs           # User routes
│   ├── samples.rs         # Sample/test routes
│   └── mod.rs
├── utils/                  # Utility functions
│   ├── auth.rs            # Authentication utilities
│   ├── api_response.rs    # Standardized API responses
│   ├── cache.rs           # Cache utilities
│   └── mod.rs
└── views/                  # View templates (if needed)
    └── mod.rs

migration/                  # Database migrations
├── src/
│   ├── lib.rs
│   ├── main.rs
│   └── m20220101_000001_create_table.rs
├── Cargo.toml
└── README.md

entity/                     # Generated entities
├── src/
└── Cargo.toml
```

## Setup Instructions

### Prerequisites

- Rust (latest stable version)
- PostgreSQL
- Redis
- Docker (optional, for containerized databases)

### Environment Variables

Create a `.env` file in the project root:

```env
# Database Configuration
DATABASE_URL=postgresql://username:password@localhost:5432/your_db_name

# Redis Configuration
REDIS_URL=redis://127.0.0.1:6379/

# JWT Configuration
JWT_SECRET=your_secure_jwt_secret_key_change_in_production

# Server Configuration
HOST=127.0.0.1
PORT=3000
```

### Installation

1. **Clone and setup the project:**

   ```bash
   git clone <your-repo-url>
   cd axum-auth
   ```

2. **Install dependencies:**

   ```bash
   cargo build
   ```

3. **Setup databases:**
   ```bash
   # Start PostgreSQL and Redis (using Docker)
   docker run --name postgres -e POSTGRES_PASSWORD=password -p 5432:5432 -d postgres
   docker run --name redis -p 6379:6379 -d redis
   ```

## Database Management

### Important Note

This project uses a Cargo workspace with the migration tools in a separate package. That's why we use `-p migration` instead of `--bin migration` in all commands below. This allows you to run migrations from the project root without changing directories.

### Running Migrations

**Apply all pending migrations (from project root):**

```bash
cargo run -p migration
```

**Apply migrations with specific database URL:**

```bash
DATABASE_URL=postgresql://user:pass@localhost/db cargo run -p migration
```

**For CI/CD pipelines (from project root):**

```bash
# Production migration
cargo run --release -p migration

# With environment variables
DATABASE_URL=$DB_URL cargo run --release -p migration
```

### Creating New Migrations

**Generate a new migration file (from project root):**

```bash
sea-orm-cli migrate generate <migration_name> -d migration
```

**Alternative using cargo:**

```bash
cargo run --bin migration -- generate <migration_name>
```

Example:

```bash
sea-orm-cli migrate generate add_user_profile_fields -d migration
```

**Edit the generated migration file in `migration/src/`:**

```rust
// Example: migration/src/m20240810_000001_add_user_profile_fields.rs
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(ColumnDef::new(Users::Avatar).string().null())
                    .add_column(ColumnDef::new(Users::Bio).text().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::Avatar)
                    .drop_column(Users::Bio)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Users {
    Table,
    Avatar,
    Bio,
}
```

**Register the new migration in `migration/src/lib.rs`:**

```rust
pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20240810_000001_add_user_profile_fields; // Add this line

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20240810_000001_add_user_profile_fields::Migration), // Add this line
        ]
    }
}
```

### Fresh Database Setup

**Reset database (drop all tables and rerun migrations) from project root:**

```bash
cargo run -p migration -- fresh
```

**Reset and seed with sample data:**

```bash
cargo run -p migration -- fresh --seed
```

**For CI/CD environments:**

```bash
# Production fresh setup
DATABASE_URL=$DB_URL cargo run --release -p migration -- fresh
```

### Rollback Migrations

**Rollback last migration (from project root):**

```bash
cargo run -p migration -- down
```

**Rollback specific number of migrations:**

```bash
cargo run -p migration -- down -n 2  # Rollback last 2 migrations
```

**For CI/CD:**

```bash
DATABASE_URL=$DB_URL cargo run --release -p migration -- down
```

### Check Migration Status

**Show migration status (from project root):**

```bash
cargo run -p migration -- status
```

**For CI/CD:**

```bash
DATABASE_URL=$DB_URL cargo run --release -p migration -- status
```

## Entity Generation

**Generate entities from database schema:**

```bash
# Install sea-orm-cli if not already installed
cargo install sea-orm-cli

# Generate entities
sea-orm-cli generate entity -o entity/src --database-url $DATABASE_URL
```

### 🚀 Quick Command Reference (CI/CD Ready)

All commands run from project root - no `cd` required:

```bash
# Apply migrations
cargo run -p migration

# Create new migration
sea-orm-cli migrate generate <name> -d migration

# Fresh database
cargo run -p migration -- fresh

# Check status
cargo run -p migration -- status

# Rollback
cargo run -p migration -- down
```

## Development Workflow

### Running the Application

**Development mode with auto-reload:**

```bash
cargo install cargo-watch
cargo watch -x run
```

**Production mode:**

```bash
cargo run --release
```

### Testing

**Run all tests:**

```bash
cargo test
```

**Run specific test:**

```bash
cargo test test_user_registration
```

**Run with output:**

```bash
cargo test -- --nocapture
```

### Code Quality

**Check code without building:**

```bash
cargo check
```

**Format code:**

```bash
cargo fmt
```

**Lint code:**

```bash
cargo clippy
```

## API Endpoints

### Authentication Routes

| Method | Endpoint         | Description       | Auth Required |
| ------ | ---------------- | ----------------- | ------------- |
| POST   | `/auth/register` | User registration | No            |
| POST   | `/auth/login`    | User login        | No            |
| GET    | `/auth/profile`  | Get user profile  | Yes           |

### Request/Response Examples

**Registration:**

```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "securepassword"
  }'
```

**Login:**

```bash
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "securepassword"
  }'
```

**Profile (requires JWT token):**

```bash
curl -X GET http://localhost:3000/auth/profile \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### Error Response Format

All errors follow a consistent structure:

```json
{
  "status": "error",
  "message": "Human readable message",
  "data": {
    "field_name": "Specific error message"
  }
}
```

Examples:

```json
// Email already exists
{
  "status": "error",
  "message": "User with email already exists",
  "data": {
    "email": "Email is already taken"
  }
}

// Invalid password
{
  "status": "error",
  "message": "Login failed",
  "data": {
    "password": "Invalid password"
  }
}
```

## Redis Integration

### Cache Strategy

The application uses Redis-first caching strategy:

1. **User Lookups**: Check Redis first, fallback to database
2. **Session Management**: Store JWT tokens and user data in Redis
3. **Cache Keys**:
   - User data: `user:{email}`
   - JWT tokens: `token:{jwt_token}`

### Cache TTL

- User sessions: 24 hours
- JWT tokens: 24 hours

### Redis Commands for Debugging

```bash
# Connect to Redis CLI
redis-cli

# Check if user exists
EXISTS user:user@example.com

# Get user data
GET user:user@example.com

# Check token
GET token:your_jwt_token

# List all user keys
KEYS user:*

# List all token keys
KEYS token:*

# Clear all cache
FLUSHALL
```

## CI/CD Pipeline Commands

All migration commands can be run from the project root, making them perfect for CI/CD pipelines:

### GitHub Actions Example

```yaml
name: Deploy
on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run Migrations
        env:
          DATABASE_URL: ${{ secrets.DATABASE_URL }}
        run: cargo run --release -p migration

      - name: Build Application
        run: cargo build --release
```

### Docker Deployment

```dockerfile
# In your Dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Run migrations in container
ENV DATABASE_URL=${DATABASE_URL}
RUN cargo run --release -p migration

# Start application
CMD ["./target/release/axum-template"]
```

### Key Commands for CI/CD

```bash
# Apply migrations (production)
DATABASE_URL=$DATABASE_URL cargo run --release -p migration

# Check migration status
DATABASE_URL=$DATABASE_URL cargo run --release -p migration -- status

# Fresh database setup (development/staging)
DATABASE_URL=$DATABASE_URL cargo run --release -p migration -- fresh

# Rollback if needed
DATABASE_URL=$DATABASE_URL cargo run --release -p migration -- down
```

## Troubleshooting

### Common Issues

**Database Connection Issues:**

```bash
# Check if PostgreSQL is running
pg_isready -h localhost -p 5432

# Check database exists
psql -h localhost -U username -l
```

**Redis Connection Issues:**

```bash
# Check if Redis is running
redis-cli ping

# Should return PONG
```

**Migration Issues:**

```bash
# Check migration status (from project root)
cargo run -p migration -- status

# Reset migrations if corrupted (from project root)
cargo run -p migration -- fresh
```

**JWT Issues:**

- Ensure JWT_SECRET is set in environment
- Check token expiration (24 hours default)
- Verify token format in Authorization header: `Bearer <token>`

### Logs and Debugging

**Enable debug logging:**

```bash
RUST_LOG=debug cargo run
```

**Database query logging:**

```bash
RUST_LOG=sea_orm::query=debug cargo run
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Run tests: `cargo test`
4. Format code: `cargo fmt`
5. Run linter: `cargo clippy`
6. Submit a pull request

## License

[MIT License](LICENSE)
