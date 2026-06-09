---
title: AList + rclone + Jellyfin（百度网盘）最简完整指南
slug: alist-rclone-jellyfin-百度网盘-最简完整指南
summary: ''
category: []
tags: []
status: published
updated: 2026-05-26T10:42
aliases: []
---
# AList + rclone + Jellyfin（百度网盘）最简完整指南

目标：

```text
百度网盘
  ↓
AList(WebDAV + 本地代理)
  ↓
rclone mount
  ↓
Jellyfin
```

适用于：

```text
小硬盘服务器
百度网盘 SVIP
Jellyfin 影视库
```

---

# 1. Docker 目录结构

```text
/root/docker/
├── alist
│   └── docker-compose.yml
└── jellyfin
    └── docker-compose.yml
```

---

# 2. AList 配置

## docker-compose.yml

路径：

```text
/root/docker/alist/docker-compose.yml
```

内容：

```yaml
version: '3.8'

services:
  openlist:
    image: xhofe/alist:latest
    container_name: openlist
    restart: always

    ports:
      - "127.0.0.1:5244:5244"

    volumes:
      - ./openlist_data:/opt/alist/data
```

启动：

```bash
cd /root/docker/alist
docker compose up -d
```

---

# 3. 配置百度网盘

打开：

```text
http://服务器IP:5244
```

进入：

```text
管理 → 存储 → 添加存储 → 百度网盘
```

完成登录授权。

---

# 4. 关键配置（最重要）

进入：

```text
管理 → 存储 → 百度网盘 → 编辑
```

必须：

```text
开启 本地代理 / Proxy
```

不要：

```text
302 Redirect
```

否则会出现：

```text
31362 sign error
403 Forbidden
```

这是最核心的问题。

---

# 5. 安装 rclone

Debian / Ubuntu：

```bash
apt update
apt install -y rclone fuse3
```

---

# 6. 开启 FUSE allow_other

编辑：

```bash
nano /etc/fuse.conf
```

取消注释：

```text
user_allow_other
```

---

# 7. 配置 rclone

执行：

```bash
rclone config
```

配置：

```text
n
name> alist-baidu

Storage> webdav

url> http://127.0.0.1:5244/dav/

vendor> other

user> AList用户名
pass> AList密码
```

测试：

```bash
rclone lsd alist-baidu:
```

能看到：

```text
百度网盘
本地
```

即可。

---

# 8. 创建挂载目录

```bash
mkdir -p /mnt/alist-baidu
mkdir -p /srv/rclone-cache/alist-baidu
```

---

# 9. 手动挂载测试

服务器配置：

```text
磁盘剩余：20GB
内存：4GB
```

推荐参数：

```bash
rclone mount alist-baidu:/ /mnt/alist-baidu \
  --allow-other \
  --vfs-cache-mode full \
  --vfs-cache-max-size 8G \
  --vfs-cache-max-age 24h \
  --vfs-read-chunk-size 16M \
  --vfs-read-chunk-size-limit 128M \
  --buffer-size 16M \
  --dir-cache-time 6h \
  --poll-interval 0 \
  --cache-dir /srv/rclone-cache/alist-baidu \
  --transfers 2 \
  --checkers 4 \
  --umask 022 \
  --read-only \
  --log-level INFO
```

新开 SSH 测试：

```bash
ls -lah /mnt/alist-baidu
```

---

# 10. Jellyfin 配置

路径：

```text
/root/docker/jellyfin/docker-compose.yml
```

内容：

```yaml
services:
  jellyfin:
    image: jellyfin/jellyfin:latest
    container_name: jellyfin
    network_mode: host

    volumes:
      - /root/docker/jellyfin/config:/config
      - /root/docker/jellyfin/cache:/cache
      - /root/docker/jellyfin/log:/log

      # 本地媒体
      - /root/docker/jellyfin/media:/media

      # rclone 挂载
      - /mnt/alist-baidu:/media/cloud:ro,rslave

      # 转码目录
      - /root/docker/jellyfin/transcodes:/transcodes

    environment:
      - TZ=Asia/Shanghai

    restart: unless-stopped
```

创建目录：

```bash
mkdir -p /root/docker/jellyfin/transcodes
```

启动：

```bash
cd /root/docker/jellyfin
docker compose up -d
```

测试：

```bash
docker exec -it jellyfin ls -lah /media/cloud
```

能看到：

```text
百度网盘
本地
```

即可。

---

# 11. Jellyfin 添加媒体库

进入：

```text
控制台 → 媒体库 → 添加媒体库
```

不要添加：

```text
/media/cloud
```

只添加具体目录，例如：

```text
/media/cloud/百度网盘/影视/电影
/media/cloud/百度网盘/影视/电视剧
```

---

# 12. Jellyfin 必关选项

媒体库设置：

```text
关闭 实时监控
关闭 视频预览缩略图
关闭 Trickplay
关闭 章节图提取
```

否则云盘会非常卡。

---

# 13. 判断是否正常

播放时查看：

```text
控制台 → 活动
```

理想状态：

```text
Direct Play
```

不是：

```text
Transcoding
```

---

# 14. 常见问题

## 问题 1：Jellyfin 看不到目录

解决：

```bash
docker restart jellyfin
```

