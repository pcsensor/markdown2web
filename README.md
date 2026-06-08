# markdown2web

一个用 Rust 编写的全栈应用，将 Markdown 笔记转换为 SSR 渲染的网站，附带管理后台。

## 特性

- **公共站点** — SSR 渲染的笔记网站，支持标签、分类和全文搜索
- **管理后台** — 登录、上传 Markdown、上传资源、编辑笔记与站点重建
- **内容真源** — `content/notes` 与 `content/assets` 目录即站点内容
- **自动重建** — 文件监听 + 800 ms 防抖，内容变更自动触发 rebuild
- **链接能力** — 支持相对 Markdown 链接、`[[Wiki Link]]` 与资源引用
- **媒体优化** — 安装 FFmpeg 后自动生成多尺寸图片、压缩视频、生成 poster
- **数学公式** — 支持 LaTeX 公式渲染
- **代码高亮** — 支持主流 fenced code block 语法高亮
- **微交互** — 滚动进度条、卡片指针光效、复制代码按钮等

## 技术栈

| 组件 | 用途 |
|------|------|
| Rust (edition 2024) | 语言 |
| Axum | Web 框架 |
| Askama | 模板引擎 |
| SQLite / rusqlite | 数据库 |
| Comrak | Markdown 渲染 |
| Notify | 文件系统监听 |
| FFmpeg | 媒体处理（可选） |

## 快速开始

### 环境要求

- Rust 工具链（stable）
- FFmpeg（可选，缺失时媒体处理降级为原资源发布）

### 启动

```bash
cargo run
```

默认地址：

- 公开站点：`http://127.0.0.1:3000`
- 管理后台：`http://127.0.0.1:3000/admin`

默认管理员账号：

- 用户名：`admin`
- 密码：`admin123456`

> **注意：** 生产环境请务必通过环境变量修改管理员密码。

## 目录约定

```text
content/
  notes/                  # Markdown 笔记源文件
  assets/                 # 共享资源文件

generated/site/assets/    # 构建产出的公开资源（哈希前缀命名）

data/app.db               # SQLite 数据库
```

## 配置

所有配置通过 `M2W_` 前缀的环境变量设置，默认值可直接 `cargo run` 使用。

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `M2W_HOST` | `127.0.0.1` | 监听地址 |
| `M2W_PORT` | `3000` | 监听端口 |
| `M2W_BASE_URL` | `http://127.0.0.1:3000` | 站点基础 URL |
| `M2W_SITE_NAME` | `markdown2web` | 站点名称 |
| `M2W_CONTENT_DIR` | `content` | 内容目录路径 |
| `M2W_GENERATED_DIR` | `generated/site` | 构建输出目录路径 |
| `M2W_DATA_DIR` | `data` | 数据目录路径 |
| `M2W_ADMIN_USERNAME` | `admin` | 管理员用户名 |
| `M2W_ADMIN_PASSWORD` | `admin123456` | 管理员密码 |
| `M2W_WATCH_ENABLED` | `true` | 是否启用文件监听自动重建 |
| `M2W_UPLOAD_LIMIT_MB` | `128` | 上传文件大小限制（MB） |
| `M2W_TURNSTILE_ENABLED` | `false` | 是否启用 Cloudflare Turnstile 验证 |
| `M2W_TURNSTILE_SITE_KEY` | — | Turnstile site key |
| `M2W_TURNSTILE_SECRET_KEY` | — | Turnstile secret key |

## Front Matter 格式

```yaml
---
title: 标题
slug: url-slug
summary: 摘要
tags: [tag1, tag2]
status: published  # 或 draft
aliases: [别名1]
---
```

## 大媒体优化

项目优先使用 FFmpeg 在构建阶段处理大媒体资源，降低带宽压力：

- 图片生成多尺寸派生图，HTML 输出 `<picture>`、`srcset`、`loading="lazy"` 和 `decoding="async"`
- 视频（`@[描述](视频路径)`）转为 720p 压缩 MP4，尽量生成 poster，页面使用 `preload="none"` + 点击加载的懒加载模式
- 生成物存在且比源文件新时直接复用，避免重复转码
- FFmpeg 缺失或处理失败时记录 warning 并回退到原资源

> 生产部署建议将 `generated/site/assets` 放在对象存储或 CDN 后面，主服务仅承担 HTML 和 API 流量。

## 验证

```bash
cargo check   # 类型检查
cargo test    # 运行全部测试
```

## 许可证

MIT
