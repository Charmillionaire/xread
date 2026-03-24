# reader-rust

基于reader重构 https://github.com/hectorqin/reader

Rust 版 `reader` 服务端，基于 axum + tokio + reqwest + sqlx(SQLite) + rquickjs。

## 运行环境

- Rust 1.75+（建议使用最新稳定版）
- SQLite

## 打包与运行

### 运行（开发模式）

```bash
cargo run
```

### 构建（Release）

```bash
cargo build --release
```

### musl 静态编译

生成的二进制文件不依赖 glibc，可在任何 x86_64 Linux 上运行。

#### Linux 环境

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --target x86_64-unknown-linux-musl --release
```

#### macOS 交叉编译

macOS 需要安装 musl-cross 工具链：

```bash
# 1. 安装 musl-cross
brew install FiloSottile/musl-cross/musl-cross

# 2. 添加 Rust target
rustup target add x86_64-unknown-linux-musl

# 3. 配置链接器（项目已包含 .cargo/config.toml）
# 如果没有，创建 .cargo/config.toml 内容如下：
# [target.x86_64-unknown-linux-musl]
# linker = "x86_64-linux-musl-gcc"

# 4. 编译
cargo build --target x86_64-unknown-linux-musl --release
```

> 注意：确保使用 rustup 安装的 Rust，而非 Homebrew 的 Rust。
> 当前Homebrew 的 Rust 不支持交叉编译 target。
> 如有冲突，临时用 rustup 的 cargo ~/.cargo/bin/cargo build --release --target x86_64-unknown-linux-musl 
> 或者卸载Homebrew 的 Rust `brew uninstall rust` 后使用 rustup 版本。

### musl 静态编译后的二进制文件

```bash
target/x86_64-unknown-linux-musl/release/reader-rust
```

### 运行（Release）

```bash
./target/release/reader-rust
```

## 配置

支持环境变量覆盖。

默认值：

### Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

### Database (SQLite)
DATABASE_URL=sqlite:storage/reader.db?mode=rwc

### Storage paths
STORAGE_DIR=storage
ASSETS_DIR=storage/assets

### Web frontend path (adjust if needed)
WEB_ROOT=web/dist

### Logging
LOG_LEVEL=info

### Request timeout in seconds
REQUEST_TIMEOUT_SECS=15

### Security settings
SECURE=false
SECURE_KEY=

### User registration
INVITE_CODE=
USER_LIMIT=50
USER_BOOK_LIMIT=2000