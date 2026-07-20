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
├── deploy/             # 方案 A：systemd / Nginx / Caddy / 安装与升级脚本
├── src/                # Rust 源代码
├── templates/          # Askama HTML 模板（编译期嵌入二进制）
└── static/             # 静态文件（JS、favicon；运行时 ServeDir）
```

> `content/` 是唯一的内容真源。`generated/` 和 `data/` 均为运行时产出，可安全删除后重建。

## 配置

所有配置通过 `M2W_` 前缀的环境变量设置。默认值覆盖本地开发场景，直接 `cargo run` 即可使用，无需任何配置。

生产环境建议将配置写入 `.env` 文件（参考 `.env.example`）。

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `M2W_HOST` | `127.0.0.1` | 监听地址（方案 A 保持本机；仅内网裸跑时才用 `0.0.0.0`） |
| `M2W_PORT` | `3000` | 监听端口 |
| `M2W_BASE_URL` | `http://127.0.0.1:3000` | 站点基础 URL（生产填公网 `https://域名`） |
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

## 公网部署（方案 A，推荐生产）

与 `web-share` 同一套工程化模型：**二进制 + systemd + 反向代理（Caddy / Nginx）+ HTTPS**。  
应用只监听 `127.0.0.1:3000`，公网只暴露 80/443。

| 项 | 生产建议 |
|----|----------|
| 监听 | `M2W_HOST=127.0.0.1`，`M2W_PORT=3000` |
| 环境文件 | systemd `EnvironmentFile=/opt/markdown2web/env`（权限 `640`，`root:m2w`） |
| 管理员 | 强密码写入 `M2W_ADMIN_PASSWORD`（非默认值会在启动时同步进 SQLite） |
| 文件监听 | `M2W_WATCH_ENABLED=false`（内容走 `/admin`） |
| 工作目录 | `WorkingDirectory=/opt/markdown2web`（须含 `static/`） |
| 可写目录 | `content/`、`generated/`、`data/` |
| 反代 body | ≥ `M2W_UPLOAD_LIMIT_MB`（示例 128MB → 反代约 140m） |
| 可选 | FFmpeg（媒体派生；缺失则降级用原资源）、Turnstile |

### 部署后目录结构

```text
/opt/markdown2web/
├── markdown2web          # 二进制
├── env                   # 密钥与配置（勿提交 git）
├── static/               # JS / CSS / favicon（运行时 ServeDir）
├── content/              # 内容真源（notes + assets，可写）
│   ├── notes/
│   └── assets/
├── generated/            # rebuild 产出（可写）
│   └── site/
│       └── assets/
└── data/                 # SQLite
    └── app.db
```

> Askama 模板在**编译期**打进二进制，无需部署 `templates/`。  
> Windows 本机编译的 `.exe` **不能**直接部署到 Linux。

### 脚本一键（服务器上已有 Caddy + 源码树时）

仓库内提供与 web-share 对齐的脚本（默认路径可按机器修改）：

| 文件 | 用途 |
|------|------|
| `deploy/markdown2web.service` | systemd 单元（硬化 + 可写路径） |
| `deploy/nginx.conf` | Nginx 反代示例 |
| `deploy/Caddyfile.snippet` | Caddy 站点块示例 |
| `deploy/server-deploy-a-caddy.sh` | 首次安装：用户/目录/二进制/env/systemd/Caddy |
| `deploy/remote-upgrade.sh` | 升级：备份 → 替换二进制与 `static/` → 重启 |

```bash
# 1. 在服务器（或同架构 Linux）编译
cd /home/justin/pcsensor/markdown2web
cargo build --release

# 2. 首次部署（root）
#    用真实域名覆盖默认 notes.example.com
sudo M2W_DOMAIN=notes.your.domain bash deploy/server-deploy-a-caddy.sh

# 3. 之后升级（保留 content / data / env）
cargo build --release
sudo bash deploy/remote-upgrade.sh
```

首次部署会打印一次性管理员密码（写入 `/opt/markdown2web/env`）。请立刻保存，并按需改 `M2W_BASE_URL` / Turnstile。

