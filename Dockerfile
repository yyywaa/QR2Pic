# Build stage
# 注意：不要加 --platform=$BUILDPLATFORM。加了会让 builder 用宿主机（arm64 Mac）架构跑，
# cargo build 就会产出 arm64 二进制，而 buildx --platform linux/amd64 只改最终镜像的标签，
# 结果就是"amd64 外壳 + arm64 内核"，x86 服务器上 exec format error。
# 去掉后 builder 跟随目标平台（amd64），编译产物才是 x86_64。
FROM rust:1.88-alpine AS builder
WORKDIR /app

RUN apk add --no-cache musl-dev

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

# Runtime stage
FROM alpine:3.19
WORKDIR /app

RUN apk add --no-cache ca-certificates curl

COPY --from=builder /app/target/release/qr2pic /usr/local/bin/qr2pic
COPY --from=builder /app/migrations /app/migrations

RUN mkdir -p /app/uploads && \
    adduser -D -u 1000 -g '' appuser && \
    chown -R appuser:appuser /app

USER appuser

EXPOSE 3000

CMD ["qr2pic"]
