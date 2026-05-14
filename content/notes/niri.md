---
title: Debian KDE 并行安装 niri 桌面完整指南
slug: niri
summary: 在不卸载 KDE 的前提下，在 Debian 上安装、配置、使用并美化 niri，同时保证随时可以回滚到安装前状态。
category: []
tags: []
status: published
updated: 2026-04-27T18:58
aliases: []
---
# Debian KDE 并行安装 niri 桌面完整指南

> 目标：在不卸载 KDE 的前提下，在 Debian 上安装、配置、使用并美化 niri，同时保证随时可以回滚到安装前状态。  
> 适用对象：已有 KDE/SDDM，准备并行添加 niri 会话的 Debian 用户。  
> 本文以用户 `pcsensor` 为例，路径中出现 `/home/pcsensor` 的地方请按自己的用户名替换。你现在就这一个普通用户，所以先不搞“通用到谁都看不懂”的抽象模板，人类已经受够了。

---

## 0. 总体原则

1. **不卸 KDE，只安装 niri 并行测试。**
2. **安装前做 Timeshift 手动快照。**
3. **Back In Time 再备份 `/root` 和 `/home/pcsensor`。**
4. **niri 先用 Waybar + Mako + Fuzzel + Swaybg + Swayidle + Hyprlock。**
5. **所有 niri 相关自启动尽量交给 systemd 用户服务，不要在 `config.kdl` 里到处 `spawn-at-startup`。**
6. **翻车时优先 Timeshift 恢复系统，再用 Back In Time 恢复用户目录。**
7. **KDE 继续保留，作为备用救生艇。别把救生艇拆了做装饰，这种行为在人类历史里通常不吉利。**

---

## 1. 备份确认

### 1.1 记录当前系统状态

先把当前机器状态记录下来。这样以后回滚时能确认系统到底改过什么，而不是靠玄学回忆。

```bash
mkdir -p ~/pre-niri-state

cat /etc/os-release | tee ~/pre-niri-state/os-release.txt
uname -a | tee ~/pre-niri-state/uname.txt

apt-mark showmanual | sort > ~/pre-niri-state/apt-manual.txt
dpkg -l > ~/pre-niri-state/dpkg-list.txt

systemctl list-unit-files > ~/pre-niri-state/system-units.txt
systemctl --user list-unit-files > ~/pre-niri-state/user-units.txt

sudo cp -a /etc/apt ~/pre-niri-state/etc-apt-copy 2>/dev/null || true
sudo cp -a /etc/sddm.conf.d ~/pre-niri-state/sddm-conf-copy 2>/dev/null || true
```

### 1.2 创建 Timeshift 手动快照

```bash
sudo timeshift --create --comments "PRE-NIRI clean KDE state $(date -Iseconds)" --tags O
sudo timeshift --list
```

Timeshift 负责系统层面恢复。用户目录你已经用 Back In Time 备份，这个分工是合理的。

### 1.3 创建 Back In Time 快照

用 Back In Time GUI 确认：

- `/root` 已经有最新快照。
- `/home/pcsensor` 已经有最新快照。
- 能浏览快照内容。

Back In Time 保存的是普通文件备份，但权限和元数据恢复最好仍然通过 Back In Time 自己完成。手动复制当然也行，只是人类手动复制权限这件事，成功率不总是配得上自信。

---

## 2. 确认 Debian 版本

```bash
source /etc/os-release
echo "$PRETTY_NAME"
echo "$VERSION_CODENAME"
```

按结果选择方案：

| 结果 | 建议 |
|---|---|
| `trixie` | 推荐，按 Debian 13 方案走 |
| `forky` | 可用 Debian Testing 方案 |
| `sid` / `unstable` | 可用 Debian Unstable 方案 |
| `bookworm` | 不建议硬上最新版 niri，优先升级到 Trixie 或先在虚拟机测试 |

---

## 3. 添加 DankLinux 仓库并安装基础组件

DankLinux OBS 仓库目前是 Debian 上安装 niri 比较省心的方案之一。先准备 keyring：

```bash
sudo install -d -m 0755 /etc/apt/keyrings
source /etc/os-release

case "$VERSION_CODENAME" in
  trixie)
    DANK_DIST="Debian_13"
    ;;
  forky)
    DANK_DIST="Debian_Testing"
    ;;
  sid|unstable)
    DANK_DIST="Debian_Unstable"
    ;;
  *)
    echo "当前版本是 $VERSION_CODENAME，不建议直接按此方案安装最新版 niri"
    exit 1
    ;;
esac

echo "$DANK_DIST"
```

添加仓库：

```bash
curl -fsSL "https://download.opensuse.org/repositories/home:AvengeMedia:danklinux/${DANK_DIST}/Release.key" \
  | sudo gpg --dearmor -o /etc/apt/keyrings/danklinux.gpg

echo "deb [signed-by=/etc/apt/keyrings/danklinux.gpg] https://download.opensuse.org/repositories/home:/AvengeMedia:/danklinux/${DANK_DIST}/ /" \
  | sudo tee /etc/apt/sources.list.d/danklinux.list

sudo apt update
```

