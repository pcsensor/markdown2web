---
title: Caddy
slug: caddy
summary: ''
category: []
tags: []
status: published
updated: 2026-04-14T11:28
aliases: []
---
### 方式：现代极客派（使用 Caddy 自动全包）

既然你都在用 Compose 管理容器了，我们完全可以抛弃繁琐的 Nginx，换成现代化的反向代理工具 **Caddy**。Caddy 是用 Go 语言写的，它的核心杀手锏就是：**原生内置了自动 HTTPS 功能**。

你不需要自己去申请证书、不需要关心续期，也不需要挂载证书文件。只要你把域名解析对，Caddy 会自动帮你完成申请、配置、续签的全流程。

利用 Compose 和 Caddy 部署 OpenList 并实现自动 HTTPS 与 IP 监测的完整方案如下：

**1. 准备 `docker-compose.yml`**

这里我们利用 Docker 自带的网络机制，让 Caddy 直接通过容器名访问后端，连 `127.0.0.1:5244` 的本地端口映射都省了，真正做到公网端口零暴露。

```yaml
version: '3.8'

services:
  openlist:
    image: xhofe/alist:latest # 你的 OpenList 镜像
    container_name: openlist
    restart: always
    volumes:
      - ./openlist_data:/opt/alist/data
    # 注意：这里不需要 ports 暴露端口，直接在 Docker 虚拟网络内通信

  caddy:
    image: caddy:latest
    container_name: caddy_proxy
    restart: always
    ports:
      - "80:80"   # 用于 HTTP 强制跳转 HTTPS 以及 Let's Encrypt 验证
      - "443:443" # 真正的 HTTPS 端口
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile
      - ./caddy_data:/data            # 极其重要：用于持久化保存自动申请的 SSL 证书，防止重启后重复申请被限流
      - ./caddy_config:/config
      - ./caddy_logs:/var/log/caddy   # 用于存放访问日志，监测 IP
    depends_on:
      - openlist
```

**2. 准备 `Caddyfile`**

在 `docker-compose.yml` 同级目录下创建一个名为 `Caddyfile` 的文件（没有任何后缀）。配置简单到令人发指：

```caddyfile
# 把这里换成你解析好的真实域名
your_domain.com {
    # 反向代理到 openlist 容器的 5244 端口
    reverse_proxy openlist:5244
    
    # 开启日志记录，方便你监测访问者的 IP
    log {
        output file /var/log/caddy/access.log {
            roll_size 100mb
            roll_keep 3
        }
        format json
    }
}
```

启动 `docker compose up -d` 后，Caddy 会自动向 Let's Encrypt 申请证书，几秒钟后，你的 WebDAV 就可以通过 `https://your_domain.com` 安全访问了。想要看访问 IP，直接 `cat ./caddy_logs/access.log` 即可。

---

### 如果没有域名怎么办？

如果你目前只有这台公共服务器的公网 IP，没有注册域名，那么走常规的 HTTPS 路线会非常艰难（因为客户端系统和大部分 WebDAV 软件会强制拦截自签名的不受信任证书）。

这种情况下，为了避免 HTTP 明文传输的风险，最干净利落的替代方案是**不暴露任何公网端口**，直接在服务器和你的客户端（比如 macOS）之间建立点对点的加密隧道：

* 使用 **Tailscale** 或 **ZeroTier**。
* 让你的服务器和电脑处于同一个虚拟局域网（例如被分配了 `10.147.x.x` 的内网 IP）。
* 你在 macOS 上直接通过类似 `http://10.147.x.x:5244/dav` 进行访问。

此时数据传输依然是加密的（由 Tailscale/ZeroTier 的底层 WireGuard 协议负责加密），完美避开了证书申请的硬性要求，且服务器在公网上如同隐形，安全性直接拉满。