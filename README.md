<div align="center">

# XRead

**高性能自托管现代阅读服务端与流式媒体引擎 · Rust + Vue 3**

轻量高效 · 生态兼容 · 流媒体代理 · 私有化多用户 · AI 助读

[![Build Docker Image](https://github.com/Charmillionaire/xread/actions/workflows/build.yml/badge.svg)](https://github.com/Charmillionaire/xread/actions/workflows/build.yml)
[![Image](https://img.shields.io/badge/ghcr.io-charmillionaire%2Fxread-2496ED?logo=docker&logoColor=white)](https://github.com/Charmillionaire/xread/pkgs/container/xread)

</div>

---

## 项目概述

XRead 是一个采用 Rust（后端）与 Vue 3（前端）构建的高性能、自托管数字阅读服务端与流式媒体代理引擎。

系统专注于提供极低的系统资源占用、毫秒级响应速度以及对通用开源阅读规则生态的深度兼容。通过内置轻量级脚本运行时，系统能够灵活解析结构化文本与流媒体，并配套现代化的响应式液态玻璃 UI 界面，为自建服务用户提供私密、安全且完全自主受控的阅读与音视频交互环境。

> 本项目由 **逍遥游（[@Charmillionaire](https://github.com/Charmillionaire)）** 主导维护，基于 [reader-rust](https://github.com/givenge/reader-rust) 深度二次开发重构；其架构溯源自开源项目 [reader](https://github.com/hectorqin/reader)。

---

## 核心设计与特性

### 1. 规则解析与生态兼容
- **通用生态支持**：深度兼容主流开源移动阅读器（如 Legado）的规则规范，降低迁移与配置成本。
- **内置轻量脚本引擎**：集成 QuickJS 运行时，安全沙箱执行数据清洗与逻辑重整。
- **全格式解析能力**：支持 CSS Selector、XPath、JSONPath、正则表达式以及 JS + JSONPath 复合规则链路。
- **会话状态管理**：内置全站网络会话状态自动处理与状态持久化机制。

### 2. 沉浸式阅读与排版引擎
- **居中悬浮控制面板**：无边框毛玻璃控制栏，单色自适应图标体系，支持中心唤醒，杜绝边缘误触。
- **纯粹翻页控制**：纯显式按需导航，移除隐式越界自动翻页，阅读翻页反馈精准平稳。
- **多端设备自适应**：无缝支持桌面宽屏与移动端竖屏，提供一致的操作流。
- **本地文档解析**：支持标准 `.txt` 与 `.epub` 格式文件的导入、元数据提取与分章节存储。
- **智能离线缓存**：支持后台预加载与章节批量下载，保障弱网环境下的阅读连续性。

### 3. 多媒体流式代理引擎
- **音视频切片代理**：针对有声书（音频）与短剧（视频）提供高性能流媒体中继支持。
- **HTTP 206 断点续传**：完整实现 Range 请求支持，拖动进度条毫秒级响应。
- **媒体选集与下载**：内嵌轻量级播放器选集弹窗与直链转存下载能力。
- **精细进度记忆**：播放状态基于用户与章节粒度精确持久化，支持自动连播。

### 4. 现代交互与主题系统
- **液态玻璃美学**：卡片与功能模块全局采用现代 Glassmorphism 毛玻璃拟态设计。
- **无缝顶栏沉浸**：无边界一体化顶部导航，搭配黑白自适应动态品牌标识。
- **全站主题色联动**：支持在 UI 设置中自由定制全站主色调与背景色，底栏 Dock 与全局组件自动跟随。
- **相关度排序搜索**：内置搜索结果相关度加权算法，多结果匹配更精准。

### 5. 智能化阅读辅助
- **AI 上下文梳理**：集成大语言模型能力，提供长文速读、章节摘要与人物脉络分析。
- **双模调用链路**：支持前端直连与服务端代理两种调用方式，兼顾灵活性与密钥安全。

### 6. 私有化部署与权限管理
- **细粒度多用户体系**：支持多账号独立书架、个人偏好与阅读进度隔离。
- **公开/私有模式切换**：支持公开只读（游客访问受限资源）与全私有鉴权模式。
- **数据自主掌控**：支持 SQLite 本地持久化与 WebDAV 云端自动化热备份。

---

## 快速部署

### Docker 容器化运行（推荐）

```bash
docker pull ghcr.io/charmillionaire/xread:latest

docker run -d \
  --name xread \
  -p 7777:7777 \
  -v $(pwd)/storage:/app/storage \
  -e SECURE=true \
  -e PUBLIC_READ=false \
  ghcr.io/charmillionaire/xread:latest
```

### Docker Compose 编排

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
      PUBLIC_READ: "false"
    restart: unless-stopped
```

启动后访问 `http://<服务器地址>:7777` 即可初始化管理账户。

---

## 环境变量配置说明

| 变量名 | 默认值 | 作用描述 |
|---|---|---|
| `SERVER_HOST` | `0.0.0.0` | 服务绑定监听网卡地址 |
| `SERVER_PORT` | `7777` | 服务对外监听端口 |
| `DATABASE_URL` | `sqlite:storage/reader.db?mode=rwc` | 数据库连接字符串 |
| `STORAGE_DIR` | `storage` | 数据与媒体缓存根目录 |
| `ASSETS_DIR` | `storage/assets` | 静态资产持久化目录 |
| `LOG_LEVEL` | `info` | 运行日志输出级别 |
| `REQUEST_TIMEOUT_SECS` | `15` | 出站网络请求超时阈值（秒） |
| `SECURE` | `false` | 是否开启多用户安全鉴权模式 |
| `PUBLIC_READ` | `false` | 是否允许未认证用户只读公开内容 |
| `INVITE_CODE` | 空 | 新用户注册邀请码（留空则开放自由注册） |
| `USER_LIMIT` | `50` | 允许注册的最大用户数上限 |

---

## 技术架构

- **后端核心**：Rust · Axum · Tokio · Reqwest · SQLx (SQLite) · rquickjs
- **前端架构**：Vue 3 · Vite · TypeScript · Pinia · Vue Router
- **部署标准**：OCI 兼容容器镜像 · GitHub Actions 持续集成与分发

---

## 免责声明与合规说明

1. **纯技术工具定位**：XRead 仅作为纯技术用途的开源阅读管理服务端、文本解析器与媒体流式中继软件。本项目本身**不内置、不提供、不聚合、不存储、亦不分发任何受法律保护的书籍、音频、视频或网络媒体内容**。
2. **规则与网络中立**：软件中包含的解析引擎属于技术中立工具。用户在使用本软件时所配置、导入的书源规则以及获取的网络资源，均由用户自行决定并完全承担法律责任。
3. **知识产权尊重**：用户应当严格遵守所在国家与地区的法律法规，确保自身对通过本软件处理的所有内容享有合法的访问、阅读或存储授权。开发者不对任何用户的违法或侵权行为承担任何连带责任。
4. **侵权联络渠道**：若任何权利人认为本项目源代码本身存在侵犯其知识产权的情形，请通过 GitHub Issues 提交正式权利通知，维护团队将在收到通知并核实后依法处置。

---

## 开源协作与致谢

- **作者与主导维护**：**逍遥游** ([@Charmillionaire](https://github.com/Charmillionaire))
- **基础项目溯源**：感谢 [reader-rust](https://github.com/givenge/reader-rust) 与 [reader](https://github.com/hectorqin/reader) 开源社区的技术积累。