### 3.1 安装 niri 和基础桌面组件

这里已经把 **hyprlock** 加入一开始的安装命令。hyprlock 用的是现代 Wayland session lock 思路，比你之前折腾的 `swaylock-effects` 更适合 niri。`swaylock-effects` 那种旧 input inhibitor 协议路线就别继续养了，像给新车装马鞍。

```bash
sudo apt install \
  niri \
  xwayland-satellite \
  waybar \
  fuzzel \
  mako-notifier \
  swaybg \
  swayidle \
  hyprlock \
  grim \
  slurp \
  wl-clipboard \
  cliphist \
  brightnessctl \
  playerctl \
  pavucontrol \
  network-manager-gnome \
  blueman \
  polkit-kde-agent-1 \
  fonts-font-awesome \
  fonts-jetbrains-mono
```

如果 `hyprlock` 在当前源里找不到，先查：

```bash
apt search '^hyprlock$'
```

如果还是没有，可以暂时先不安装 hyprlock，等 niri 基础环境起来后再用 backports、sid 包、第三方源或源码方式安装。但安装主流程里推荐保留 `hyprlock`，因为本文锁屏配置默认使用它。

### 3.2 安装中文输入法组件

如果 KDE 已经装过 Fcitx5，可以跳过安装，只做后面的 niri 自启动和环境变量配置。稳妥起见可以执行：

```bash
sudo apt install \
  fcitx5 \
  fcitx5-chinese-addons \
  fcitx5-config-qt \
  fcitx5-frontend-gtk3 \
  fcitx5-frontend-gtk4 \
  fcitx5-frontend-qt5 \
  im-config
```

如果某个包名不存在，删掉那个包名再执行。Debian 包名偶尔像考古碎片，拼起来才能看懂。

---

## 4. 第一次进入 niri

先重启：

```bash
sudo reboot
```

在 SDDM 登录界面选择：

```text
Session / 会话 → Niri
```

进入后先检查显示器和窗口状态：

```bash
niri msg outputs
niri msg windows
```

检查 XWayland：

```bash
journalctl --user-unit=niri -b | grep -i x11
journalctl --user-unit=niri -b | grep -i xwayland
```

如果看到类似：

```text
listening on X11 socket
```

说明 `xwayland-satellite` 基本正常。

退出 niri：

```text
Super + Shift + E
```

---

## 5. niri 配置原则

配置文件位置：

```bash
~/.config/niri/config.kdl
```

第一次进入 niri 后先备份默认配置：

```bash
mkdir -p ~/.config/niri/backups
cp ~/.config/niri/config.kdl ~/.config/niri/backups/config.kdl.clean
```

以后每次改完都检查：

```bash
niri validate
```

niri 支持配置热重载，但遇到锁屏、自启动、环境变量这种东西，重登会话或重启更干净。桌面配置不是玄学，但很喜欢装成玄学。

---

## 6. niri 基础美化配置

编辑配置：

```bash
nano ~/.config/niri/config.kdl
```

### 6.1 圆角、阴影、边距

在合适位置添加或修改：

```kdl
prefer-no-csd

layout {
    gaps 12

    center-focused-column "on-overflow"

    default-column-width {
        proportion 0.5
    }

    focus-ring {
        width 3
        active-color "#89b4fa"
        inactive-color "#45475a"
    }

    border {
        off
    }

    shadow {
        on
        softness 30
        spread 5
        offset x=0 y=5
        color "#0007"
    }
}

window-rule {
    geometry-corner-radius 12
    clip-to-geometry true
}
```

### 6.2 输出配置

先查询真实输出名：

```bash
niri msg outputs
```

然后按你的机器写，例如：

```kdl
output "eDP-1" {
    mode "2560x1600@120.017"
    scale 1.5
    position x=0 y=0
    focus-at-startup
}
```

注意：`scale 1.5` 会影响整个桌面的缩放。如果某个应用的 `.desktop` 里又写了 `QT_SCALE_FACTOR=1.5`，就可能叠加成接近 2.25 倍。微信这种东西就很擅长把缩放玩成事故。

### 6.3 隐藏开机热键提示

```kdl
hotkey-overlay {
    skip-at-startup
}
```

### 6.4 截图路径

```kdl
screenshot-path "~/Pictures/Screenshots/Screenshot from %Y-%m-%d %H-%M-%S.png"
```

### 6.5 不要在 niri 里启动 Waybar

Waybar 后面会用 systemd 用户服务启动，所以确保这行被注释：

```kdl
// spawn-at-startup "waybar"
```

如果你同时用 `spawn-at-startup "waybar"` 和 `niri-waybar.service`，就会出现两个甚至三个 Waybar。一个状态栏已经够丑了，三个只是把丑做成阵列。

### 6.6 锁屏快捷键改为 Hyprlock

找到类似这行：

```kdl
Super+Alt+L hotkey-overlay-title="Lock the Screen: swaylock" { spawn "swaylock"; }
```

改成：

