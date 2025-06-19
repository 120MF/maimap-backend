# 构建阶段 - 使用Alpine作为构建基础镜像
FROM rust:1.86-alpine3.21 AS build
WORKDIR /app

# 安装构建依赖
RUN apk add --no-cache musl-dev pkgconfig openssl-dev perl make git

# 分层缓存构建依赖
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p crates/maimap-server/src crates/maimap-scrape/src crates/maimap-utils/src crates/maimap-derive/src
COPY crates/maimap-server/Cargo.toml ./crates/maimap-server/
COPY crates/maimap-scrape/Cargo.toml ./crates/maimap-scrape/
COPY crates/maimap-derive/Cargo.toml ./crates/maimap-derive/
COPY crates/maimap-utils/Cargo.toml ./crates/maimap-utils/

RUN echo "fn main() {}" > crates/maimap-server/src/main.rs && \
    echo "fn main() {}" > crates/maimap-scrape/src/main.rs && \
    echo "pub fn dummy() {}" > crates/maimap-utils/src/lib.rs && \
    echo 'extern crate proc_macro;' > crates/maimap-derive/src/lib.rs && \
    echo 'use proc_macro::TokenStream;' >> crates/maimap-derive/src/lib.rs && \
    echo '' >> crates/maimap-derive/src/lib.rs && \
    echo '#[proc_macro_derive(Dummy)]' >> crates/maimap-derive/src/lib.rs && \
    echo 'pub fn dummy_derive(_input: TokenStream) -> TokenStream {' >> crates/maimap-derive/src/lib.rs && \
    echo '    TokenStream::new()' >> crates/maimap-derive/src/lib.rs && \
    echo '}' >> crates/maimap-derive/src/lib.rs

# 复制实际源代码并重新构建
COPY crates ./crates/
RUN touch crates/maimap-server/src/main.rs crates/maimap-scrape/src/main.rs crates/maimap-utils/src/lib.rs crates/maimap-derive/src/lib.rs && \
    cargo build --release

# 运行阶段
FROM alpine:3.21

# 安装运行依赖
RUN apk add --no-cache ca-certificates curl chromium tzdata dcron mongodb-tools && \
    mkdir -p /etc/cron.d /app

# 设置时区
ENV TZ=Asia/Shanghai

# 创建cron任务
RUN echo "0 */5 * * * cd /app && /app/maimap-scrape > /proc/1/fd/1 2>&1" > /etc/cron.d/scraper-cron && \
    chmod 0644 /etc/cron.d/scraper-cron && \
    crontab /etc/cron.d/scraper-cron

# 指定chromium路径
ENV CHROME_PATH=/usr/bin/chromium-browser
ENV CHROME_DEVEL_SANDBOX=/usr/bin/chrome-devel-sandbox

WORKDIR /app

ARG GITHUB_TOKEN
ARG ENV_FILE_URL
ARG CACHEBUST=1

RUN if [ -z "$GITHUB_TOKEN" ] || [ -z "$ENV_FILE_URL" ]; then \
    echo "Error: GITHUB_TOKEN and ENV_FILE_URL must be provided"; \
    exit 1; \
    fi && \
    curl -H "Authorization: token ${GITHUB_TOKEN}" -H "Accept: application/vnd.github.v3.raw" -H "Cache-Control: no-cache" \
    -o /app/.env -L "${ENV_FILE_URL}?cachebust=${CACHEBUST}"

# 从构建阶段复制二进制文件
COPY --from=build /app/target/release/maimap-server /app/maimap-server
COPY --from=build /app/target/release/maimap-scrape /app/maimap-scrape
RUN chmod +x /app/maimap-server /app/maimap-scrape

RUN echo '#!/bin/sh' > /app/entrypoint.sh && \
    echo 'echo "Starting cron service..."' >> /app/entrypoint.sh && \
    echo 'crond -f &' >> /app/entrypoint.sh && \
    echo 'echo "Cron service started"' >> /app/entrypoint.sh && \
    echo 'echo "Running initial scrape..."' >> /app/entrypoint.sh && \
    echo '/app/maimap-scrape' >> /app/entrypoint.sh && \
    echo 'echo "Starting main application..."' >> /app/entrypoint.sh && \
    echo 'exec /app/maimap-server' >> /app/entrypoint.sh && \
    chmod +x /app/entrypoint.sh

CMD ["/app/entrypoint.sh"]