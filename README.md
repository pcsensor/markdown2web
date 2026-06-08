# markdown2web

一个用 Rust 编写的全栈应用，将 Markdown 笔记发布为 SSR 渲染的公共站点，附带管理后台。

## 特性

### 内容发布

- **SSR 渲染** — Axum + Askama 模板，服务端直出 HTML，无 JavaScript 框架依赖
- **Markdown 扩展** — 数学公式（LaTeX）、语法高亮（syntect）、表格、脚注、任务列表
- **Wiki 链接** — 支持 `[[Wiki Link]]`、相对 `.md` 链接，解析失败自动标记为 broken link
- **反向链接** — 构建全站笔记引用关系图，每篇笔记自动展示被哪些笔记引用
- **标签 / 分类** — 按标签、分类聚合笔记，支持独立标签页与分类页
- **全文搜索** — 基于内存索引的实时搜索
- **微交互** — 滚动进度条、卡片指针光效、一键复制代码块

### 媒体优化

- **图片多尺寸** — FFmpeg 自动生成多尺寸派生图，HTML 输出 `<picture>` + `srcset` + 懒加载
- **视频转码** — `@[描述](视频路径)` 语法，转为 720p 压缩 MP4 并生成 poster，页面采用点击加载模式
- **增量处理** — 生成物存在且比源文件新时直接复用，避免重复转码
- **降级兜底** — FFmpeg 缺失或转码失败时，自动回退到原始资源，记录 warning

### 管理后台（`/admin`）

- **Markdown 上传** — 批量上传 `.md` 文件，自动触发增量 rebuild
- **资源上传** — 上传图片、视频等媒体资源到 `content/assets/`
- **在线编辑** — 在后台直接新建、编辑笔记
- **手动重建** — 一键触发全站 rebuild，实时查看构建进度
- **用户管理** — 查看、创建、修改、删除公共用户
- **密码修改** — 在线修改管理员登录密码

### 公共账号系统（`/account`）

- **注册 / 登录** — 公共用户账号体系，Argon2 密码哈希，Cookie 会话
- **划线批注** — 登录后可对笔记段落进行高亮批注，支持颜色标记与私密/公开可见性
- **视频弹幕** — 登录后可在视频播放时发送弹幕

### 自动重建

- **文件监听** — 基于 Notify 的实时监听，800 ms 防抖后自动触发 rebuild
- **全量内存** — rebuild 完成后，所有笔记数据全量存入 `Arc<RwLock<SiteData>>`，请求直接读内存，零查库延迟
- **哈希缓存** — `BuildCache` 记录文件内容哈希，用于判断变更范围

## 技术栈

| 组件 | 版本 | 用途 |
|------|------|------|
| Rust | edition 2024 | 语言 |
| Axum | 0.8 | Web 框架 |
| Askama | 0.14 | 服务端模板引擎 |
| rusqlite | 0.37 (bundled) | SQLite 数据库 |
| Comrak | 0.29 | Markdown 渲染 |
| Notify | 7 | 文件系统监听 |
| Argon2 | 0.5 | 密码哈希 |
| FFmpeg | 系统安装 | 媒体处理（可选） |

## 快速开始

### 环境要求

- Rust 工具链（stable，edition 2024）
- FFmpeg（可选；缺失时媒体处理降级为原资源发布）

### 启动开发服务器

```bash
cargo run
```

首次启动会自动创建必要目录并写入示例内容，无需额外初始化步骤。

默认访问地址：

| 路径 | 说明 |
|------|------|
| `http://127.0.0.1:3000` | 公开站点首页 |
| `http://127.0.0.1:3000/admin` | 管理后台 |
| `http://127.0.0.1:3000/account` | 公共用户登录/注册 |
| `http://127.0.0.1:3000/health` | 健康检查接口 |

默认管理员账号：

- 用户名：`admin`
- 密码：`admin123456`

> **⚠ 安全提示：** 生产环境务必通过环境变量修改管理员密码（见下方配置说明）。

### 常用命令

```bash
cargo run          # 启动开发服务器
cargo check        # 快速类型检查（不产出二进制）
cargo test         # 运行全部测试
cargo fmt --all    # 格式化代码
cargo clippy --all-targets --all-features  # Lint 检查
```

## 目录结构