```kdl
Super+Alt+L hotkey-overlay-title="Lock the Screen" { spawn-sh "if ! pgrep -x hyprlock >/dev/null; then hyprlock & fi; sleep 30; niri msg action power-off-monitors"; }
```

检查：

```bash
niri validate
```

---

## 7. Hyprlock 锁屏配置

### 7.1 为什么不用 swaylock-effects

你之前源码构建的 `swaylock-effects` 能解析 `clock`、`timestr` 这类配置，但在 niri 里报：

```text
Compositor does not support the input inhibitor protocol, refusing to run insecurely
```

这说明它走的锁屏协议和 niri 不匹配。hyprlock 支持 `ext-session-lock`，支持分数缩放，能做壁纸、模糊、时间、输入框，正好适合 niri 这种现代 Wayland 桌面。继续折腾 `swaylock-effects` 不是坚韧，是对错误路线的忠诚。

### 7.2 创建壁纸目录

```bash
mkdir -p ~/.config/backgrounds
```

准备一张壁纸，例如：

```bash
cp ~/Pictures/Wallpapers/niri.jpg ~/.config/backgrounds/wallpaper.jpg
```

如果你的壁纸是 PNG：

```bash
cp ~/Pictures/Wallpapers/niri.png ~/.config/backgrounds/wallpaper.png
```

下面配置默认使用：

```text
/home/pcsensor/.config/backgrounds/wallpaper.jpg
```

### 7.3 写入 hyprlock 配置

```bash
mkdir -p ~/.config/hypr

cat > ~/.config/hypr/hyprlock.conf <<'EOF'
background {
    monitor =
    path = /home/pcsensor/.config/backgrounds/wallpaper.jpg
    blur_passes = 2
    blur_size = 6
    noise = 0.0117
    contrast = 0.95
    brightness = 0.72
    vibrancy = 0.20
}

label {
    monitor =
    text = cmd[update:1000] date +"%H:%M"
    color = rgba(205, 214, 244, 1.0)
    font_size = 88
    font_family = Maple Mono NF CN
    position = 0, 80
    halign = center
    valign = center
}

label {
    monitor =
    text = cmd[update:60000] date +"%Y-%m-%d  %A"
    color = rgba(186, 194, 222, 0.95)
    font_size = 22
    font_family = Maple Mono NF CN
    position = 0, -8
    halign = center
    valign = center
}

input-field {
    monitor =
    size = 280, 58
    outline_thickness = 3
    dots_size = 0.25
    dots_spacing = 0.35
    dots_center = true

    outer_color = rgba(137, 180, 250, 1.0)
    inner_color = rgba(30, 30, 46, 0.62)
    font_color = rgba(205, 214, 244, 1.0)
    check_color = rgba(166, 227, 161, 1.0)
    fail_color = rgba(243, 139, 168, 1.0)

    fade_on_empty = false
    placeholder_text = <span foreground="##cdd6f4">Password</span>
    fail_text = <span foreground="##f38ba8">Wrong password</span>

    position = 0, -115
    halign = center
    valign = center
}
EOF
```

测试：

```bash
hyprlock
```

如果能锁住并能解锁，说明配置可用。如果提示找不到图片，检查：

```bash
ls -lh ~/.config/backgrounds/wallpaper.jpg
```

---

## 8. Waybar 美化

### 8.1 写入 Waybar 配置

```bash
mkdir -p ~/.config/waybar
nano ~/.config/waybar/config.jsonc
```

写入：

