# 1. Build Stage
FROM rust:slim-bullseye AS builder

# Install build dependencies required by utoipa-swagger-ui build scripts
RUN apt-get update && apt-get install -y curl pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Create a new empty shell project
WORKDIR /usr/src/app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies
RUN mkdir src && \
    echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy real source code
COPY src ./src
COPY templates ./templates

# Build for release (touch main.rs to ensure cargo rebuilds it)
RUN touch src/main.rs && cargo build --release

# 2. Runtime Stage
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates libssl1.1 && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN groupadd -r appuser && useradd -r -g appuser appuser

WORKDIR /usr/local/bin

# Copy the compiled binary from the builder environment
COPY --from=builder /usr/src/app/target/release/mfa-service .

# Set permissions and ownership
RUN chmod +x mfa-service && chown appuser:appuser mfa-service

# Switch to the non-root user
USER appuser:appuser

# Expose port
EXPOSE 3000

# Run the binary
CMD ["./mfa-service"]
