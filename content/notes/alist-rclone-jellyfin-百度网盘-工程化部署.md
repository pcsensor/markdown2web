---
title: AList + rclone + Jellyfin（百度网盘）工程化部署
slug: alist-rclone-jellyfin-百度网盘-工程化部署
summary: 实机验证版：统一 /opt/media-stack、systemd 依赖链、本机绑定 + Caddy 并站、2G 内存参数、媒体库电影/电视剧分目录，以及加片后刷缓存运维流程。
category: [运维, 媒体]
tags: [AList, rclone, Jellyfin, 百度网盘, systemd, Docker, Caddy]
status: published
updated: 2026-07-20T18:30
aliases: [工程化媒体栈, baidu-jellyfin-prod]
---

# AList + rclone + Jellyfin（百度网盘）工程化部署

> 本文是 [[AList + rclone + Jellyfin（百度网盘）最简完整指南]] 的**工程化升级版**，并吸收了 **Debian 12 / ~2G 内存 / 已有 Caddy 多站点** 实机部署结论。  
> 数据流与「必须 Proxy」等铁律不变；本文侧重：**可重复安装、启动顺序、本机-only + Caddy、分库路径、加片运维**。

## 0. 目标与边界

### 数据流

```text
百度网盘
  ↓  本地代理 / Proxy（禁止 302）
AList WebDAV  127.0.0.1:5244
  ↓
rclone mount（只读 FUSE + VFS 磁盘缓存）
  ↓  ro,rslave
Jellyfin  127.0.0.1:8096
  ↓
Caddy HTTPS（与其它站点并排，禁止覆盖整份 Caddyfile）
  · alist.example.com
  · jellyfin.example.com
```

### 实机约束（写进流程，不要事后才发现）

| 条件 | 实机结论 | 部署动作 |
|------|----------|----------|
| 内存 ~2GB | Jellyfin+Docker 易 OOM | **先加 2G swap**；rclone 降并发/buffer |
| Debian 12 包名 | 常是 `docker-compose` v1，不是 plugin | unit/脚本统一写 **`docker-compose`** |
| 已有 Caddy 站点 | easyshare / markdown2web 等 | **只追加**站点块，先 `cp` 备份 |
| 无 `rsync` | 部分精简镜像 | 用 `cp -a` 即可 |
| 空 AList 无存储 | `rclone lsd` → directory not found | 先加至少一个存储（百度或本地）再测 WebDAV |
| AList 改密 | rclone 立刻 401 | 改密后必须同步 `rclone.conf` 并重启 mount |

### 核心原则

```text
AList 百度存储必须本地代理（Proxy），禁止 302
应用只绑 127.0.0.1，公网只走 Caddy 443
Jellyfin 分「电影 / 电视剧」两个库，只扫具体子目录
rclone 挂载只读；整理网盘目录走 AList/百度，不走 FUSE
加片后必须刷目录缓存，再在 Jellyfin 扫库
优先 Direct Play；关缩略图 / Trickplay / 章节图 / 实时监控
```

### 与「最简指南」对照

| 项 | 最简版 | 工程化 / 实机版 |
|----|--------|-----------------|
| 目录 | `/root/docker/*` | `/opt/media-stack/` |
| 启动 | 手启、易乱序 | Docker → openlist → rclone → jellyfin |
| Jellyfin 网络 | host 暴露 8096 | **`127.0.0.1:8096:8096` + Caddy** |
| 媒体库 | 易扫整个 cloud 根 | **电影 / 电视剧分目录 + 两库** |
| 加片 | 常忘刷缓存 | `rescan-after-upload.sh` 固定流程 |
| 限速 | 可选 CAKE | 可选；实机示例 `ens17` + `50mbit` |

---

## 1. 推荐落地顺序（按序一次做完）

按依赖从底向上，**不要**在百度未配好时就启 jellyfin-stack：

