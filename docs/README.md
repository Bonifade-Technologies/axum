## Validation Process for DTOs (Signup, Login, etc.)

This project uses the [`validator`](https://crates.io/crates/validator) crate to validate incoming request data (DTOs) for endpoints like signup and login.

### How Validation Works

- Each DTO (e.g., `SignupDto`, `LoginDto`) derives or implements the `Validate` trait.
- Field-level validation is specified using `#[validate(...)]` attributes (e.g., `length`, `email`).
- Custom validation logic (e.g., password confirmation, unique email) is implemented using custom functions or by manually checking in the handler.
- If validation fails, the API returns a response in the format:

```json
{
  "success": false,
  "message": "Validation failed",
  "errors": {
    "field": "error message"
  }
}
```

### Example: SignupDto

```rust
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct SignupDto {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,

    #[validate(email(message = "invalid email"), custom(function = "validate_unique_email"))]
    #[validate(length(min = 1, message = "email is required"))]
    pub email: String,

    #[validate(length(min = 6, message = "password must be at least 6 characters"))]
    pub password: String,

    #[validate(length(min = 6, message = "password confirmation is required"))]
    pub password_confirmation: String,

    #[validate(length(min = 10, max = 15, message = "phone must be between 10 and 15 characters"))]
    pub phone: String,
}
```

#### Custom Validation Example

```rust
fn validate_unique_email(email: &str) -> Result<(), validator::ValidationError> {
    // Replace with async DB check in handler for real uniqueness
    if email == "taken@example.com" {
        return Err(validator::ValidationError::new("email_taken")
            .with_message("email is already taken".into()));
    }
    Ok(())
}
```

#### Password Confirmation

Password and password_confirmation are checked for equality in a custom `Validate` implementation or in the handler.

#### Unique Email (Async DB Check)

Because database checks are async, unique email validation is performed in the handler after DTO validation:

```rust
// In your handler (pseudo-code):
if email_exists_in_db(&dto.email).await {
    return api_response::failure(
        Some("Validation failed"),
        Some({ "email": "email is already taken" }),
        Some(StatusCode::UNPROCESSABLE_ENTITY),
    );
}
```

### Error Response

All validation errors are returned in the standard API error format, with field-level error messages.

### See Also

- [`validator` crate docs](https://docs.rs/validator/)
- `src/dtos/auth_dto.rs` for DTO definitions
- `src/extractors/json_extractor.rs` for extractor logic

# Project Structure Documentation

This project was refactored to move most of the application logic from `main.rs` into the library crate (`lib.rs`). This makes the codebase more modular, testable, and reusable.

## Before Refactor

Previously, the main application logic was in `src/main.rs`:

```rust
use project_name_in_the_crate::run; //package name in the toml file

#[tokio::main]
async fn main() {
    run().await;
}
```

## After Refactor

Now, the main logic is in `src/lib.rs`, and `main.rs` simply calls into the library:

**src/main.rs**

```rust
use axum_template::run;

#[tokio::main]
async fn main() {
    run().await;
}
```

**src/lib.rs** (example structure)

```rust
use axum::{routing::get, Router};

pub mod config;
pub mod controllers;
pub mod models;
pub mod routes;
pub mod utils;
pub mod views;

pub async fn run() {
    // build our application with a single route
    let app = Router::new().route("/", get(hello_world));

    // Use APP_URL and APP_PORT static variables from config/database.rs
    let addr = format!(
        "{}:{}",
        *crate::config::database::APP_URL,
        *crate::config::database::APP_PORT
    );

    println!("Starting server at {}", &addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn hello_world() -> &'static str {
    "Hello, world!, Welcome to axum apis. This is a template for building web applications with Rust and Axum."
}

```

## Benefits

- **Separation of concerns:** Keeps the entry point minimal and moves logic to the library.
- **Testability:** You can now test your application logic by calling functions from `lib.rs` directly.
- **Reusability:** Other binaries or integration tests can reuse the same logic.

## How to Run

Build and run as usual:

```sh
cargo run
```

## Using dotenv with once_cell for Environment Variables

To manage environment variables easily, you can use the [`dotenv`](https://crates.io/crates/dotenvy) crate together with [`once_cell`](https://crates.io/crates/once_cell`). This allows you to load variables from a `.env` file and initialize them as static variables.

### 1. Add dependencies

In your `Cargo.toml`:

```toml
[dependencies]
dotenv = "0.15"
once_cell = "1.21"
```

```toml
[dependencies]
dotenv = "0.15"
once_cell = "1.21"
```

### 2. Create a `.env` file

```
APP_URL=0.0.0.0
APP_PORT=3000
```

### 3. Load dotenv in your code

Call `dotenv::dotenv().ok();` at the start of your program (e.g., in `main.rs` or at the top of `lib.rs`):

```rust
fn main() {
    dotenv::dotenv().ok();
    // ...rest of your code
}
```

Or, for async main:

```rust
#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    run().await;
}
```

### 4. Use once_cell for static environment variables

In your `src/config/database.rs`:

```rust
use std::env;
use once_cell::sync::Lazy;

pub static APP_URL: Lazy<String> = Lazy::new(|| env::var("APP_URL").unwrap());
pub static APP_PORT: Lazy<String> = Lazy::new(|| env::var("APP_PORT").unwrap());
```

### 5. Import and use in your code

```rust
let addr = format!(
    "{}:{}",
    *crate::config::database::APP_URL,
    *crate::config::database::APP_PORT
);
```

This setup ensures your environment variables are loaded from `.env` and available as statics throughout your app.

## API Response Traits

To standardize your API responses, this project provides two response types in `utils/api_response.rs`:

### SuccessResponse

Returned by the `success` function for successful API calls.

```rust
#[derive(Serialize)]
pub struct SuccessResponse<T: Serialize> {
    pub success: bool,         // Always true
    pub message: String,       // A message describing the result
    pub data: Option<T>,       // Optional data payload (any serializable type)
}
```

- `data` is only present in successful responses and omitted if `None`.

### FailureResponse

Returned by the `failure` function for failed API calls.

```rust
#[derive(Serialize)]
pub struct FailureResponse {
    pub success: bool,               // Always false
    pub message: String,             // Error message
    pub errors: Option<serde_json::Value>, // Optional error details
}
```

- `errors` is only present in failure responses and omitted if `None`.

### Usage Example

```rust
use crate::utils::api_response::{success, failure};

// Success response
success(Some("Fetched successfully"), Some(my_data), None);

// Failure response
failure(Some("Not found"), Some("Resource missing"), None);
```

All responses are returned as JSON and are compatible with Axum's `IntoResponse` trait for easy use in handlers.

## Common HTTP Status Codes

You can use any of these standard HTTP status codes with your API responses by passing `Some(StatusCode::XXX)` to the response helpers.

| Code | Name                  | Description                        |
| ---- | --------------------- | ---------------------------------- |
| 200  | OK                    | Standard success                   |
| 201  | CREATED               | Resource created                   |
| 202  | ACCEPTED              | Request accepted, processing later |
| 204  | NO_CONTENT            | Success, no content returned       |
| 400  | BAD_REQUEST           | Invalid request                    |
| 401  | UNAUTHORIZED          | Authentication required            |
| 403  | FORBIDDEN             | Not allowed                        |
| 404  | NOT_FOUND             | Resource not found                 |
| 409  | CONFLICT              | Conflict with current state        |
| 422  | UNPROCESSABLE_ENTITY  | Validation error                   |
| 429  | TOO_MANY_REQUESTS     | Rate limit exceeded                |
| 500  | INTERNAL_SERVER_ERROR | Server error                       |
| 502  | BAD_GATEWAY           | Invalid response from upstream     |
| 503  | SERVICE_UNAVAILABLE   | Service temporarily unavailable    |

For the full list, see the [axum::http::StatusCode docs](https://docs.rs/http/latest/http/status/struct.StatusCode.html).

---

For more details, see the code in `src/lib.rs` and `src/main.rs`.

## Group Routing and Fallback (Catch-All) Handler in Axum

### Group Routing

You can organize your routes into groups (sub-routers) for better structure. For example, to group all sample routes under `/samples`:

```rust
use crate::routes::samples::samples_router;
use axum::{Router, routing::get};

pub fn app_router() -> Router {
    Router::new()
        .route("/", get(hello_world))
        .nest("/samples", samples_router())
}
```

- `.nest("/samples", samples_router())` mounts all routes from `samples_router()` under the `/samples` path.

### Fallback (Catch-All) Handler

To handle any route that is not defined, use the `.fallback()` method:

```rust
async fn not_found() -> impl axum::response::IntoResponse {
    api_response::failure(
        Some("Route not found"),
        Some("The requested endpoint does not exist."),
        None,
    )
}

pub fn app_router() -> Router {
    Router::new()
        .route("/", get(hello_world))
        .nest("/samples", samples_router())
        .fallback(not_found)
}
```

- Any request to an undefined route will trigger the `not_found` handler, returning a JSON error response.

This approach keeps your API organized and user-friendly, always returning a clear message for unknown endpoints.

## SeaORM Usage: Pagination, Sorting, and Case-Insensitive Search

This project uses [SeaORM](https://www.sea-ql.org/SeaORM/) as the ORM for database access. Below are some key patterns and customizations used in the codebase:

### Pagination and Sorting

The user listing endpoint supports pagination and sorting using SeaORM's paginator and order_by methods:

```rust
let paginator = query.paginate(&db, per_page);
let total = paginator.num_items().await.unwrap_or(0);
let users = paginator.fetch_page(page - 1).await;

// Sorting
query = match sort_by.as_str() {
    "name" => {
        if sort_order == "asc" {
            query.order_by_asc(user::Column::Name)
        } else {
            query.order_by_desc(user::Column::Name)
        }
    }
    "email" => {
        if sort_order == "asc" {
            query.order_by_asc(user::Column::Email)
        } else {
            query.order_by_desc(user::Column::Email)
        }
    }
    _ => {
        if sort_order == "asc" {
            query.order_by_asc(user::Column::CreatedAt)
        } else {
            query.order_by_desc(user::Column::CreatedAt)
        }
    }
};
```

### Case-Insensitive Search (Postgres)

Postgres `LIKE` is case-sensitive by default. To enable case-insensitive search for fields like `name` and `email`, this project uses custom SQL expressions with the `LOWER()` function:

```rust
use sea_orm::sea_query::Expr;
let search_term = format!("%{}%", s.to_lowercase());
query = query.filter(
    Expr::cust("LOWER(name)").like(&search_term)
        .or(Expr::cust("LOWER(email)").like(&search_term)),
);
```

This ensures that both the column data and the search term are compared in lowercase, providing a user-friendly search experience.

### References

- See `src/controllers/users.rs` for the full implementation of the user listing endpoint.
- See `src/utils/api_response.rs` for pagination info and response helpers.

---

# Redis Caching for User List Endpoint

This project uses Redis for caching the user list endpoint with support for query params, a 24-hour TTL, and efficient cache invalidation on create, update, or delete events.

## Redis Configuration

- Redis config is managed in `src/config/redis.rs`:

```rust
use once_cell::sync::Lazy;
use redis::Client;
use std::env;

pub static REDIS_URL: Lazy<String> = Lazy::new(|| {
    env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string())
});

pub fn redis_client() -> Client {
    Client::open(REDIS_URL.as_str()).expect("Failed to create Redis client")
}
```

- Set your Redis URL in `.env`:

```
REDIS_URL=redis://127.0.0.1/
```

## Caching Utility

- The cache utility is in `src/utils/cache.rs` and supports:
  - Caching any serializable value with a TTL of 24 hours
  - Cache key includes query params for correct pagination/sorting/search
  - Automatic cache invalidation by prefix (e.g., after create/update/delete)

```rust
use redis::{AsyncCommands, Client};
use serde::{Serialize, de::DeserializeOwned};

const CACHE_TTL_SECONDS: usize = 60 * 60 * 24; // 24 hours

pub async fn get_or_set_cache<T, F, Fut>(
    client: &Client,
    key: &str,
    query_params: &str,
    fetch_fn: F,
) -> redis::RedisResult<T>
where
    T: Serialize + DeserializeOwned + Clone,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    // ... see code for details ...
}

pub async fn invalidate_cache_by_prefix(client: &Client, prefix: &str) -> redis::RedisResult<()> {
    // ... see code for details ...
}
```

## Usage Pattern

- When listing users, use `get_or_set_cache` with a cache key and the query params stringified (e.g., page, per_page, search, sort_by, sort_order).
- On create, update, or delete, call `invalidate_cache_by_prefix` with the cache key prefix to clear all related cached pages.

---

# Redis Caching: Usage in User Endpoints

This project uses Redis to cache user data for performance and scalability. Caching is implemented for both the user list and single user endpoints, with automatic invalidation on create, update, and delete events.

## How Caching Works

- **User List Endpoint**: Results are cached per unique combination of query params (pagination, search, sort, etc.) for 24 hours.
- **Single User Endpoint**: Each user's data is cached by their ID for 24 hours.
- **Cache Invalidation**: On any create, update, or delete, all user-related caches are invalidated to ensure fresh data.

## Example Usage

### List Users (with cache)

```rust
let query_params = serde_json::json!({
    "page": page,
    "per_page": per_page,
    "sort_by": sort_by,
    "sort_order": sort_order,
    "search": search
}).to_string();
let cache_key = "user_list";
let fetch_fn = || async {
    // ...fetch from DB and build response...
};
let data = get_or_set_cache(cache_key, &query_params, fetch_fn).await;
```

### Get Single User (with cache)

```rust
let cache_key = "user";
let query_params = &id;
let fetch_fn = || async {
    // ...fetch user from DB and map to UserResource...
};
let data = get_or_set_cache(cache_key, query_params, fetch_fn).await;
```

### Invalidate Cache on Write

```rust
// After create, update, or delete:
let _ = invalidate_cache_by_prefix("user").await;
```

## Where to Find the Code

- `src/utils/cache.rs`: Cache logic and helpers
- `src/controllers/users.rs`: Example usage in handlers
- `src/config/redis.rs`: Redis connection config

## Notes

- All cached data is serialized as JSON for compatibility and easy debugging.
- The cache key always includes query params for list endpoints, and user ID for single user endpoints.
- Invalidation is done by prefix, so all related cache entries are cleared on any write.

---
