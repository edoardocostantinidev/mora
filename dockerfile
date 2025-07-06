# Build stage
FROM rust:1.88.0-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy all crate Cargo.toml files first for better caching
COPY mora-core/Cargo.toml ./mora-core/
COPY mora-queue/Cargo.toml ./mora-queue/
COPY mora-api/Cargo.toml ./mora-api/
COPY mora-server/Cargo.toml ./mora-server/
COPY mora-client/Cargo.toml ./mora-client/
COPY mora-cli/Cargo.toml ./mora-cli/
COPY mora-channel/Cargo.toml ./mora-channel/

# Create dummy source files to cache dependencies
RUN mkdir -p mora-core/src && echo "fn main() {}" > mora-core/src/lib.rs
RUN mkdir -p mora-queue/src && echo "fn main() {}" > mora-queue/src/lib.rs
RUN mkdir -p mora-api/src && echo "fn main() {}" > mora-api/src/lib.rs
RUN mkdir -p mora-server/src && echo "fn main() {}" > mora-server/src/main.rs
RUN mkdir -p mora-client/src && echo "fn main() {}" > mora-client/src/lib.rs
RUN mkdir -p mora-cli/src && echo "fn main() {}" > mora-cli/src/main.rs
RUN mkdir -p mora-channel/src && echo "fn main() {}" > mora-channel/src/lib.rs

# Build dependencies (this will be cached)
RUN cargo build --release --bin mora-server

# Copy the actual source code
COPY . .

# Touch the files to ensure they are newer than the dummy files
RUN find . -name "*.rs" -exec touch {} \;

# Build the actual application
RUN cargo build --release --bin mora-server

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -r -u 1000 mora

# Copy the binary from builder stage
COPY --from=builder /app/target/release/mora-server /usr/local/bin/mora-server

# Change ownership of the binary
RUN chown mora:mora /usr/local/bin/mora-server

# Switch to non-root user
USER mora

# Expose the port
EXPOSE 2626

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:2626/health || exit 1

CMD ["mora-server"]