```text
 1. swap（≤2G 内存强烈建议）+ apt：docker.io docker-compose fuse3 rclone
 2. user_allow_other + 目录 /opt/media-stack、/mnt、/srv/rclone-cache
 3. 写 compose / unit / scripts，enable openlist-stack → 起 AList
 4. 记录 admin 初始密码；（可选）Caddy 先挂 alist 域名便于操作
 5. 【人工】AList 添加百度网盘 + 开启 Proxy
 6. 写 rclone.conf（WebDAV）→ rclone lsd 验收
 7. enable rclone-alist-baidu → findmnt 验收
 8. 【人工】在百度/AList 建好 电影、电视剧 目录并放片
 9. rescan / 确认挂载路径
10. enable jellyfin-stack（ConditionPathIsMountPoint）
11. Caddy 追加 alist + jellyfin 域名；确认 5244/8096 仅本机
12. 【人工】Jellyfin 向导 + 添加两个媒体库 + 关四项
13. （可选）CAKE 出口
14. healthcheck + 备份 data/config/rclone.conf
```

**需要人工介入的只有：** 百度 OAuth、网盘目录整理、Jellyfin 网页向导与扫库。其余可脚本化。

---

## 2. 统一目录布局

```text
/opt/media-stack/
├── OPERATOR_NOTES.txt          # 本机备忘（chmod 600，勿提交）
├── scripts/
│   ├── healthcheck.sh
│   ├── cache-purge.sh
│   └── rescan-after-upload.sh
├── systemd/
│   ├── openlist-stack.service
│   ├── rclone-alist-baidu.service
│   ├── jellyfin-stack.service
│   └── tc-cake-egress.service    # 可选
├── alist/
│   ├── docker-compose.yml
│   └── data/                     # AList 状态
├── jellyfin/
│   ├── docker-compose.yml
│   ├── config/                   # 向导后有状态，勿丢
│   ├── cache/
│   ├── log/
│   ├── media/
│   └── transcodes/
└── caddy/
    └── sites.snippet             # 追加用片段（勿整文件覆盖 /etc/caddy/Caddyfile）

/mnt/alist-baidu                  # FUSE 挂载点（只读）
/srv/rclone-cache/alist-baidu     # VFS 缓存（可删可重建）
/root/.config/rclone/rclone.conf  # WebDAV 凭据，chmod 600
```

---

## 3. 依赖图与硬规则

```text
network-online.target
        │
        ├─► tc-cake-egress（可选，独立）
        │
        ▼
  docker.service
        │
        ▼
  openlist-stack.service   →  容器 openlist @ 127.0.0.1:5244
        │
        ▼
  rclone-alist-baidu.service
  ExecStartPre: curl 5244
  mount /mnt/alist-baidu
        │
        ▼
  jellyfin-stack.service
  ConditionPathIsMountPoint=/mnt/alist-baidu
  → 容器 jellyfin @ 127.0.0.1:8096
        │
        ▼
  caddy（已有进程）
  alist.*     → 127.0.0.1:5244
  jellyfin.*  → 127.0.0.1:8096
```

1. 先有 AList WebDAV，再 mount。  
2. 先 **真正挂载**（`findmnt`），再起 Jellyfin。  
3. 卷必须 **`ro,rslave`**。  
4. 百度 **Proxy on / 302 off**。  
5. **Caddy 只追加**，与 chat-transfer / markdown2web 等共存。

---

## 4. 前置：swap 与软件包

```bash
# --- 内存 ≤2G：先 swap，再装 Docker ---
if ! swapon --show | grep -q .; then
  fallocate -l 2G /swapfile || dd if=/dev/zero of=/swapfile bs=1M count=2048
  chmod 600 /swapfile
  mkswap /swapfile
  swapon /swapfile
  grep -q '/swapfile' /etc/fstab || echo '/swapfile none swap sw 0 0' >> /etc/fstab
fi

export DEBIAN_FRONTEND=noninteractive
apt update
# Debian 12 实测：docker-compose（v1 包名）；若有 compose plugin 可改命令
apt install -y docker.io docker-compose fuse3 rclone curl iproute2 ca-certificates

systemctl enable --now docker

# FUSE
sed -i 's/^#user_allow_other/user_allow_other/' /etc/fuse.conf 2>/dev/null || true
grep -q '^user_allow_other' /etc/fuse.conf || echo 'user_allow_other' >> /etc/fuse.conf

mkdir -p \
  /opt/media-stack/{alist/data,jellyfin/{config,cache,log,media,transcodes},scripts,systemd,caddy} \
  /mnt/alist-baidu \
  /srv/rclone-cache/alist-baidu
```

---

## 5. AList（本机端口）

### 5.1 compose

`/opt/media-stack/alist/docker-compose.yml`：

