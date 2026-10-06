# Docker 部署

## 使用 Dockerfile

### 构建镜像

```bash
docker build -t reader-rust .
```

### 运行容器

```bash
docker run -d \
  --name reader \
  -p 7777:7777 \
  -v $(pwd)/storage:/app/storage \
  reader-rust
```

## 使用 Docker Compose

创建 `docker-compose.yml`:

```yaml
version: '3.8'

services:
  reader:
    image: reader-rust
    container_name: reader
    ports:
      - "7777:7777"
    volumes:
      - ./storage:/app/storage
      - ./assets:/app/assets
    environment:
      - SERVER_PORT=7777
      - LOG_LEVEL=info
    restart: unless-stopped
```

启动服务：

```bash
docker-compose up -d
```

## 环境变量配置

```yaml
environment:
  - SERVER_HOST=0.0.0.0
  - SERVER_PORT=7777
  - DATABASE_URL=sqlite:storage/reader.db?mode=rwc
  - LOG_LEVEL=info
  - REQUEST_TIMEOUT_SECS=15
```

## 数据持久化

确保挂载存储目录：

```bash
-v $(pwd)/storage:/app/storage
```

这包括：
- SQLite 数据库
- 章节内容缓存
- 配置文件

## 多平台构建

```bash
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t yourusername/reader-rust:latest \
  --push .
```