### 手工步骤（与脚本等价）

#### 1. 服务器准备

```bash
sudo apt update
sudo apt install -y build-essential pkg-config curl
# 反代二选一：nginx 或 caddy
# sudo apt install -y nginx
# 或安装 Caddy（自动 HTTPS）

# 可选：服务器本机编译
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# 可选媒体处理
sudo apt install -y ffmpeg
```

也可在同架构 Linux 上 `cargo build --release`，再上传：

- `target/release/markdown2web`
- 整个 `static/`
- 需要的 `content/`（或空目录，由后台上传）
- `deploy/markdown2web.service`、`deploy/nginx.conf`（或 Caddy 片段）

#### 2. 系统用户与目录

```bash
sudo useradd --system --home /opt/markdown2web --shell /usr/sbin/nologin m2w || true
sudo mkdir -p \
  /opt/markdown2web/content/notes \
  /opt/markdown2web/content/assets \
  /opt/markdown2web/generated/site/assets \
  /opt/markdown2web/data \
  /opt/markdown2web/static
sudo chown -R m2w:m2w /opt/markdown2web
```

#### 3. 安装二进制与静态资源

```bash
cargo build --release

sudo install -o m2w -g m2w -m 550 \
  target/release/markdown2web /opt/markdown2web/markdown2web
sudo rsync -a --delete static/ /opt/markdown2web/static/
# 首次可同步内容；升级时不要覆盖生产 content/
# sudo rsync -a content/ /opt/markdown2web/content/
sudo chown -R m2w:m2w /opt/markdown2web
sudo chmod 750 /opt/markdown2web
```

#### 4. 环境文件（生产关键）

```bash
DOMAIN=notes.your.domain
ADMIN_PASS='请改成足够长的随机密码'

sudo tee /opt/markdown2web/env >/dev/null <<EOF
M2W_HOST=127.0.0.1
M2W_PORT=3000
M2W_BASE_URL=https://${DOMAIN}
M2W_SITE_NAME=markdown2web

M2W_CONTENT_DIR=/opt/markdown2web/content
M2W_GENERATED_DIR=/opt/markdown2web/generated/site
M2W_DATA_DIR=/opt/markdown2web/data

M2W_ADMIN_USERNAME=admin
M2W_ADMIN_PASSWORD=${ADMIN_PASS}

M2W_WATCH_ENABLED=false
M2W_TURNSTILE_ENABLED=false
M2W_UPLOAD_LIMIT_MB=128

RUST_LOG=markdown2web=info,tower_http=info
EOF

sudo chown root:m2w /opt/markdown2web/env
sudo chmod 640 /opt/markdown2web/env
```

**注意：**

1. `M2W_ADMIN_PASSWORD` 若不是默认 `admin123456`，启动时会与库中哈希比对，不匹配则更新（便于首次改密）。日常改密优先用后台「修改密码」。
2. 备份务必包含 `env` + `data/` + `content/`。
3. 含空格的值写 `EnvironmentFile` 时须加引号（与 systemd 规则一致）。

#### 5. systemd

```bash
sudo cp deploy/markdown2web.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now markdown2web
sudo systemctl status markdown2web
journalctl -u markdown2web -n 50 --no-pager
```

期望日志中出现：

```text
markdown2web listening on http://127.0.0.1:3000
```

端口占用时：

```bash
sudo ss -lptn 'sport = :3000'
sudo systemctl stop markdown2web
```

#### 6a. Caddy 反代（推荐，自动 HTTPS）

见 `deploy/Caddyfile.snippet`，或把站点块并入 `/etc/caddy/Caddyfile`：

```caddyfile
notes.your.domain {
	encode gzip
	request_body { max_size 140MB }
	reverse_proxy 127.0.0.1:3000 {
		header_up Host {host}
		header_up X-Real-IP {remote_host}
		header_up X-Forwarded-For {remote_host}
		header_up X-Forwarded-Proto {scheme}
		transport http {
			read_timeout 3600s
			write_timeout 3600s
		}
	}
}
```

