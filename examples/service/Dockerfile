# ==========================================
# Multi-stage Dockerfile for thai-break-service (Alpine Base)
# ==========================================

# 1. Builder stage
FROM rust:1.98.1-alpine3.24 AS builder

# Build from the repository root, so the path dependency ../../rust is the engine of the same
# revision as this service:
#   docker build -f examples/service/Dockerfile -t thai-break-service .

WORKDIR /usr/src/thai-break

# Install required build tools on Alpine
RUN apk add --no-cache musl-dev build-base

# The engine (path dependency of the service)
COPY rust rust

WORKDIR /usr/src/thai-break/examples/service

# Copy dependency definitions
COPY examples/service/Cargo.toml examples/service/Cargo.lock ./

# Create dummy source files to pre-build and cache external dependencies
RUN mkdir -p src data && \
    echo "pub fn dummy() {}" > src/lib.rs && \
    echo "fn main() {}" > src/main.rs && \
    touch data/words.fst data/lines.fst

RUN cargo build --release && rm -rf src data

# Copy real application source code and FST dictionaries
COPY examples/service/src src
COPY examples/service/data data

# Touch source files to ensure cargo re-compiles our code
RUN touch src/lib.rs src/main.rs && cargo build --release

# 2. Production runtime stage (Ultra-minimal Alpine 3.24)
FROM alpine:3.24 AS runner

# Install runtime certificates and timezone data (curl omitted for minimal attack surface)
RUN apk add --no-cache \
    ca-certificates \
    tzdata

# Run as non-root user for container security
RUN adduser -D -u 10001 -s /bin/sh appuser

WORKDIR /app

# Copy compiled binary and FST dictionaries
COPY --from=builder --chown=appuser:appuser /usr/src/thai-break/examples/service/target/release/thai-break-service /usr/local/bin/thai-break-service
COPY --from=builder --chown=appuser:appuser /usr/src/thai-break/examples/service/data /app/data

USER appuser:appuser

ENV HOST=0.0.0.0 \
    PORT=8080 \
    RUST_LOG=info \
    THAIBREAK_WORDS_FST_PATH=/app/data/words.fst \
    THAIBREAK_LINES_FST_PATH=/app/data/lines.fst

EXPOSE 8080

HEALTHCHECK --interval=15s --timeout=3s --start-period=5s --retries=3 \
  CMD wget -qO- http://127.0.0.1:8080/health || exit 1

ENTRYPOINT ["thai-break-service"]
