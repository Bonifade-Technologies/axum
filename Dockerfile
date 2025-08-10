# Multi-stage Dockerfile for lean Rust production builds
# Stage 1: Build dependencies and cache them
FROM rust:1.75-slim as chef
WORKDIR /app
RUN cargo install cargo-chef

# Stage 2: Prepare recipe file for dependency caching
FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Build dependencies (cached layer)
FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# Install system dependencies needed for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 4: Build the application
COPY . .
RUN cargo build --release --bin axum-template

# Stage 5: Runtime - Create minimal final image
FROM debian:bookworm-slim as runtime

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user for security
RUN useradd -r -s /bin/false -m axum_user

# Set working directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/axum-template /app/axum-template

# Copy necessary files
COPY --from=builder /app/src/views /app/src/views

# Change ownership to non-root user
RUN chown -R axum_user:axum_user /app

# Switch to non-root user
USER axum_user

# Expose port
EXPOSE 3001

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3001/health || exit 1

# Run the application
CMD ["./axum-template"]
