FROM rust:1.73-slim as builder

WORKDIR /app

# 复制 Cargo.toml 和 Cargo.lock
COPY Cargo.toml Cargo.lock ./
COPY auth-service/Cargo.toml ./auth-service/
COPY common/Cargo.toml ./common/
COPY crypto/Cargo.toml ./crypto/

# 创建空的src/main.rs以构建依赖项
RUN mkdir -p auth-service/src common/src crypto/src \
    && echo "fn main() {}" > auth-service/src/main.rs \
    && echo "pub fn dummy() {}" > common/src/lib.rs \
    && echo "pub fn dummy() {}" > crypto/src/lib.rs \
    && cargo build --release -p auth-service

# 复制实际代码
COPY auth-service/src ./auth-service/src
COPY common/src ./common/src
COPY crypto/src ./crypto/src
COPY proto ./proto

# 删除假的编译产物，重新构建
RUN rm -f target/release/deps/auth_service* \
    && rm -f target/release/deps/libcommon* \
    && rm -f target/release/deps/libcrypto* \
    && cargo build --release -p auth-service

# 使用更小的镜像来运行
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 复制构建好的可执行文件
COPY --from=builder /app/target/release/auth-service /app/auth-service

# 创建配置目录
RUN mkdir -p /config

# 暴露服务端口和健康检查端口
EXPOSE 50051 50052

# 设置默认配置路径
ENV CONFIG_PATH="/config/config.yaml"

# 启动命令
CMD ["/app/auth-service", "--config", "/config/config.yaml", "--k8s-config"] 