```yaml
version: "3.8"
services:
  openlist:
    image: xhofe/alist:latest
    container_name: openlist
    restart: unless-stopped
    ports:
      - "127.0.0.1:5244:5244"
    volumes:
      - ./data:/opt/alist/data
```

### 5.2 systemd 包装

`/opt/media-stack/systemd/openlist-stack.service`：

```ini
[Unit]
Description=OpenList (AList) stack
After=network-online.target docker.service
Requires=docker.service

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=/opt/media-stack/alist
ExecStart=/usr/bin/docker-compose up -d
ExecStop=/usr/bin/docker-compose stop
TimeoutStartSec=180

[Install]
WantedBy=multi-user.target
```

```bash
cp /opt/media-stack/systemd/openlist-stack.service /etc/systemd/system/
systemctl daemon-reload
systemctl enable --now openlist-stack.service
docker logs openlist 2>&1 | tail -30
# 首次启动日志含：initial password is: ********
```

### 5.3 密码与登录

| 场景 | 做法 |
|------|------|
| 首次 | `docker logs openlist` 看初始密码；用户名多为 `admin` |
| 忘记 / 与 rclone 不一致 | `docker exec -w /opt/alist openlist ./alist admin set '新密码'` |
| 改密后 | **同步** `rclone.conf` 的 `pass`（`rclone obscure`）并 `systemctl restart rclone-alist-baidu` |

```bash
# 验证登录 API
curl -sS -X POST http://127.0.0.1:5244/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"admin","password":"你的密码"}'
```

公网未就绪时用隧道：

```bash
ssh -L 5244:127.0.0.1:5244 root@服务器
# http://127.0.0.1:5244
```

### 5.4 百度存储（人工）

1. 管理 → 存储 → 添加 → **百度网盘** → OAuth。  
2. 编辑存储：**开启本地代理 / Proxy**。  
3. **关闭 / 不要用 302 Redirect**。  
4. 验收：后台能浏览；`rclone lsd` 能看到挂载名（如实机的 `百度网盘`）。

可选：加一个 Local 存储，避免「零存储时 WebDAV Depth:1 异常」干扰排障。

---

## 6. rclone

### 6.1 非交互写入 conf（推荐）

```bash
mkdir -p /root/.config/rclone
PASS_OBSCURED="$(rclone obscure 'AList管理员密码')"
cat > /root/.config/rclone/rclone.conf <<EOF
[alist-baidu]
type = webdav
url = http://127.0.0.1:5244/dav/
vendor = other
user = admin
pass = ${PASS_OBSCURED}
EOF
chmod 600 /root/.config/rclone/rclone.conf

rclone lsd alist-baidu:
# 期望：百度网盘 等
```

401 → 密码不同步；directory not found 且无任何存储 → 先加存储。

### 6.2 小内存推荐挂载参数（实机 ~2G）

比「4G 内存 / 8G cache」更保守：

| 参数 | 小内存建议 |
|------|------------|
| `--vfs-cache-max-size` | `4G` |
| `--buffer-size` | `8M` |
| `--vfs-read-chunk-size` | `8M` |
| `--vfs-read-chunk-size-limit` | `64M` |
| `--transfers` / `--checkers` | `1` / `2` |
| `--read-only` | 开 |

内存更宽裕时可调回最简指南的 8G / 16M / transfers 2。

### 6.3 systemd 挂载单元

```ini
[Unit]
Description=Rclone mount AList Baidu WebDAV to /mnt/alist-baidu
After=network-online.target docker.service
Wants=network-online.target
Requires=docker.service

[Service]
Type=simple
Environment=RCLONE_CONFIG=/root/.config/rclone/rclone.conf
ExecStartPre=/bin/bash -c 'for i in $(seq 1 60); do curl -fsS -o /dev/null http://127.0.0.1:5244/ && exit 0; sleep 2; done; echo "AList not ready"; exit 1'
ExecStartPre=/bin/mkdir -p /mnt/alist-baidu /srv/rclone-cache/alist-baidu
ExecStart=/usr/bin/rclone mount alist-baidu:/ /mnt/alist-baidu \
  --config /root/.config/rclone/rclone.conf \
  --allow-other \
  --vfs-cache-mode full \
  --vfs-cache-max-size 4G \
  --vfs-cache-max-age 24h \
  --vfs-read-chunk-size 8M \
  --vfs-read-chunk-size-limit 64M \
  --buffer-size 8M \
  --dir-cache-time 6h \
  --poll-interval 0 \
  --cache-dir /srv/rclone-cache/alist-baidu \
  --transfers 1 \
  --checkers 2 \
  --umask 022 \
  --read-only \
  --log-level INFO
ExecStop=/bin/fusermount3 -uz /mnt/alist-baidu
Restart=on-failure
RestartSec=5
TimeoutStopSec=30

[Install]
WantedBy=multi-user.target
```

