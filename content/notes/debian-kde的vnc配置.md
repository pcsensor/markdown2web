---
title: Debian-KDE的vnc配置
slug: debian-kde的vnc配置
summary: ''
category: []
tags: []
status: published
updated: 2026-04-23T12:42
aliases: []
---
# 步骤

```sh
# 1. 安装 KDE Plasma 桌面环境（如未安装）
sudo apt update
sudo apt install kde-plasma-desktop

# 2. 安装 TigerVNC 服务器和 D-Bus X11 支持
sudo apt install tigervnc-standalone-server tigervnc-common dbus-x11

# 3. 设置 VNC 密码
vncpasswd

# 4. 创建/编辑 xstartup 文件
nano ~/.vnc/xstartup

# 5. 粘贴上面的配置内容，然后赋予执行权限
chmod +x ~/.vnc/xstartup

# 6. 启动 VNC 服务器
vncserver :1 -geometry 1920x1080 -depth 24 -localhost no

# 7. 客户端连接：vnc://<服务器IP>:5901
```

# 配置文件内容

```sh
#!/bin/sh
# unset SESSION_MANAGER
# unset DBUS_SESSION_BUS_ADDRESS
export XKL_XMODMAP_DISABLE=1
export XDG_CURRENT_DESKTOP="KDE"
export XDG_SESSION_TYPE=x11
dbus-launch gnome-session &
```