# reader-rust

Rust 版 `reader` 服务端（核心阅读 API），基于 axum + tokio + reqwest + sqlx(SQLite) + rquickjs。

## 已实现功能
- 服务端启动、配置加载、日志
- 健康检查接口 `GET /health`
- 书源管理
- 书源保存 `POST /reader3/saveBookSource`
- 书源批量保存 `POST /reader3/saveBookSources`
- 书源查询 `GET|POST /reader3/getBookSource`
- 书源列表 `GET|POST /reader3/getBookSources`
- 书源删除 `POST /reader3/deleteBookSource`
- 书源批量删除 `POST /reader3/deleteBookSources`
- 删除全部书源 `POST /reader3/deleteAllBookSources`
- 搜索
- 单书源搜索 `GET|POST /reader3/searchBook`
- 多书源搜索 `GET|POST /reader3/searchBookMulti`
- 书籍详情 `GET|POST /reader3/getBookInfo`
- 目录 `GET|POST /reader3/getChapterList`
- 正文 `GET|POST /reader3/getBookContent`
- 删除章节缓存 `POST /reader3/deleteBookCache`
- 文件缓存（章节正文）
- 静态资源托管
- `web/` 前端静态资源目录
- `storage/assets` 资源目录

## 尚未实现功能
- RSS 相关 API
- 书签、替换规则、WebDAV、用户相关 API
- SSE（搜索/缓存流式接口）
- 本地书籍（TXT/EPUB/UMD）
- 更完整的规则语法兼容
- XPath 规则
- 复杂 JS 规则上下文（目前仅 `input` 和 `base_url`）
- 多步规则拼接、变量注入、缓存脚本钩子
- 完整的 Java 版缓存与并发控制策略

## 运行环境
- Rust 1.75+（建议使用最新稳定版）
- SQLite

## 打包与运行

### 运行（开发模式）
```bash
cd reder/reader-rust
cargo run
```

### 构建（Release）
```bash
cd reder/reader-rust
cargo build --release
```

### 运行（Release）
```bash
reader-rust/target/release/reader-rust
```

## 配置
支持环境变量覆盖，环境变量分隔符为双下划线。

默认值：
- `SERVER_HOST=0.0.0.0`
- `SERVER_PORT=8080`
- `DATABASE_URL=sqlite:storage/reader.db?mode=rwc`
- `STORAGE_DIR=storage`
- `WEB_ROOT=../reader/web`
- `ASSETS_DIR=storage/assets`
- `LOG_LEVEL=info`
- `REQUEST_TIMEOUT_SECS=15`
- `SECURE=false`
- `SECURE_KEY=`
- `INVITE_CODE=`
- `USER_LIMIT=50`
- `USER_BOOK_LIMIT=2000`

示例：
```bash
SERVER_PORT=8090 DATABASE_URL=sqlite://storage/dev.db cargo run
```

## 接口请求示例

### 保存书源
```bash
curl -X POST http://127.0.0.1:8080/reader3/saveBookSource \
  -H 'Content-Type: application/json' \
  -d @/path/to/book_source.json
```

### 搜索
```bash
curl -X GET "http://127.0.0.1:8080/reader3/searchBook?key=斗罗大陆&page=1&bookSourceUrl=https://example.com"
```

### 获取详情
```bash
curl -X GET "http://127.0.0.1:8080/reader3/getBookInfo?url=https://example.com/book/1&bookSourceUrl=https://example.com"
```

### 获取目录
```bash
curl -X GET "http://127.0.0.1:8080/reader3/getChapterList?tocUrl=https://example.com/book/1/toc&bookSourceUrl=https://example.com"
```

### 获取正文
```bash
curl -X GET "http://127.0.0.1:8080/reader3/getBookContent?chapterUrl=https://example.com/book/1/1&bookSourceUrl=https://example.com"
```

## 目录结构
```text
reader-rust/
  Cargo.toml
  README.md
  src/
    main.rs
    lib.rs
    app/
    api/
    service/
    parser/
    crawler/
    storage/
    model/
    error/
    util/
```

## 说明
- 书源规则兼容性为“最小可用”版本，仅支持 CSS 选择器和 JSONPath；JS 规则仅支持 `js:` 前缀执行。
- 章节正文使用文件缓存，缓存 key 为章节 URL 的 MD5。
- 多书源搜索默认并发执行。