```bash
cp /opt/media-stack/systemd/rclone-alist-baidu.service /etc/systemd/system/
systemctl daemon-reload
systemctl enable --now rclone-alist-baidu.service
findmnt /mnt/alist-baidu
ls /mnt/alist-baidu
```

---

## 7. 网盘目录约定（电影 / 电视剧分离）

**务必在加 Jellyfin 媒体库之前分好目录。** 混在一个 `jellyfin/` 根下会导致识别痛苦。

### 7.1 推荐树（实机）

AList 根下存储名如实机为 `百度网盘` 时：

```text
百度网盘/last/影视/jellyfin/
├── 电影/
│   └── 电影名 (年份)/...
└── 电视剧/
    └── 剧名/
        ├── S01E01.mp4
        └── Season 01/...
```

挂载后：

| 位置 | 路径 |
|------|------|
| 宿主机 | `/mnt/alist-baidu/百度网盘/last/影视/jellyfin/{电影,电视剧}` |
| 容器内 | `/media/cloud/百度网盘/last/影视/jellyfin/{电影,电视剧}` |

整理目录时：

- **走 AList / 百度**（可写）  
- **不要**指望只读 FUSE `mv`  
- **不必**为了整理而先停 Jellyfin；改完后 **必须刷 rclone 目录缓存**（见 §10）

---

## 8. Jellyfin（本机端口 + 依赖挂载）

### 8.1 compose（生产推荐：不要 host 网络裸奔 8096）

```yaml
version: "3.8"
services:
  jellyfin:
    image: jellyfin/jellyfin:latest
    container_name: jellyfin
    ports:
      - "127.0.0.1:8096:8096"
    volumes:
      - ./config:/config
      - ./cache:/cache
      - ./log:/log
      - ./media:/media
      - /mnt/alist-baidu:/media/cloud:ro,rslave
      - ./transcodes:/transcodes
    environment:
      - TZ=Asia/Shanghai
      # 换成真实域名
      - JELLYFIN_PublishedServerUrl=https://jellyfin.example.com
    restart: unless-stopped
```

> 实机曾用 `network_mode: host` 导致 **0.0.0.0:8096 对公网暴露**。上 Caddy 后务必改为 **仅 127.0.0.1**。

### 8.2 systemd

```ini
[Unit]
Description=Jellyfin stack (after rclone AList mount)
After=network-online.target docker.service rclone-alist-baidu.service
Requires=docker.service rclone-alist-baidu.service
ConditionPathIsMountPoint=/mnt/alist-baidu

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=/opt/media-stack/jellyfin
ExecStart=/usr/bin/docker-compose up -d
ExecStop=/usr/bin/docker-compose stop
TimeoutStartSec=180

[Install]
WantedBy=multi-user.target
```

```bash
systemctl enable --now jellyfin-stack.service
docker exec jellyfin ls -lah "/media/cloud/百度网盘/last/影视/jellyfin"
```

### 8.3 媒体库（UI）

添加 **两个** 库，路径示例：

| 类型 | 文件夹 |
|------|--------|
| 电影 | `/media/cloud/百度网盘/last/影视/jellyfin/电影` |
| 电视剧 | `/media/cloud/百度网盘/last/影视/jellyfin/电视剧` |

**不要**添加 `/media/cloud` 或整个 `jellyfin` 根（除非下面只有一类且你接受混扫）。

每个库关闭：

| 选项 | 状态 |
|------|------|
| 实时监控 | 关 |
| 视频预览缩略图 | 关 |
| Trickplay | 关 |
| 章节图提取 | 关 |

播放：控制台 → 活动 → 尽量 **Direct Play**。

### 8.4 扫库为什么慢？（预期管理）

慢 **正常**，且一般 **不是**「整部影片完整下载进本地盘」：

```text
扫库 = 列目录 + 对每个文件 ffprobe 读头/探针
     → rclone → AList Proxy → 百度
     → 片段进入 VFS 缓存（可见 /srv/rclone-cache 上涨）
```