```text
markdown2web/
├── content/
│   ├── notes/          # Markdown 笔记源文件（内容真源）
│   └── assets/         # 共享媒体资源
├── generated/
│   └── site/
│       └── assets/     # 构建产出的公开资源（哈希前缀命名）
├── data/
│   └── app.db          # SQLite 数据库
├── src/                # Rust 源代码
├── templates/          # Askama HTML 模板
└── static/             # 静态文件（JS、favicon）
```

> `content/` 是唯一的内容真源。`generated/` 和 `data/` 均为运行时产出，可安全删除后重建。

## 配置

所有配置通过 `M2W_` 前缀的环境变量设置。默认值覆盖本地开发场景，直接 `cargo run` 即可使用，无需任何配置。

生产环境建议将配置写入 `.env` 文件（参考 `.env.example`）。

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `M2W_HOST` | `127.0.0.1` | 监听地址（生产环境改为 `0.0.0.0`） |
| `M2W_PORT` | `3000` | 监听端口 |
| `M2W_BASE_URL` | `http://127.0.0.1:3000` | 站点基础 URL（影响链接生成） |
| `M2W_SITE_NAME` | `markdown2web` | 站点名称（显示在页面标题） |
| `M2W_CONTENT_DIR` | `content` | 内容目录路径 |
| `M2W_GENERATED_DIR` | `generated/site` | 构建输出目录路径 |
| `M2W_DATA_DIR` | `data` | 数据目录路径 |
| `M2W_ADMIN_USERNAME` | `admin` | 管理员用户名 |
| `M2W_ADMIN_PASSWORD` | `admin123456` | **管理员密码，生产必改** |
| `M2W_WATCH_ENABLED` | `true` | 是否启用文件监听自动重建（服务器端建议关闭） |
| `M2W_UPLOAD_LIMIT_MB` | `128` | 上传文件大小限制（MB）；上传视频时建议调大，如 `512` |
| `M2W_TURNSTILE_ENABLED` | `false` | 是否启用 Cloudflare Turnstile 人机验证 |
| `M2W_TURNSTILE_SITE_KEY` | — | Turnstile site key |
| `M2W_TURNSTILE_SECRET_KEY` | — | Turnstile secret key |

## Markdown Front Matter

每篇笔记的 YAML front matter 支持以下字段：

```yaml
---
title: 文章标题          # 必填
slug: url-friendly-slug  # 必填，用于生成 URL
summary: 摘要一句话       # 可选，显示在列表页
tags: [tag1, tag2]       # 可选，标签列表
category: 分类名          # 可选，分类
status: published        # published（默认）或 draft（不公开）
aliases: [别名1, 别名2]   # 可选，Wiki 链接别名
updated: "2024-01-01"    # 可选，手动指定更新时间；未填则读取文件 mtime
---
```

## 媒体语法

### 图片

标准 Markdown 语法，支持相对路径引用 `content/assets/` 下的资源：

```markdown
![图片描述](image.jpg)
```

构建时自动生成多尺寸版本，输出带 `srcset` 的 `<picture>` 标签。

### 视频

使用扩展语法：

```markdown
@[视频描述](video.mp4)
```

构建时自动转码为 720p MP4，生成 poster，页面采用点击展开的懒加载模式。

### Wiki 链接

```markdown
[[另一篇笔记的标题]]
[[slug-of-note]]
[[别名]]
```

支持按标题、slug、文件名、别名多路查找。解析失败的链接会被标记为 broken link，并在管理后台展示。

## 生产部署建议

1. **修改管理员密码**：必须通过 `M2W_ADMIN_PASSWORD` 环境变量设置强密码
2. **关闭文件监听**：服务器端内容通过管理后台管理，建议设置 `M2W_WATCH_ENABLED=false`
3. **CDN 加速静态资源**：将 `generated/site/assets/` 托管到对象存储或 CDN，主服务仅承担 HTML 和 API 流量
4. **启用 Turnstile**：注册/登录表单建议开启 Cloudflare Turnstile 防刷，配置 `M2W_TURNSTILE_ENABLED=true`
5. **反向代理**：建议在 Nginx 前面套一层，处理 HTTPS 和静态资源缓存头

## 验证

```bash
cargo check   # 类型检查（快速迭代时使用）
cargo test    # 运行全部集成与单元测试
```

## 许可证

MIT
