---
title: Caddy
slug: caddy
summary: Caddy配置文件，最初学习用的，现在还是交给AI完成吧
category: []
tags: []
status: published
updated: 2026-04-14T12:46
aliases: []
---
## Caddy反向代理alist

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

## Caddy反向代理markdown2web

用 Rust 写服务并用 systemctl 守护进程管理，这是非常标准且硬核的生产级部署方式。

由于你之前的 Caddy 是运行在 **Docker 容器** 里的，而你的 Rust 服务是运行在 **宿主机 (Host)** 上的，这里有一个非常经典的“网络隔离陷阱”：**如果在 Caddyfile 里直接写 `127.0.0.1:3000`，Caddy 会去自己的容器内部找 3000 端口，从而导致 502 报错。**

为了解决这个问题并添加反向代理，你需要完成两步：打通网络，以及修改 Caddyfile。

### 第一步：打通 Docker 到宿主机的网络

你需要让 Docker 里的 Caddy 能够识别宿主机的 IP。最优雅的方式是修改你的 `docker-compose.yml`，给 Caddy 容器添加一个特殊的 host 映射。

打开你的 `docker-compose.yml`，在 `caddy` 服务下添加 `extra_hosts` 字段：

```yaml
  caddy:
    image: caddy:latest
    container_name: caddy_proxy
    restart: always
    ports:
      - "80:80"
      - "443:443"
    # 新增下面这两行，把宿主机网关映射为一个域名
    extra_hosts:
      - "host.docker.internal:host-gateway"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile
      # ... 其他挂载保持不变 ...
```
修改完成后，运行 `docker compose up -d` 让网络配置生效。此时，在 Caddy 容器内，`host.docker.internal` 就等同于你宿主机的真实 IP。

---

### 第二步：修改 Caddyfile

接下来，你面临一个架构选择：你是想用一个**新的子域名**来访问 Rust 服务，还是想在现有的 `your_domain.com` 下分出一个**子路径**？

我为你提供这两种方案的 Caddyfile 语法：

#### 方案 A：子域名模式（推荐，最干净利落）
解析一个新的二级域名（比如 `api.your_domain.com`）到你的服务器。Rust 服务和 OpenList 完全独立。

```caddyfile
# 原有的 OpenList 保持不变
your_domain.com {
    reverse_proxy openlist:5244
    log {
        output file /var/log/caddy/access.log {
            roll_size 100mb
            roll_keep 3
        }
        format json
    }
}

# 新增的 Rust 服务块
api.your_domain.com {
    # 代理到刚刚在 compose 里映射的宿主机地址
    reverse_proxy host.docker.internal:3000
    
    log {
        output file /var/log/caddy/rust_access.log {
            roll_size 100mb
            roll_keep 3
        }
        format json
    }
}
```

#### 方案 B：子路径模式（适合不想折腾新域名的场景）
访问 `your_domain.com/api/` 的流量去 Rust 服务，其他的默认去 OpenList。
*注意：这里使用了 Caddy 的路由匹配器 `handle_path`，它会自动把 `/api` 前缀剥离掉再发给你的 Rust 监听端，这样你的 Rust 代码里就不需要专门处理 `/api` 这个前缀了。*

```caddyfile
your_domain.com {
    # 1. 优先匹配 /api/ 开头的请求，转发给宿主机的 Rust 服务
    handle_path /api/* {
        reverse_proxy host.docker.internal:3000
    }

    # 2. 剩下的所有请求，兜底转发给 OpenList
    handle {
        reverse_proxy openlist:5244
    }
    
    log {
        output file /var/log/caddy/access.log {
            roll_size 100mb
            roll_keep 3
        }
        format json
    }
}
```

### 第三步：热重载 Caddy

无论你选择了哪种 Caddyfile 方案，保存文件后，无需重启整个容器导致服务中断，直接让 Caddy 热重载配置即可。在宿主机终端执行：

```bash
docker exec -w /etc/caddy caddy_proxy caddy reload
```

只要终端没有报错输出，你的 Rust 服务就已经被成功包裹在 HTTPS 下，并且可以通过外网安全访问了。