因为 rclone mount 要先于 Jellyfin。

---

## 问题 2：31362 sign error / 403

原因：

```text
百度网盘未开启本地代理
```

解决：

```text
AList → 百度网盘存储 → 开启 Proxy
```

---

## 问题 3：拖进度条慢

正常。

原因：

```text
百度网盘是远程读取
```

缓解：

```text
增大 rclone cache
尽量 Direct Play
不要播放高码率 REMUX
```

---

# 15. systemd 开机自启（最终）

创建：

```bash
nano /etc/systemd/system/rclone-alist-baidu.service
```

内容：

```ini
[Unit]
Description=Rclone Mount AList Baidu
After=network-online.target docker.service

[Service]
Type=simple

ExecStart=/usr/bin/rclone mount alist-baidu:/ /mnt/alist-baidu \
  --config /root/.config/rclone/rclone.conf \
  --allow-other \
  --vfs-cache-mode full \
  --vfs-cache-max-size 8G \
  --vfs-cache-max-age 24h \
  --vfs-read-chunk-size 16M \
  --vfs-read-chunk-size-limit 128M \
  --buffer-size 16M \
  --dir-cache-time 6h \
  --poll-interval 0 \
  --cache-dir /srv/rclone-cache/alist-baidu \
  --transfers 2 \
  --checkers 4 \
  --umask 022 \
  --read-only

ExecStop=/bin/fusermount3 -uz /mnt/alist-baidu

Restart=on-failure

[Install]
WantedBy=multi-user.target
```

启动：

```bash
systemctl daemon-reload
systemctl enable --now rclone-alist-baidu.service
```

自动清理 `rclone` 缓存脚本：

```bash
#!/usr/bin/env bash
set -e

systemctl stop rclone-alist-baidu.service || true
fusermount3 -uz /mnt/alist-baidu || fusermount -uz /mnt/alist-baidu || true

rm -rf /srv/rclone-cache/alist-baidu/*

systemctl start rclone-alist-baidu.service
docker restart jellyfin

df -h
du -sh /srv/rclone-cache/alist-baidu || true
```

百度网盘添加剧集后：

```bash
systemctl restart rclone-alist-baidu.service
docker restart jellyfin
```

---

# 16. 重启后检查

```bash
ls -lah /mnt/alist-baidu

docker exec -it jellyfin ls -lah /media/cloud
```

如果 Jellyfin 看不到：

```bash
docker restart jellyfin
```

---

# 17. 最终推荐

适合：

```text
1080p
普通 H.264/H.265
中低码率 4K
```

不推荐：

```text
蓝光原盘
REMUX
100Mbps 高码率
频繁大跨度拖进度条
```

核心原则：

```text
AList 必须开启本地代理
Jellyfin 只扫具体影视目录
rclone cache 不要过大
优先 Direct Play
```

# 18. 限速配置

换成 **Linux 内核层限速：tc + CAKE**。
它不管 Jellyfin、Nginx、Caddy，直接在服务器网卡出口整形，最接近“源头限速”。🧱

## 1. 确认出口网卡

```bash
ip route get 1.1.1.1
```

看 `dev` 后面的网卡名，例如：

```text
dev eth0
```

下面假设网卡是 `eth0`。

---

## 2. 安装 tc

Debian / Ubuntu：

```bash
apt update
apt install -y iproute2
```

---

## 3. 删除旧规则

```bash
tc qdisc del dev eth0 root 2>/dev/null || true
```

---

## 4. 设置出口总带宽 + 按目标 IP 公平分配

你的服务器是 **20Mbps**，建议限制到 **18Mbps**，留一点余量：

```bash
tc qdisc replace dev eth0 root cake bandwidth 18mbit besteffort dual-dsthost
```

效果：

```text
1 个人看：最多可用接近 18Mbps
2 个人看：大约每人 9Mbps
3 个人看：大约每人 6Mbps
```

这比 Nginx 更稳，因为它在网卡出口层面生效。

---

## 5. 验证

播放时查看：

```bash
tc -s qdisc show dev eth0
```

看是否有流量统计增长。

也可以看实时带宽：

```bash
apt install -y nload
nload eth0
```

---

## 6. 开机自启

创建：

```bash
nano /etc/systemd/system/tc-cake-egress.service
```

内容：

```ini
[Unit]
Description=CAKE egress shaping
After=network-online.target
Wants=network-online.target

[Service]
Type=oneshot
RemainAfterExit=yes

ExecStart=/bin/bash -c 'tc qdisc del dev eth0 root 2>/dev/null || true; tc qdisc replace dev eth0 root cake bandwidth 18mbit besteffort dual-dsthost'
ExecStop=/bin/bash -c 'tc qdisc del dev eth0 root 2>/dev/null || true'

[Install]
WantedBy=multi-user.target
```

启用：

```bash
systemctl daemon-reload
systemctl enable --now tc-cake-egress.service
```

---

## 重要前提

Cloudflare 必须是：

```text
DNS only / 灰色云朵
```

如果走 Cloudflare 橙色云朵，服务器看到的目标 IP 主要是 Cloudflare 节点，不是真实用户，按用户公平分配会失效。

这是更接近生产里的“出口整形”。
