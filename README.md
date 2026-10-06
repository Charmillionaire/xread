<div align="center">

# XRead

**自托管高性能现代UI风格阅读服务端 · Rust + Vue 3**

支持自定义书源、多格式解析、短剧/听书流式播放、多用户权限、RSS 与 AI 阅读助手

[![Build Docker Image](https://github.com/Charmillionaire/xread/actions/workflows/build.yml/badge.svg)](https://github.com/Charmillionaire/xread/actions/workflows/build.yml)
[![Image](https://img.shields.io/badge/ghcr.io-charmillionaire%2Fxread-2496ED?logo=docker&logoColor=white)](https://github.com/Charmillionaire/xread/pkgs/container/xread)

</div>

---

## 简介

XRead 是一个基于 Rust 构建的自托管高性能现代UI风格阅读服务端，配套 Vue 3 现代 Web 客户端。

沿用 Legado / reader 的书源生态，通过内置 QuickJS 引擎完整兼容各类书源规则，同时提供流式媒体播放、多用户权限体系与苹果风格的液态玻璃界面。

> 本项目由 **逍遥游（[@Charmillionaire](https://github.com/Charmillionaire)）** 开发维护，基于 [reader-rust](https://github.com/givenge/reader-rust) 二次开发；其源头为 [reader](https://github.com/hectorqin/reader)。

## 界面预览

| 桌面端 | 移动端 |
|:---:|:---:|
| ![书架](docs/images/index/bookshelf/shelf.png) | ![书架移动端](docs/images/index/bookshelf/shelf-mobile.png) |

## 功能特性

### 书源解析

- 完整兼容 Legado 书源格式，内置 QuickJS 脚本引擎
- 支持 CSS 选择器、JSONPath、XPath、正则、JavaScript 多种解析方式
- 复合规则解析：`<js>...</js>$.data` 形式的 JS + JSONPath 混合规则
- 全站 Cookie 智能回退与登录态持久化
- 书源批量测试、失效清理、登录取源、导入导出
- 替换规则管理

### 阅读与内容

- 书籍搜索、目录获取、正文加载与章节缓存
- 本地 `.txt` / `.epub` 导入与解析
- 分组管理、批量操作、书架排序
- 阅读进度记忆、书架云同步
- TTS 语音朗读

### 媒体播放

- 短剧视频 / 听书音频流式代理播放
- 完整支持 HTTP Range（206 断点续传与拖动）
- 播放位置按「书籍 + 章节」持久化记忆
- 播放结束自动连播下一集

### AI 阅读助手

- 基于 **Gemini 3.8 Flash** 的书籍理解与 AI 资料生成
- 人物关系图谱、章节摘要、故事脉络梳理
- 支持浏览器直连与服务端代理两种调用方式
- 可自定义文本模型与绘图模型

### 多用户与权限

- 注册用户各自独立的书架、书源与阅读配置
- 公开只读模式下游客可浏览管理员共享内容，但无权写入
- 管理员拥有最高权限：用户增删、密码重置、权限调整
- 服务器备份与恢复（WebDAV）

### 其他

- RSS 订阅与文章阅读
- 服务端版本更新检查
- 苹果风格 Dock 导航栏与液态玻璃 UI，支持亮色 / 暗色主题
- PWA 支持，可安装到主屏幕离线使用

## 快速开始

### Docker 部署（推荐）

```bash
docker pull ghcr.io/charmillionaire/xread:latest

docker run -d \
  --name xread \
  -p 7777:7777 \
  -v $(pwd)/storage:/app/storage \
  -e SECURE=true \
  -e PUBLIC_READ=true \
  ghcr.io/charmillionaire/xread:latest
```

或使用 `docker-compose.yml`：

```yaml
services:
  xread:
    image: ghcr.io/charmillionaire/xread:latest
    container_name: xread
    ports:
      - "7777:7777"
    volumes:
      - ./storage:/app/storage
    environment:
      SERVER_PORT: "7777"
      LOG_LEVEL: "info"
      SECURE: "true"
      PUBLIC_READ: "true"
    restart: unless-stopped
```

```bash
docker compose up -d
```

启动后访问 `http://<你的地址>:7777`。

### 从源码构建

```bash
# 克隆项目
git clone https://github.com/Charmillionaire/xread.git
cd xread

# 构建前端
cd frontend
npm install
npm run build
cd ..

# 运行后端
cargo run
```

前端开发模式（热更新）：

```bash
cd frontend
npm run dev
```

## 配置项

配置通过环境变量注入（可参考 `.env.example`）：

| 变量 | 默认值 | 说明 |
|---|---|---|
| `SERVER_HOST` | `0.0.0.0` | 监听地址 |
| `SERVER_PORT` | `7777` | 监听端口 |
| `DATABASE_URL` | `sqlite:storage/reader.db?mode=rwc` | SQLite 数据库连接 |
| `STORAGE_DIR` | `storage` | 数据存储目录 |
| `ASSETS_DIR` | `storage/assets` | 静态资源目录 |
| `WEB_ROOT` | `frontend/dist` | 前端产物目录 |
| `LOG_LEVEL` | `info` | 日志级别 |
| `REQUEST_TIMEOUT_SECS` | `15` | 请求超时（秒） |
| `SECURE` | `false` | 是否开启多用户模式 |
| `SECURE_KEY` | 空 | 管理密钥（可选） |
| `PUBLIC_READ` | `false` | 公开只读模式：游客可读管理员的书源 / 书架 |
| `INVITE_CODE` | 空 | 注册邀请码 |
| `USER_LIMIT` | `50` | 用户数上限 |
| `USER_BOOK_LIMIT` | `2000` | 单用户书籍上限 |
| `USER_LOCAL_BOOK_LIMIT` | `0` | 单用户本地书上限（0 表示不限） |

## 技术栈

- **后端**：Rust · axum · tokio · reqwest · sqlx (SQLite) · rquickjs (QuickJS)
- **前端**：Vue 3 · Vite · TypeScript · Pinia · vue-router
- **AI**：Gemini 3.8 Flash（AI 阅读助手）
- **构建**：GitHub Actions → GitHub Container Registry (GHCR)

## 测试

```bash
# 后端测试
cargo test

# 前端单测
cd frontend && npm test

# 端到端测试
npm run test:e2e
```

## 自动构建

推送代码到 `master` 分支后，GitHub Actions 会自动构建并发布镜像：

```
ghcr.io/charmillionaire/xread:latest
```

## 项目结构

```
xread/
├── src/              # Rust 后端
│   ├── api/          # 路由与请求处理
│   ├── service/      # 业务服务（书源、书籍、用户、RSS…）
│   ├── model/        # 数据模型
│   └── app/          # 配置与启动
├── frontend/         # Vue 3 前端
│   └── src/
│       ├── views/        # 页面
│       ├── components/   # 组件
│       ├── stores/       # Pinia 状态
│       └── utils/        # 工具函数
├── docs/             # 文档站点
└── tests/            # 集成测试
```

## 免责声明

本项目仅提供书源管理、内容解析、阅读与缓存等技术能力，**不内置、不存储、不分发、不提供任何受版权保护的书籍内容**。

用户应确保自行添加的书源、上传的本地文件以及通过本服务访问的内容均已获得合法授权，并自行承担由此产生的版权与合规责任。

如任何权利人认为本项目相关内容或使用方式侵犯了其合法权益，请通过项目 Issues 联系维护者，我们将在核实后及时处理。

## 贡献者

| 角色 | 贡献者 |
|---|---|
| 作者 / 维护者 | **逍遥游** ([@Charmillionaire](https://github.com/Charmillionaire)) |
| AI 协作开发 | **Gemini** · **Claude** |
| 上游项目 | [reader-rust](https://github.com/givenge/reader-rust) · [reader](https://github.com/hectorqin/reader) |

欢迎提交 Issue 与 Pull Request，一起把它做得更好。

## Star 趋势

如果这个项目对你有帮助，欢迎点个 ⭐ Star 支持一下！
