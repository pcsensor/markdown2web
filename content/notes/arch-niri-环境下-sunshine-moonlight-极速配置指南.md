---
title: Arch + niri 环境下 Sunshine/Moonlight 极速配置指南
slug: arch-niri-环境下-sunshine-moonlight-极速配置指南
summary: ''
category: []
tags: []
status: published
updated: 2026-05-14T20:42
aliases: []
---
## Arch + niri 环境下 Sunshine/Moonlight 极速配置指南

基于我们刚才的排查与最终成功启动的结果，针对使用最新 Beta 版 Sunshine（`sunshine-git`）以及 niri Wayland 合成器的系统，这是最快、最精简的标准配置流程。

### 一、 核心组件安装

**1. 安装 Sunshine (Beta 版)**
为了获得最好的 Wayland 和 XDG Portal 支持，避免旧版的兼容性问题，直接安装 `git` 稳定版：

```bash
sudo pacman -S lizardbyte-beta/sunshine-git

```

**2. 安装屏幕捕获依赖与硬件编码驱动**
确保安装完整的 PipeWire 体系、Portal 框架，以及符合你设备的硬件编码支持（例如为 GTX 1650 准备的 NVIDIA 编码驱动，或为 AMD 设备准备的 VAAPI 驱动）：

```bash
sudo pacman --needed -S pipewire wireplumber \
    xdg-desktop-portal xdg-desktop-portal-gnome \
    xdg-desktop-portal-gtk polkit-gnome

```

*(注：AMD 环境通常已由 `libva-mesa-driver` 提供支持，无需额外安装。)*

---

### 二、 配置 niri 屏幕捕获 (Portal)

niri 环境下，最稳定的画面截取方案是通过 GNOME 的 Portal 配合 PipeWire。

**1. 写入 Portal 配置**
编辑或新建配置文件 `~/.config/xdg-desktop-portal/niri-portals.conf`，写入以下内容：

```ini
[preferred]
default=gtk
org.freedesktop.impl.portal.ScreenCast=gnome
org.freedesktop.impl.portal.RemoteDesktop=gnome
org.freedesktop.impl.portal.Settings=gnome

```

**2. 重启 Portal 服务使配置生效**

```bash
systemctl --user restart xdg-desktop-portal.service
systemctl --user restart xdg-desktop-portal-gnome.service

```

---

### 三、 权限赋予与服务启动（关键变更）

在新版的 Sunshine 中，Systemd 服务名称已经更新为标准的 AppStream 命名格式。

**1. 赋予底层捕获权限 (为 KMS 模式备用)**
赋予二进制文件特权，以便在需要时直接读取 GPU 帧缓冲：

```bash
sudo setcap cap_sys_admin+p $(readlink -f $(which sunshine))

```

**2. 启动并配置服务开机自启**
刷新用户级 systemd 守护进程，并启动最新名称的 Sunshine 服务：

```bash
systemctl --user daemon-reload
systemctl --user --now enable app-dev.lizardbyte.app.Sunshine.service

```

---

### 四、 Sunshine Web UI 设置

**1. 初始化管理员账号**
在 Arch 主机上使用浏览器访问 `https://localhost:47990`（如有“不安全”提示直接忽略并继续访问）。首次进入会强制要求设置管理员的用户名和密码。

**2. 配置捕获方式 (Capture Method)**
登录后，导航至 **Configuration** -> **Audio/Video** 标签页：

* **Capture Method**: 优先选择 `portal`（目前对 niri 兼容性最好）。如果后续感觉有延迟或遇到个别黑屏问题，再尝试切换为 `kms`。
* **Encoder**: 保持默认的自动即可，系统会自动调用相应的硬件编码器进行加速。
* 滚动到底部点击 **Save** 并重启应用配置。

---

### 五、 Moonlight 客户端配对 (macOS / iOS)

由于 Apple 的安全策略，首次配对必须在同一局域网下进行。

1. **处于同网络**：确保你的 iPhone、iPad 或 macOS 设备连接到与 Arch 主机相同的局域网。
2. **发现主机**：打开客户端的 Moonlight，等待主页面自动刷新并浮现出你的 Arch 电脑。
3. **获取 PIN 码**：点击该主机图标，Moonlight 屏幕上会显示一个随机的 4 位数 PIN 码。
4. **后台验证**：
* 回到 Arch 主机的 Sunshine Web UI，点击顶栏菜单中的 **PIN**。
* 输入设备屏幕上的 4 位 PIN 码并提交。


5. **开始串流**：配对完成后，Moonlight 上的主机锁形图标会解锁。直接点击默认的 "Desktop" 选项，即可流畅远控 niri 桌面。