```jsonc
{
  "exclusive": true,
  "reload_style_on_change": true,
  "layer": "top",
  "position": "top",
  "height": 34,
  "spacing": 0,
  "margin-top": 6,
  "margin-left": 8,
  "margin-right": 8,

  "modules-left": [
    "niri/workspaces",
    "niri/window"
  ],

  "modules-center": [
    "clock",
    "mpris"
  ],

  "modules-right": [
    "tray",
    "backlight",
    "network",
    "bluetooth",
    "pulseaudio#output",
    "pulseaudio#input",
    "memory",
    "cpu",
    "battery"
  ],

  "niri/workspaces": {
    "format": "{icon}",
    "all-outputs": false,
    "format-icons": {
      "active": "",
      "focused": "",
      "empty": "",
      "urgent": "",
      "default": ""
    }
  },

  "niri/window": {
    "format": "󰣇  {title}",
    "max-length": 55,
    "separate-outputs": true,
    "rewrite": {
      "(.*) - Mozilla Firefox": "󰈹  $1",
      "(.*) - Chromium": "  $1",
      "(.*) - Google Chrome": "  $1",
      "(.*) - zsh": "  zsh",
      "(.*) - fish": "  fish",
      "(.*) - WezTerm": "  WezTerm"
    }
  },

  "clock": {
    "format": "  {:%H:%M}",
    "format-alt": "  {:%Y-%m-%d  %A}",
    "tooltip-format": "{calendar}",
    "calendar": {
      "mode": "month",
      "mode-mon-col": 3,
      "format": {
        "today": "<b>{}</b>"
      }
    }
  },

  "mpris": {
    "format": "󰎈  {artist} - {title}",
    "format-paused": "󰏤  {artist} - {title}",
    "max-length": 34,
    "ignored-players": ["firefox", "chromium", "brave"],
    "tooltip-format": "{player}: {dynamic}",
    "on-click": "playerctl play-pause",
    "on-scroll-up": "playerctl previous",
    "on-scroll-down": "playerctl next"
  },

  "tray": {
    "icon-size": 14,
    "spacing": 8,
    "show-passive-items": true
  },

  "backlight": {
    "format": "{icon}  {percent}%",
    "format-icons": ["", "", "", "", "", "", "", "", "", "", "", "", "", "", ""]
  },

  "network": {
    "interval": 3,
    "format-wifi": "󰤨  {signalStrength}%",
    "format-ethernet": "󰈀  wired",
    "format-disconnected": "󰤮",
    "tooltip-format-wifi": "{essid}\n⇣ {bandwidthDownBytes}  ⇡ {bandwidthUpBytes}",
    "tooltip-format-ethernet": "⇣ {bandwidthDownBytes}  ⇡ {bandwidthUpBytes}",
    "tooltip-format-disconnected": "Disconnected"
  },

  "bluetooth": {
    "format": "󰂯",
    "format-disabled": "󰂲",
    "format-connected": "󰂱  {num_connections}",
    "tooltip-format": "Bluetooth: {status}\nConnected: {num_connections}"
  },

  "pulseaudio#output": {
    "format": "{icon}  {volume}%",
    "format-muted": "󰝟  muted",
    "format-icons": {
      "headphone": "",
      "headset": "",
      "default": ["", "", ""]
    },
    "scroll-step": 2,
    "on-click": "pavucontrol"
  },

  "pulseaudio#input": {
    "format-source": "  {volume}%",
    "format-source-muted": "  muted",
    "scroll-step": 2,
    "on-click": "pavucontrol"
  },

  "memory": {
    "interval": 3,
    "format": "  {percentage}%"
  },

  "cpu": {
    "interval": 3,
    "format": "  {usage}%"
  },

  "battery": {
    "interval": 5,
    "format": "{icon}  {capacity}%",
    "format-charging": "󰂄  {capacity}%",
    "format-plugged": "  {capacity}%",
    "format-full": "󰁹  {capacity}%",
    "format-icons": ["󰁺", "󰁻", "󰁼", "󰁽", "󰁾", "󰁿", "󰂀", "󰂁", "󰂂", "󰁹"],
    "states": {
      "warning": 25,
      "critical": 10
    }
  }
}
```

让 Waybar 默认读它：

```bash
ln -sf ~/.config/waybar/config.jsonc ~/.config/waybar/config
```

### 8.2 写入 CSS

```bash
nano ~/.config/waybar/style.css
```

写入：

```css
@define-color bg rgba(22, 22, 22, 0.86);
@define-color bg2 rgba(32, 32, 36, 0.92);
@define-color fg #f2f4f8;
@define-color muted #8d8d8d;
@define-color blue #33b1ff;
@define-color pink #ee5396;
@define-color green #42be65;
@define-color purple #be95ff;
@define-color cyan #08bdba;
@define-color red #ff7eb6;
@define-color border rgba(138, 138, 141, 0.85);
@define-color border_dim rgba(54, 54, 54, 0.9);

* {
  border: none;
  border-radius: 0;
  min-height: 0;
  padding: 0;
  font-family: "Maple Mono NF CN", "JetBrainsMono Nerd Font", "JetBrains Mono", "Font Awesome 6 Free", sans-serif;
  font-size: 12px;
  font-weight: 700;
}

window#waybar {
  background: transparent;
  color: @fg;
}

window#waybar.empty #window {
  opacity: 0.35;
}

.modules-left,
.modules-center,
.modules-right {
  background: transparent;
}

#workspaces,
#window,
#clock,
#mpris,
#tray,
#backlight,
#network,
#bluetooth,
#pulseaudio,
#memory,
#cpu,
#battery {
  background: @bg;
  color: @fg;
  margin: 2px 4px;
  padding: 0 14px;
  border-radius: 14px;
  border: 2px solid @border;
  box-shadow: 0 2px 7px rgba(0, 0, 0, 0.55);
}

#workspaces {
  padding: 0 8px;
}

#workspaces button {
  all: initial;
  color: @muted;
  padding: 0 8px;
  margin: 0 2px;
  font-family: "Maple Mono NF CN", "JetBrainsMono Nerd Font", sans-serif;
  font-size: 13px;
  font-weight: 900;
}

#workspaces button.focused {
  color: @pink;
}

#workspaces button.active {
  color: @blue;
}

#workspaces button.empty {
  color: @muted;
  opacity: 0.35;
}

#workspaces button.urgent {
  color: @red;
}

#window {
  color: @fg;
  min-width: 160px;
}

#clock {
  color: @fg;
  border-color: @border;
}

#mpris {
  color: @purple;
}

#tray {
  padding: 0 10px;
}

#backlight {
  color: @green;
}

#network,
#bluetooth {
  color: @blue;
}

#pulseaudio.output,
#pulseaudio {
  color: @red;
}

#memory {
  color: @cyan;
}

#cpu {
  color: @pink;
}

#battery {
  color: @blue;
}

#battery.warning {
  color: #f1c21b;
}

#battery.critical:not(.charging) {
  color: #ff5555;
  border-color: #ff5555;
  animation-name: blink-critical;
  animation-duration: 0.7s;
  animation-timing-function: steps(12);
  animation-iteration-count: infinite;
  animation-direction: alternate;
}

@keyframes blink-critical {
  to {
    color: @bg;
    background: #ff5555;
  }
}

tooltip {
  background: @bg2;
  color: @fg;
  border: 2px solid @border;
  border-radius: 12px;
  padding: 8px;
}

tooltip label {
  color: @fg;
}
```

