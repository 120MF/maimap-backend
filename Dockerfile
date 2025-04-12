# 构建阶段 - 使用Alpine作为构建基础镜像
FROM rust:1.86-alpine3.21 AS build
WORKDIR /app

# 安装构建依赖
RUN apk add --no-cache musl-dev pkgconfig openssl-dev perl make

# 分层缓存构建依赖
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src/bin && \
    echo "fn main() {}" > src/bin/server.rs && \
    echo "fn main() {}" > src/bin/scrape.rs && \
    echo "pub fn dummy() {}" > src/lib.rs && \
    cargo build --release --features server,scrape

# 复制实际源代码并重新构建
COPY src ./src/
RUN touch src/bin/server.rs src/bin/scrape.rs src/lib.rs && \
    cargo build --release --features server,scrape

# 运行阶段
FROM alpine:3.21

# 安装运行依赖
RUN apk add --no-cache ca-certificates curl chromium tzdata dcron mongodb-tools && \
    mkdir -p /etc/cron.d /app

# 设置时区
ENV TZ=Asia/Shanghai

# 创建cron任务
RUN echo "0 */5 * * * cd /app && /app/scraper >> /app/scraper.log 2>&1" > /etc/cron.d/scraper-cron && \
    chmod 0644 /etc/cron.d/scraper-cron && \
    crontab /etc/cron.d/scraper-cron

# 指定chromium路径
ENV CHROME_PATH=/usr/bin/chromium-browser
ENV CHROME_DEVEL_SANDBOX=/usr/bin/chrome-devel-sandbox

WORKDIR /app

ARG GITHUB_TOKEN
ARG ENV_FILE_URL

RUN if [ -z "$GITHUB_TOKEN" ] || [ -z "$ENV_FILE_URL" ]; then \
    echo "Error: GITHUB_TOKEN and ENV_FILE_URL must be provided"; \
    exit 1; \
    fi && \
    curl -H "Authorization: token ${GITHUB_TOKEN}" -H "Accept: application/vnd.github.v3.raw" -H "Cache-Control: no-cache" \
    -o /app/.env -L "${ENV_FILE_URL}"

# 从构建阶段复制二进制文件
COPY --from=build /app/target/release/server /app/app
COPY --from=build /app/target/release/scrape /app/scraper
RUN chmod +x /app/app /app/scraper

RUN echo '#!/bin/sh' > /app/entrypoint.sh && \
    echo 'echo "Starting cron service..."' >> /app/entrypoint.sh && \
    echo 'crond -f &' >> /app/entrypoint.sh && \
    echo 'echo "Cron service started"' >> /app/entrypoint.sh && \
    echo 'echo "Running initial scrape..."' >> /app/entrypoint.sh && \
    echo '/app/scraper' >> /app/entrypoint.sh && \
    echo 'echo "Starting main application..."' >> /app/entrypoint.sh && \
    echo 'exec /app/app' >> /app/entrypoint.sh && \
    chmod +x /app/entrypoint.sh

CMD ["/app/entrypoint.sh"]