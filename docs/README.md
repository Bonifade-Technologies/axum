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

To manage environment variables easily, you can use the [`dotenv`](https://crates.io/crates/dotenv) crate together with [`once_cell`](https://crates.io/crates/once_cell`). This allows you to load variables from a `.env` file and initialize them as static variables.

### 1. Add dependencies

In your `Cargo.toml`:

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