### 8.3 图标乱码处理

如果 Waybar 图标显示方块，安装 Nerd Font 或确认 Maple Mono NF CN 真的是 Nerd Font 版。

```bash
sudo apt install fonts-font-awesome fonts-jetbrains-mono
```

---

## 9. Fuzzel 启动器美化

```bash
mkdir -p ~/.config/fuzzel
nano ~/.config/fuzzel/fuzzel.ini
```

写入：

```ini
[main]
font=Maple Mono NF CN:size=13
terminal=wezterm
width=44
lines=14
horizontal-pad=16
vertical-pad=12
inner-pad=8
layer=overlay

[colors]
background=1e1e2edd
text=cdd6f4ff
match=89b4faff
selection=313244ff
selection-text=cdd6f4ff
border=89b4faff

[border]
width=2
radius=14
```

如果你实际终端不是 WezTerm，把：

```ini
terminal=wezterm
```

改成：

```ini
terminal=alacritty
```

或：

```ini
terminal=konsole
```

---

## 10. Mako 通知美化

```bash
mkdir -p ~/.config/mako
nano ~/.config/mako/config
```

写入：

```ini
font=Maple Mono NF CN 12
background-color=#1e1e2edd
text-color=#cdd6f4ff
border-color=#89b4faff
progress-color=over #313244ff
border-size=2
border-radius=12
padding=12
margin=12
default-timeout=5000
max-visible=5
anchor=top-right
```

---

## 11. systemd 用户服务

创建目录：

```bash
mkdir -p ~/.config/systemd/user
```

### 11.1 Waybar 服务

```bash
cat > ~/.config/systemd/user/niri-waybar.service <<'EOF'
[Unit]
Description=Waybar for niri
PartOf=graphical-session.target
After=graphical-session.target
Requisite=graphical-session.target

[Service]
ExecStart=/usr/bin/waybar
Restart=on-failure

[Install]
WantedBy=niri.service
EOF
```

### 11.2 Mako 服务

```bash
cat > ~/.config/systemd/user/niri-mako.service <<'EOF'
[Unit]
Description=Mako notifications for niri
PartOf=graphical-session.target
After=graphical-session.target
Requisite=graphical-session.target

[Service]
ExecStart=/usr/bin/mako
Restart=on-failure

[Install]
WantedBy=niri.service
EOF
```

### 11.3 壁纸服务

建议统一使用 `~/.config/backgrounds/wallpaper.jpg`，这样 hyprlock 和 swaybg 用同一张壁纸。

```bash
cat > ~/.config/systemd/user/niri-swaybg.service <<'EOF'
[Unit]
Description=Wallpaper for niri
PartOf=graphical-session.target
After=graphical-session.target
Requisite=graphical-session.target

[Service]
ExecStart=/usr/bin/swaybg -m fill -i %h/.config/backgrounds/wallpaper.jpg
Restart=on-failure

[Install]
WantedBy=niri.service
EOF
```

### 11.4 Swayidle 空闲锁屏服务

使用 hyprlock 作为锁屏器：

```bash
cat > ~/.config/systemd/user/niri-swayidle.service <<'EOF'
[Unit]
Description=Idle management for niri
PartOf=graphical-session.target
After=graphical-session.target
Requisite=graphical-session.target

[Service]
ExecStart=/usr/bin/swayidle -w timeout 600 'pgrep -x hyprlock >/dev/null || hyprlock' timeout 660 'niri msg action power-off-monitors' before-sleep 'pgrep -x hyprlock >/dev/null || hyprlock --immediate'
Restart=on-failure

[Install]
WantedBy=niri.service
EOF
```

说明：

- 600 秒无操作后启动 hyprlock。
- 660 秒后关闭显示器。
- 睡眠前执行 hyprlock。
- `pgrep -x hyprlock` 用来避免重复启动多个 hyprlock。
- 如果你的 hyprlock 不支持 `--immediate`，把 `--immediate` 删掉即可。

### 11.5 Fcitx5 输入法服务

如果你确认 XDG autostart 已经自动启动 Fcitx5，可以不建这个服务。否则创建：

