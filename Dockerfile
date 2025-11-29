# Build stage
FROM rust:1.91.1-alpine as builder

# Install build dependencies
RUN apk add --no-cache musl-dev

# Set working directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Create dummy source files to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs

# Build dependencies
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Create app user
RUN addgroup -g 1001 -S appgroup && \
    adduser -u 1001 -S appuser -G appgroup

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/matrix-bot-help /usr/local/bin/matrix-bot-help

# Verify the binary exists and is executable
RUN ls -la /usr/local/bin/matrix-bot-help && chmod +x /usr/local/bin/matrix-bot-help

# Create directories for config and data
RUN mkdir -p /app/config /app/data && \
    chown -R appuser:appgroup /app

# Switch to non-root user
USER appuser

# Copy example config files
COPY --chown=appuser:appgroup bot.toml.example /app/config/
COPY --chown=appuser:appgroup bot-help.md.example /app/config/
COPY --chown=appuser:appgroup bot-help.html.example /app/config/
COPY --chown=appuser:appgroup bot-help.txt.example /app/config/

# Create default config if none exists
RUN if [ ! -f /app/config/bot.toml ]; then \
        cp /app/config/bot.toml.example /app/config/bot.toml; \
    fi

# Expose volume for config and data
VOLUME ["/app/config", "/app/data"]

# Set entrypoint and default command with error handling
ENTRYPOINT ["/bin/sh", "-c", "echo 'Starting matrix-bot-help...' && /usr/local/bin/matrix-bot-help --config /app/config/bot.toml 2>&1"]
CMD []

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD pgrep matrix-bot-help > /dev/null || exit 1
