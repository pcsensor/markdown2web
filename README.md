# markdown2web

一个用 Rust 编写的 Markdown -> 网站全栈项目：

- 公共站点：SSR 渲染的笔记网站
- 后台管理：登录、上传 Markdown、上传资源、编辑与重建
- 内容真源：`content/notes` 与 `content/assets`
- 自动更新：启动服务后可监听内容目录并自动重建
- 链接能力：支持相对 Markdown 链接与 `[[Wiki Link]]`
- 资源能力：支持图片/附件复制到公开 `/assets/*` 路径
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

## 快速启动

```bash
cargo run
```

默认地址：

- 公开站点：`http://127.0.0.1:3000`
- 管理后台：`http://127.0.0.1:3000/admin`

默认管理员账号：

- 用户名：`admin`
- 密码：`Pcsensor1121@`

## 目录约定

```text
content/
  notes/      # Markdown 笔记真源
  assets/     # 共享资源

generated/site/assets/  # 构建后的公开资源

data/app.db             # SQLite 数据库
```

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
- `M2W_UPLOAD_LIMIT_MB`

## 验证

```bash
cargo check
cargo test
```