```bash
cat > ~/.config/systemd/user/niri-fcitx5.service <<'EOF'
[Unit]
Description=Fcitx5 input method for niri
PartOf=graphical-session.target
After=graphical-session.target
Requisite=graphical-session.target

[Service]
ExecStart=/usr/bin/fcitx5
Restart=on-failure

[Install]
WantedBy=niri.service
EOF
```

### 11.6 Polkit 自启动服务

测试 KDE 的 Polkit agent 是否存在

```bash
ls /usr/lib/x86_64-linux-gnu/libexec/polkit-kde-authentication-agent-1
```

#### 如果存在, 创建服务：

```bash
cat > ~/.config/systemd/user/niri-polkit.service <<'EOF'
[Unit]
Description=KDE Polkit Authentication Agent for niri
PartOf=graphical-session.target
After=graphical-session.target
Requisite=graphical-session.target

[Service]
ExecStart=/usr/lib/x86_64-linux-gnu/libexec/polkit-kde-authentication-agent-1
Restart=on-failure

[Install]
WantedBy=niri.service
EOF
```

#### 如果不存在

安装一个轻量 Polkit agent：

```bash
sudo apt install lxqt-policykit
```

确认路径：

```bash
which lxqt-policykit-agent
```

然后创建服务：

```bash
cat > ~/.config/systemd/user/niri-polkit.service <<'EOF'
[Unit]
Description=LXQt Polkit Authentication Agent for niri
PartOf=graphical-session.target
After=graphical-session.target
Requisite=graphical-session.target

[Service]
ExecStart=/usr/bin/lxqt-policykit-agent
Restart=on-failure

[Install]
WantedBy=niri.service
EOF
```

启用所有服务：

```bash
systemctl --user daemon-reload
systemctl --user enable niri-waybar.service
systemctl --user enable niri-mako.service
systemctl --user enable niri-swaybg.service
systemctl --user enable niri-swayidle.service
systemctl --user enable niri-fcitx5.service
systemctl --user enable niri-polkit.service
```

当前会话立即启动：

```bash
systemctl --user restart niri-waybar.service
systemctl --user restart niri-mako.service
systemctl --user restart niri-swaybg.service
systemctl --user restart niri-swayidle.service
systemctl --user restart niri-fcitx5.service
systemctl --user restart niri-polkit.service
```

检查：

```bash
systemctl --user status niri-waybar.service --no-pager
systemctl --user status niri-mako.service --no-pager
systemctl --user status niri-swaybg.service --no-pager
systemctl --user status niri-swayidle.service --no-pager
systemctl --user status niri-fcitx5.service --no-pager
```

### 11.7 Backintime-root的niri root启动脚本

```bash
mkdir -p ~/.local/bin

cat > ~/.local/bin/backintime-root-niri <<'EOF'
#!/usr/bin/env bash

set -e

xhost +SI:localuser:root >/dev/null

cleanup() {
    xhost -SI:localuser:root >/dev/null 2>&1 || true
}
trap cleanup EXIT

pkexec env \
    HOME=/root \
    USER=root \
    LOGNAME=root \
    XDG_CONFIG_HOME=/root/.config \
    QT_QPA_PLATFORM=xcb \
    DISPLAY="$DISPLAY" \
    backintime-qt
EOF

chmod +x ~/.local/bin/backintime-root-niri
```

fuzzel 启动项也改成这个脚本:

```bash
mkdir -p ~/.local/share/applications

cat > ~/.local/share/applications/backintime-root-niri.desktop <<'EOF'
[Desktop Entry]
Type=Application
Name=Back In Time (root, niri)
Name[zh_CN]=Back In Time（root，niri）
Comment=Run Back In Time as root under niri
Exec=/home/pcsensor/.local/bin/backintime-root-niri
Icon=document-save
Terminal=false
Categories=System;Utility;
EOF

update-desktop-database ~/.local/share/applications 2>/dev/null || true
```

---

## 12. 处理常见自启动冲突

### 12.1 Waybar 出现两个或三个

检查进程来源：

```bash
for pid in $(pgrep -x waybar); do
  echo "===== PID $pid ====="
  ps -p "$pid" -o pid,ppid,cmd
  cat /proc/$pid/cgroup
  echo
done
```

如果看到：

```text
niri-waybar.service
waybar.service
```

保留 `niri-waybar.service`，禁用通用的 `waybar.service`：

```bash
systemctl --user disable --now waybar.service
systemctl --user mask waybar.service
```

如果提示 global scope 仍启用，mask 对当前用户已经够用。想全局关闭再执行：

```bash
sudo systemctl --global disable waybar.service
```

### 12.2 禁用 XWayland Video Bridge 空白窗口

KDE 的 `xwaylandvideobridge` 可能在 niri 里弹出空白窗口。先查真实文件名：

```bash
ls /etc/xdg/autostart | grep -iE "xwayland|video|bridge"
```

如果看到：

```text
org.kde.xwaylandvideobridge.desktop
```

创建用户级屏蔽：