```bash
sudo caddy validate --config /etc/caddy/Caddyfile
sudo systemctl reload caddy
```

DNS 解析到本机（灰云 / DNS-only）后，Caddy 会自动申请证书。

#### 6b. Nginx 反代

```bash
sudo cp deploy/nginx.conf /etc/nginx/sites-available/markdown2web
sudo ln -sf /etc/nginx/sites-available/markdown2web /etc/nginx/sites-enabled/markdown2web
# 编辑 server_name；client_max_body_size ≥ 上传上限
sudo nginx -t && sudo systemctl reload nginx

# 有域名时：
# sudo apt install -y certbot python3-certbot-nginx
# sudo certbot --nginx -d notes.your.domain
```

#### 7. 防火墙

```bash
sudo ufw allow OpenSSH
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
# 不要对公网开放 3000
sudo ufw enable
```

#### 8. 上线验收

| 检查项 | 方法 |
|--------|------|
| 进程 | `systemctl is-active markdown2web` |
| 本机 | `curl -sS -o /dev/null -w '%{http_code}\n' http://127.0.0.1:3000/health` |
| 公网 HTTPS | 浏览器打开 `https://域名/` |
| 管理后台 | `https://域名/admin`，用 `env` 中账号登录 |
| 上传 / 重建 | 后台上传 md 或点 rebuild，确认 `generated/` 更新 |
| 大文件 | 上传体积接近 `M2W_UPLOAD_LIMIT_MB` 时反代不 413 |

#### 9. 升级与备份

**备份：**

```bash
sudo systemctl stop markdown2web
sudo tar czf ~/markdown2web-backup-$(date +%F).tgz \
  /opt/markdown2web/env \
  /opt/markdown2web/content \
  /opt/markdown2web/data
sudo systemctl start markdown2web
```

**升级：**

```bash
cargo build --release
# 或: sudo bash deploy/remote-upgrade.sh
sudo systemctl stop markdown2web
sudo install -o m2w -g m2w -m 550 \
  target/release/markdown2web /opt/markdown2web/markdown2web
sudo rsync -a --delete static/ /opt/markdown2web/static/
sudo systemctl start markdown2web
journalctl -u markdown2web -n 30 --no-pager
```

**不要**用空库覆盖生产 `data/app.db`，除非有意清空用户/批注/弹幕。

**回滚：** 停服务 → 换回备份的二进制与 `static/` → 启动；`content/` 与 `data/` 尽量与版本兼容使用。

### 生产加固清单（简要）

1. **强管理员密码**（`M2W_ADMIN_PASSWORD` / 后台改密）
2. **关闭监听**（`M2W_WATCH_ENABLED=false`）
3. **HTTPS 反代**，应用不绑公网
4. 可选 **Turnstile**（`M2W_TURNSTILE_ENABLED=true` + site/secret key）
5. 可选将 `/assets` 静态派生放到 CDN（HTML/API 仍由本服务承担）

### 方案 B：内网 / 临时裸跑（不推荐公网长期）

```bash
export M2W_HOST=0.0.0.0
export M2W_PORT=3000
export M2W_BASE_URL=http://服务器IP:3000
export M2W_ADMIN_PASSWORD='...'
export M2W_WATCH_ENABLED=false
cd /path/to/markdown2web   # 需能解析 static/
./target/release/markdown2web
```

安全组放行 TCP 3000。无 TLS 时勿长期暴露公网。

---

## 验证

```bash
cargo check   # 类型检查（快速迭代时使用）
cargo test    # 运行全部集成与单元测试
```

## 项目结构（部署相关）

```text
markdown2web/
├── deploy/
│   ├── markdown2web.service      # systemd
│   ├── nginx.conf                # Nginx 反代
│   ├── Caddyfile.snippet         # Caddy 站点块
│   ├── server-deploy-a-caddy.sh  # 首次方案 A 安装
│   └── remote-upgrade.sh         # 生产升级
├── .env.example
├── content/                      # 内容真源
├── static/                       # 运行时静态资源
├── templates/                    # 编译期嵌入
├── src/
└── ...
```

## 许可证

MIT