- 会有可观远程读流量（尤其 probesize 较大时）  
- 扫完后元数据在 Jellyfin DB，浏览会快很多  
- 首次播放仍可能顿一下（开始拉流）  

---

## 9. Caddy 并排反代

### 9.1 原则

1. `cp -a /etc/caddy/Caddyfile /etc/caddy/Caddyfile.bak.$(date +%Y%m%d-%H%M%S)`  
2. **追加**站点块，不删 easyshare / markdown2web 等  
3. DNS A 记录指向本机；Cloudflare 建议 **DNS only（灰云）**（CAKE 按用户公平也依赖此）  
4. `caddy validate` → `systemctl reload caddy`  

### 9.2 片段示例

```caddyfile
# --- alist (alist.example.com) ---
alist.example.com {
	encode gzip
	reverse_proxy 127.0.0.1:5244 {
		header_up Host {host}
		header_up X-Real-IP {remote_host}
		transport http {
			read_timeout 3600s
			write_timeout 3600s
		}
	}
}

# --- jellyfin (jellyfin.example.com) ---
jellyfin.example.com {
	encode gzip
	reverse_proxy 127.0.0.1:8096 {
		header_up Host {host}
		header_up X-Real-IP {remote_host}
		header_up Connection {http.request.header.Connection}
		header_up Upgrade {http.request.header.Upgrade}
		transport http {
			read_timeout 3600s
			write_timeout 3600s
		}
	}
}
```

### 9.3 端口验收

```bash
ss -lptn | grep -E ':5244|:8096|:80|:443'
# 5244 / 8096 必须是 127.0.0.1
# 80 / 443 为 caddy
```

---

## 10. 日常运维：网盘加新剧

```text
1. AList/百度：文件放入 电视剧/剧名 或 电影/...
2. 服务器刷缓存（必做，否则 dir-cache 最长约 6h 仍见旧树）
3. Jellyfin：对对应库「扫描媒体库」
```

### 10.1 一键脚本（推荐）

`/opt/media-stack/scripts/rescan-after-upload.sh`：

```bash
#!/usr/bin/env bash
set -euo pipefail
systemctl restart rclone-alist-baidu.service
sleep 3
findmnt /mnt/alist-baidu
docker restart jellyfin
echo "mount refreshed; jellyfin restarted — scan libraries in UI"
```

```bash
chmod +x /opt/media-stack/scripts/*.sh
/opt/media-stack/scripts/rescan-after-upload.sh
# 然后浏览器扫库
```

### 10.2 健康检查

```bash
/opt/media-stack/scripts/healthcheck.sh
# 期望：alist 200、findmnt 成功、jellyfin 容器能 ls /media/cloud
```

### 10.3 清 VFS 缓存（磁盘紧张时）

`cache-purge.sh`：停 jellyfin-stack → 停 mount → 清空 `/srv/rclone-cache/alist-baidu` → 再起 mount → 起 jellyfin。会短暂中断播放。

### 10.4 改 AList 密码后

```bash
docker exec -w /opt/alist openlist ./alist admin set '新密码'
# 重写 rclone.conf 中 pass=rclone obscure 结果
systemctl restart rclone-alist-baidu.service
rclone lsd alist-baidu:
```

---

## 11. 可选：出口 CAKE

```bash
# 确认网卡
ip route get 1.1.1.1   # 实机曾为 ens17
tc qdisc replace dev ens17 root cake bandwidth 50mbit besteffort dual-dsthost
```

做成 oneshot systemd（`IFACE` / `RATE` 按线路改）。**Cloudflare 须灰云**，否则 dual-dsthost 难按真实用户公平。

---

## 12. 备份与升级

### 备份

| 路径 | 内容 |
|------|------|
| `/opt/media-stack/alist/data` | AList 与 token |
| `/opt/media-stack/jellyfin/config` | 用户与媒体库 |
| `/root/.config/rclone/rclone.conf` | WebDAV |
| `/etc/caddy/Caddyfile`（及 caddy 数据目录） | 站点与证书相关 |

不必备份 `/srv/rclone-cache`。

### 升级

```bash
cd /opt/media-stack/alist && docker-compose pull && docker-compose up -d
cd /opt/media-stack/jellyfin && docker-compose pull && docker-compose up -d
systemctl restart rclone-alist-baidu.service
/opt/media-stack/scripts/healthcheck.sh
```