```bash
mkdir -p ~/.config/autostart

cat > ~/.config/autostart/org.kde.xwaylandvideobridge.desktop <<'EOF'
[Desktop Entry]
Type=Application
Name=XWayland Video Bridge
Hidden=true
NoDisplay=true
X-GNOME-Autostart-enabled=false
EOF
```

重登 niri 后确认：

```bash
pgrep -a xwaylandvideobridge
```

没有输出就是关闭了。

---

## 13. 中文输入法 Fcitx5

### 13.1 配置环境变量

创建：

```bash
mkdir -p ~/.config/environment.d

cat > ~/.config/environment.d/fcitx5.conf <<'EOF'
XMODIFIERS=@im=fcitx
QT_IM_MODULE=fcitx
QT_IM_MODULES=wayland;fcitx
SDL_IM_MODULE=fcitx
GLFW_IM_MODULE=ibus
EOF
```

先不要全局添加 `GTK_IM_MODULE=fcitx`。如果 GTK 应用无法输入中文，再追加：

```bash
echo 'GTK_IM_MODULE=fcitx' >> ~/.config/environment.d/fcitx5.conf
```

修改环境变量后建议重启：

```bash
reboot
```

### 13.2 打开 Fcitx5 配置工具

```bash
fcitx5-configtool
```

添加：

- `Keyboard - English`
- `Pinyin`

常见切换键：

```text
Ctrl + Space
```

或按你在 Fcitx5 里配置的切换键。

### 13.3 检查 Fcitx5 状态

```bash
pgrep -a fcitx5
env | grep -E 'XMODIFIERS|IM_MODULE|GLFW'
fcitx5-diagnose
```

如果 Fcitx5 启动了两次，查自启动来源：

```bash
grep -Rni "fcitx" \
  ~/.config/autostart \
  /etc/xdg/autostart \
  ~/.config/systemd/user \
  /etc/systemd/user \
  /usr/lib/systemd/user \
  2>/dev/null
```

最终只保留一种启动方式。输入法启动两遍不会让中文更中文，只会让系统更像人格分裂。

---

## 14. 微信缩放处理

niri 的输出配置里如果有：

```kdl
scale 1.5
```

同时微信 `.desktop` 里又有：

```ini
Exec=env QT_SCALE_FACTOR=1.5 /usr/bin/wechat %U
```

就很可能叠加成接近 2.25 倍。

正确做法是不要改 `/usr/share/applications/wechat.desktop`，而是复制到用户目录覆盖：

```bash
mkdir -p ~/.local/share/applications
cp /usr/share/applications/wechat.desktop ~/.local/share/applications/wechat.desktop
nano ~/.local/share/applications/wechat.desktop
```

把：

```ini
Exec=env QT_SCALE_FACTOR=1.5 /usr/bin/wechat %U
```

改成：

```ini
Exec=env QT_SCALE_FACTOR=1 /usr/bin/wechat %U
```

刷新：

```bash
update-desktop-database ~/.local/share/applications 2>/dev/null || true
pkill -f wechat
```

然后重新用 fuzzel 启动微信。

---

## 15. 多显示器配置

查询输出：

```bash
niri msg outputs
```

示例：

```kdl
output "HDMI-A-1" {
    mode "1920x1080@60"
    scale 1
    position x=0 y=0
    focus-at-startup
}

output "DP-1" {
    mode "2560x1440@144"
    scale 1
    position x=1920 y=0
}
```

注意输出位置使用逻辑像素。比如 3840×2160 且 scale 2 的屏幕，逻辑宽度是 1920。

---

## 16. 常用快捷键

你当前默认逻辑大致是：

| 快捷键 | 作用 |
|---|---|
| `Super + T` | 打开 WezTerm |
| `Super + D` | 打开 Fuzzel |
| `Super + Alt + L` | 锁屏，运行 Hyprlock |
| `Super + Q` | 关闭窗口 |
| `Super + O` | Overview 总览 |
| `Super + H / L` | 左右切列 |
| `Super + J / K` | 当前列内上下切窗口 |
| `Super + U / I` | 上下切 workspace |
| `Super + PageUp / PageDown` | 上下切 workspace |
| `Super + 1~9` | 切到指定 workspace |
| `Super + Ctrl + 1~9` | 把当前列移动到指定 workspace |
| `Super + F` | 最大化当前列 |
| `Super + Shift + F` | 当前窗口全屏 |
| `Super + V` | 浮动 / 平铺切换 |
| `Super + Shift + E` | 退出 niri |
| `Print` | 截图 |

如果你想让 `Super + 上/下` 切 workspace，把：

```kdl
Mod+Down  { focus-window-down; }
Mod+Up    { focus-window-up; }
```

改成：

```kdl
Mod+Down  { focus-workspace-down; }
Mod+Up    { focus-workspace-up; }
```

但这样会失去当前列内上下切窗口的默认逻辑。别一边改掉它，一边下一分钟问它为什么不工作，人类配置桌面的经典循环剧目已经够多了。

---

## 17. 更激进的美化：DMS / Noctalia

### 17.1 DMS

