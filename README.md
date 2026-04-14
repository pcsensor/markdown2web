# markdown2web

一个用 Rust 编写的 Markdown -> 网站全栈项目：

- 公共站点：SSR 渲染的笔记网站
- 后台管理：登录、上传 Markdown、上传资源、编辑与重建
- 内容真源：`content/notes` 与 `content/assets`
- 自动更新：启动服务后可监听内容目录并自动重建
- 链接能力：支持相对 Markdown 链接与 `[[Wiki Link]]`
- 资源能力：支持图片/附件复制到公开 `/assets/*` 路径，并在安装 `ffmpeg` 时自动生成轻量图片/视频派生资源
- 数学公式：支持 LaTeX 公式渲染
- 代码块：支持主流 fenced code block 语法高亮
- 交互体验：支持滚动进度、卡片指针光效、复制代码按钮等微交互

## 技术栈

- Rust
- Axum
- Askama
- SQLite (`rusqlite`)
- Comrak
- Notify
- FFmpeg（可选，用于图片多尺寸生成、视频压缩和 poster 生成；缺失时会降级为原资源发布）

## 快速启动

```bash
cargo run
```

默认地址：

- 公开站点：`http://127.0.0.1:3000`
- 管理后台：`http://127.0.0.1:3000/admin`

默认管理员账号：

- 用户名：`admin`
- 密码：`admin123456`

## 目录约定

```text
content/
  notes/      # Markdown 笔记真源
  assets/     # 共享资源

generated/site/assets/  # 构建后的公开资源

data/app.db             # SQLite 数据库
```

## 大媒体优化

项目会优先使用 `ffmpeg` 在构建阶段处理大媒体资源，降低低带宽服务器的传输压力：

- Markdown 图片会生成多尺寸派生图，并在 HTML 中输出 `<picture>`、`srcset`、`loading="lazy"` 和 `decoding="async"`。
- `@[描述](视频路径)` 视频会转为 `720p` 压缩 MP4，尽量生成 poster，并在页面中使用 `preload="none"` + 点击后加载的懒加载模式。
- 生成物存在且比源文件新时会直接复用，不会每次 rebuild 都重复转码。
- 如果 `ffmpeg` 不存在，或某个媒体文件处理失败，构建不会中断，会记录 warning 并回退到原资源懒加载/发布。

生产部署建议把 `generated/site/assets` 放在对象存储或 CDN 后面，由主服务只承担 HTML 和 API 流量。

## 环境变量

- `M2W_HOST`
- `M2W_PORT`
- `M2W_BASE_URL`
- `M2W_SITE_NAME`
- `M2W_CONTENT_DIR`
- `M2W_GENERATED_DIR`
- `M2W_DATA_DIR`
- `M2W_ADMIN_USERNAME`
- `M2W_ADMIN_PASSWORD`
- `M2W_WATCH_ENABLED`
- `M2W_SECURE_COOKIES`（默认 `false`；HTTPS 生产部署建议设为 `true`）
- `M2W_SESSION_TTL_HOURS`（默认 `168`）
- `M2W_UPLOAD_LIMIT_MB`（默认 `128`，用于 Markdown/资源上传；上传视频时按实际文件大小调高）

如果数据库中已经存在管理员账号，启动时只有在显式设置 `M2W_ADMIN_PASSWORD`（例如写入 `.env`）的情况下，服务才会把该账号密码同步为配置值，并清理旧管理员会话；未显式设置时不会用默认密码覆盖后台里手动修改过的密码。

## 安全说明

- 后台与账号状态变更请求会校验 CSRF token；自定义前端请求需要携带 `X-CSRF-Token`。
- 管理员提交的 slug 和上传文件名只允许安全 basename，禁止 `/`、`\`、`..`。
- 上传资源不支持 SVG；`/static/favicon.svg` 属于内置静态资源，不受后台上传限制影响。
- 登录、注册、批注和弹幕写入有 SQLite 限流；单进程部署下无需额外服务。

## 验证

```bash
cargo check
cargo test
```
