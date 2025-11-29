ARG RUST_VERSION=1.91.1
ARG APP_NAME=matrix-bot-help

FROM rust:${RUST_VERSION}-alpine AS build
ARG APP_NAME
WORKDIR /app

RUN apk add --no-cache clang lld musl-dev git

# Copy source files instead of using mounts
COPY src/ src/
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

RUN cargo build --locked --release && \
    cp ./target/release/$APP_NAME /bin/bot


FROM alpine:latest AS final

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Set working directory
WORKDIR /app

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

# Copy the executable from the "build" stage.
COPY --from=build /bin/bot /bin/

# Expose volume for config and data
VOLUME ["/app/config", "/app/data"]

# What the container should run when it is started.
CMD ["/bin/bot", "-c", "/app/config/bot.toml"]

