# ── Stage 1: Build ────────────────────────────────────────────────────────────
FROM rust:1.78-alpine AS builder

WORKDIR /build

RUN apk add --no-cache musl-dev gcc

COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo 'fn main(){}' > src/main.rs
RUN cargo build --release 2>/dev/null; rm -f target/release/webapp

COPY src ./src
RUN touch src/main.rs && cargo build --release

# ── Stage 2: Runtime ──────────────────────────────────────────────────────────
FROM alpine:3.19

RUN apk add --no-cache ca-certificates

RUN adduser -D -u 1001 appuser
WORKDIR /app

COPY --from=builder /build/target/release/webapp ./webapp

RUN mkdir -p /data && chown -R appuser:appuser /data /app

USER appuser

EXPOSE 8080
ENV PORT=8080
ENV DB_PATH=/data/records.db

ENTRYPOINT ["./webapp"]
