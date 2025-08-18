# Multi-stage build para optimizar tamaño
FROM rust:1.75 as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    cmake \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Build dependencies (cached layer)
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source code
COPY src/ src/
COPY migrations/ migrations/

# Build application
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder stage
COPY --from=builder /app/target/release/tracking-consumer /usr/local/bin/tracking-consumer

# Copy default config (will be overridden by volume mount)
COPY config/ config/

# Create log directory
RUN mkdir -p /var/log/tracking-consumer

# Create non-root user
RUN useradd -r -s /bin/false tracking && \
    chown -R tracking:tracking /var/log/tracking-consumer

USER tracking

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD pgrep tracking-consumer || exit 1

EXPOSE 8080

CMD ["tracking-consumer"]