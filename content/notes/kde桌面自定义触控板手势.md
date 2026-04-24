---
title: kde桌面自定义触控板手势
slug: kde桌面自定义触控板手势
summary: 本文记录在 KDE Plasma 桌面中使用 `libinput-gestures` 和 `ydotool` 自定义触控板手势的操作步骤。
category: []
tags: []
status: published
updated: 2026-04-24T15:48
aliases: []
---
# KDE 桌面自定义触控板手势

本文记录在 KDE Plasma 桌面中使用 `libinput-gestures` 和 `ydotool` 自定义触控板手势的操作步骤。

## 背景

KDE Plasma 目前自带的触控板手势只有 4 指上滑等少量手势，并且不方便直接自定义。可以通过 `libinput-gestures` 监听触控板手势，再调用 `ydotool` 模拟快捷键，从而实现自定义操作。

## 安装 `libinput-gestures`

Arch 用户可以通过 AUR 安装：

```bash
paru -S libinput-gestures
```

其他发行版用户可以手动安装：

```bash
git clone https://github.com/bulletmark/libinput-gestures.git
cd libinput-gestures
sudo ./libinput-gestures-setup install
```

Debian 或 Ubuntu 用户可能还需要安装 `libinput-tools`：

```bash
sudo apt install libinput-tools
```

## 启动 `libinput-gestures`

将当前用户加入 `input` 用户组：

```bash
sudo gpasswd -a $USER input
newgrp input
```

添加自启动，并立即启动：

```bash
libinput-gestures-setup autostart start
```

## 安装并启动 `ydotool`

`libinput-gestures` 文档建议使用的键盘模拟器是 `xdotool`，但 `xdotool` 主要用于 Xorg，在较新的 KDE Wayland 会话中不一定适用。因此这里使用 `ydotool`。

Arch 用户安装：

```bash
sudo pacman -S ydotool
```

Debian 系发行版安装：

```bash
sudo apt install ydotool
```

启用用户级服务并立即启动：

```bash
systemctl --user enable --now ydotool.service
```

## 创建自定义配置

复制默认配置到用户配置目录：

```bash
cp /etc/libinput-gestures.conf ~/.config/
kate ~/.config/libinput-gestures.conf
```

## `libinput-gestures` 配置格式

配置文件按行定义手势。基本格式如下：

```text
gesture <action> [finger_count] <command>
```

其中：

- `action` 表示手势动作。
- `finger_count` 表示手指数，例如 `3` 表示三指手势。
- `command` 表示检测到手势后执行的命令，可以是任意命令，包括模拟键盘输入。

## 支持的手势动作

`libinput-gestures` 常用动作包括：

- `swipe up`
- `swipe down`
- `swipe left`
- `swipe right`
- `swipe left_up`
- `swipe left_down`
- `swipe right_up`
- `swipe right_down`
- `pinch in`
- `pinch out`
- `pinch clockwise`
- `pinch anticlockwise`
- `hold on`
- `hold on N`

## `ydotool` 按键说明

`ydotool key` 使用 Linux input keycode。常见按键代码可以在系统头文件中查看：

```text
/usr/include/linux/input-event-codes.h
```

其中：

- `keycode:1` 表示按住对应按键。
- `keycode:0` 表示释放对应按键。

例如模拟 `Ctrl + F12` 时，需要依次按下 `Ctrl` 和 `F12`，再依次释放它们：

```bash
ydotool key 29:1 88:1 88:0 29:0
```

## 参考配置

下面是一组可直接使用的示例配置：

```text
gesture swipe down 3 ydotool key 29:1 88:1 88:0 29:0    # 三指下滑回到桌面
gesture swipe up 3 ydotool key 29:1 68:1 68:0 29:0      # 三指上滑查看窗口
gesture swipe right 4 ydotool key 56:1 15:1 15:0 56:0   # 任务窗口切换
gesture swipe left 4 ydotool key 56:1 15:1 15:0 56:0    # 任务窗口切换
gesture pinch out ydotool key 29:1 13:1 13:0 29:0       # 放大，等同 Ctrl + =
gesture pinch in ydotool key 29:1 12:1 12:0 29:0        # 缩小，等同 Ctrl + -
```

## 让配置生效

修改配置文件后，重启 `libinput-gestures`：

```bash
libinput-gestures-setup stop desktop autostart start
```

如果手势没有生效，可以检查：

- 当前用户是否已经加入 `input` 用户组。
- 是否重新登录或执行过 `newgrp input`。
- `ydotool.service` 是否正在运行。
- 当前桌面会话是否为 Wayland。
- `~/.config/libinput-gestures.conf` 中的配置格式是否正确。