DMS 是完整 Wayland shell，能替代 Waybar、Mako、Fuzzel、锁屏等组件。它更完整也更重，适合作为第二阶段。

如果已经添加 DankLinux，可以继续按官方 DMS 文档添加 DMS 源并安装。安装后让它只跟 niri 会话绑定，避免污染 KDE：

```bash
systemctl --user add-wants niri.service dms
```

DMS 对 niri 的 include 示例：

```kdl
include "dms/colors.kdl"
include "dms/layout.kdl"
include "dms/alttab.kdl"
include "dms/binds.kdl"
```

### 17.2 Noctalia

Noctalia 很漂亮，但对已有 KDE 的 Debian 桌面来说更激进。它牵涉 Quickshell/Noctalia-QS 等组件，建议等 niri 基础桌面稳定几天后再试。

推荐顺序：

```text
第一阶段：niri + Waybar + Mako + Fuzzel + Swaybg + Swayidle + Hyprlock
第二阶段：DMS
第三阶段：Noctalia
```

---

## 18. 回滚方案

### 18.1 轻度回滚

```bash
systemctl --user disable --now niri-waybar.service niri-mako.service niri-swaybg.service niri-swayidle.service niri-fcitx5.service 2>/dev/null || true
systemctl --user daemon-reload

sudo apt purge niri xwayland-satellite waybar fuzzel mako-notifier swaybg swayidle hyprlock
sudo apt autoremove --purge

rm -rf ~/.config/niri
rm -rf ~/.config/waybar
rm -rf ~/.config/fuzzel
rm -rf ~/.config/mako
rm -rf ~/.config/hypr
rm -rf ~/.config/backgrounds
rm -f ~/.config/systemd/user/niri-*.service
```

如果你只想回滚 niri，不想影响 KDE 输入法，不要删 `~/.config/fcitx5`。

### 18.2 删除第三方源

```bash
sudo rm -f /etc/apt/sources.list.d/danklinux.list
sudo rm -f /etc/apt/keyrings/danklinux.gpg
sudo apt update
```

如果安装了 DMS，还要删除 DMS 源：

```bash
sudo rm -f /etc/apt/sources.list.d/avengemedia-dms.list
sudo rm -f /etc/apt/keyrings/avengemedia-dms.gpg
sudo apt update
```

### 18.3 严格回滚

1. 用 KDE 或 Live USB 打开 Timeshift。
2. 恢复 `PRE-NIRI clean KDE state ...` 快照。
3. 重启。
4. 用 Back In Time 恢复 `/root` 和 `/home/pcsensor`。

如果目标是“不留一点残留”，以 Timeshift + Back In Time 为准。轻度回滚只是清理包和配置，不是法医级时光倒流。

---

## 19. 最终执行顺序

```text
1. 记录系统状态
2. Timeshift 创建手动快照
3. Back In Time 备份 /root 和 /home/pcsensor
4. 添加 DankLinux 仓库
5. 安装 niri、xwayland-satellite、Waybar、Fuzzel、Mako、Swaybg、Swayidle、Hyprlock 等基础组件
6. SDDM 登录界面选择 Niri
7. 验证 niri msg outputs 和 journalctl
8. 备份 ~/.config/niri/config.kdl
9. 配置 niri 圆角、阴影、缩放、截图路径、锁屏快捷键
10. 配置 Hyprlock
11. 配置 Waybar / Fuzzel / Mako
12. 配置 systemd 用户服务
13. 配置 Fcitx5 中文输入法
14. 屏蔽多余 Waybar、xwaylandvideobridge 等 KDE 遗留自启动
15. 稳定使用几天
16. 再考虑 DMS 或 Noctalia
```

---

## 20. 参考资料

- Timeshift: https://github.com/linuxmint/timeshift
- Back In Time: https://backintime.readthedocs.io/
- Debian Releases: https://www.debian.org/releases/
- niri Releases: https://github.com/niri-wm/niri/releases
- niri Getting Started: https://niri-wm.github.io/niri/Getting-Started.html
- niri Integrating niri: https://niri-wm.github.io/niri/Integrating-niri.html
- niri XWayland: https://github.com/niri-wm/niri/wiki/Xwayland
- niri FAQ: https://github.com/niri-wm/niri/wiki/FAQ
- niri systemd example: https://github.com/niri-wm/niri/wiki/Example-systemd-Setup
- DankLinux Repository: https://danklinux.com/docs/danklinux/
- Waybar niri workspaces: https://man.archlinux.org/man/extra/waybar/waybar-niri-workspaces.5.en
- Hyprlock: https://github.com/hyprwm/hyprlock
- Hyprlock Wiki: https://wiki.hypr.land/Hypr-Ecosystem/hyprlock/
- Fcitx5 Setup: https://fcitx-im.org/wiki/Setup_Fcitx_5
- Fcitx5 on Wayland: https://fcitx-im.org/wiki/Using_Fcitx_5_on_Wayland
- DMS: https://github.com/AvengeMedia/DankMaterialShell
- Noctalia: https://docs.noctalia.dev/getting-started/installation/