---

## 13. 上线验收清单

| # | 检查 | 期望 |
|---|------|------|
| 1 | `swapon --show`（小内存） | 有 swap |
| 2 | `curl -sS -o /dev/null -w '%{http_code}\n' http://127.0.0.1:5244/` | 2xx |
| 3 | 百度存储 Proxy | 开；非 302 |
| 4 | `rclone lsd alist-baidu:` | 见网盘根 |
| 5 | `findmnt /mnt/alist-baidu` | fuse.rclone |
| 6 | `ss` 上 5244/8096 | **仅 127.0.0.1** |
| 7 | 宿主机 `ls .../jellyfin/电影` 与 `.../电视剧` | 分目录存在 |
| 8 | `docker exec jellyfin ls .../电视剧` | 与宿主一致 |
| 9 | Jellyfin 两库路径 | 见 §8.3 |
| 10 | 库四项重负载 | 全关 |
| 11 | `https://alist.*` / `https://jellyfin.*` | 证书有效、可登录 |
| 12 | 旧 Caddy 站点 | 仍 200 |
| 13 | 播放活动 | 尽量 Direct Play |
| 14 | `./scripts/healthcheck.sh` | 打印 OK |

---

## 14. 故障速查

| 现象 | 处理 |
|------|------|
| rclone 401 | AList 密码变更 → 同步 conf → 重启 mount |
| rclone directory not found | 无存储 / WebDAV 空；先加存储 |
| Jellyfin 看不到新剧 | `rescan-after-upload.sh` 后扫库 |
| 容器 `/media/cloud` 空 | mount 未就绪就起了容器 → 先 `findmnt` 再 `docker restart jellyfin` |
| 31362 / 403 | 百度存储开 Proxy |
| 8096 被公网扫到 | 改为 `127.0.0.1:8096:8096`，勿 host 裸奔 |
| 扫库极慢 | 预期内；确认未开缩略图等；非整库下载 |
| Caddy 证书失败 | DNS 未指向 / 橙云干扰 HTTP-01 |
| 追加 Caddy 后旧站挂了 | 用 bak 恢复；检查是否误覆盖整文件 |

---

## 15. 脚本参考（完整）

### healthcheck.sh

```bash
#!/usr/bin/env bash
set -euo pipefail
echo "== docker =="; systemctl is-active docker
docker ps --format 'table {{.Names}}\t{{.Status}}' || true
echo "== alist =="
curl -fsS -o /dev/null -w "alist_http:%{http_code}\n" http://127.0.0.1:5244/ || echo "alist_http:FAIL"
echo "== mount =="
systemctl is-active rclone-alist-baidu.service || true
findmnt /mnt/alist-baidu || { echo "NOT_A_MOUNT"; exit 1; }
ls /mnt/alist-baidu >/dev/null
echo "== jellyfin =="
if docker ps --format '{{.Names}}' | grep -qx jellyfin; then
  docker exec jellyfin ls /media/cloud >/dev/null && echo "jellyfin cloud ok"
else
  echo "jellyfin not running"
fi
echo "== disk =="; df -h / | tail -1
du -sh /srv/rclone-cache/alist-baidu 2>/dev/null || true
echo "OK"
```

### cache-purge.sh

```bash
#!/usr/bin/env bash
set -euo pipefail
systemctl stop jellyfin-stack.service 2>/dev/null || docker stop jellyfin 2>/dev/null || true
systemctl stop rclone-alist-baidu.service || true
fusermount3 -uz /mnt/alist-baidu 2>/dev/null || true
rm -rf /srv/rclone-cache/alist-baidu/*
systemctl start rclone-alist-baidu.service
for i in $(seq 1 30); do findmnt /mnt/alist-baidu >/dev/null 2>&1 && break; sleep 1; done
findmnt /mnt/alist-baidu
systemctl start jellyfin-stack.service 2>/dev/null || docker start jellyfin 2>/dev/null || true
docker restart jellyfin 2>/dev/null || true
df -h /
du -sh /srv/rclone-cache/alist-baidu || true
```

---

## 16. 相关笔记

- 最简操作与排错原文：[[AList + rclone + Jellyfin（百度网盘）最简完整指南]]  
- Caddy 与 Docker/宿主机互通：[[Caddy]]
