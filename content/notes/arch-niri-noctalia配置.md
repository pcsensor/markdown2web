---
title: Arch_niri_noctalia配置
slug: arch-niri-noctalia配置
summary: 没啥用，越来越讨厌折腾了，还是回归macOS的怀抱吧
category: []
tags: []
status: published
updated: 2026-05-13T14:54
aliases: []
---
## 1. 安装所有依赖（官方仓库 + AUR）

```bash
# 更新系统
sudo pacman -Syu

# 安装基础构建工具（如果还没有 AUR helper，先装 paru）
sudo pacman -S --needed base-devel git
# 若未安装 paru，执行下面三行：
# git clone https://aur.archlinux.org/paru.git /tmp/paru
# cd /tmp/paru && makepkg -si

# 安装 niri 桌面基础组件 + Noctalia 系统依赖 + 字体
sudo pacman -S --needed \
  niri xwayland-satellite \
  xdg-desktop-portal xdg-desktop-portal-gtk xdg-desktop-portal-gnome \
  pipewire wireplumber polkit gnome-keyring \
  kitty fuzzel wl-clipboard cliphist \
  brightnessctl playerctl pavucontrol \
  networkmanager bluez bluez-utils upower \
  power-profiles-daemon wlsunset \
  qt6ct nwg-look papirus-icon-theme adwaita-icon-theme \
  nautilus ddcutil

# 安装字体（AUR）
paru -S --needed ttf-maple-mono-nf-cn

# 安装 Noctalia v5（AUR）
paru -S --needed noctalia-git
```

---

## 2. 启用系统服务

```bash
sudo systemctl enable --now NetworkManager
sudo systemctl enable --now bluetooth
sudo systemctl enable --now upower
```

---

## 3. 写入配置文件

### 3.1 Niri 配置

```bash
mkdir -p ~/.config/niri
cat > ~/.config/niri/config.kdl << 'EOF'
spawn-at-startup "noctalia"

environment {
    QT_QPA_PLATFORM "wayland;xcb"
    QT_QPA_PLATFORMTHEME "qt6ct"
    XDG_CURRENT_DESKTOP "niri"
}

input {
    keyboard {
        numlock
    }
    touchpad {
        tap
        natural-scroll
    }
}

output "eDP-1" {
	mode "2560x1600@120.017"
	scale 1.25
	position x=0 y=0
	transform "normal"
}

layout {
    gaps 16
    center-focused-column "never"
    preset-column-widths {
        proportion 0.33333
        proportion 0.5
        proportion 0.66667
    }
    default-column-width { proportion 0.5; }
    focus-ring {
        width 4
        active-color "#7fc8ff"
        inactive-color "#505050"
    }
    border {
        off
        width 4
        active-color "#ffc87f"
        inactive-color "#505050"
        urgent-color "#9b0000"
    }
    shadow {
        softness 30
        spread 5
        offset x=0 y=5
        color "#0007"
    }
}

screenshot-path "~/Pictures/Screenshots/Screenshot from %Y-%m-%d %H-%M-%S.png"

window-rule {
    geometry-corner-radius 20
    clip-to-geometry true
}

window-rule {
    match app-id="dev.noctalia.Noctalia.Settings"
    open-floating true
    default-column-width { fixed 1080; }
    default-window-height { fixed 920; }
}

window-rule {
    background-effect {
        blur true
        xray false
    }
}

layer-rule {
    match namespace="^noctalia-backdrop"
    place-within-backdrop true
}

layer-rule {
    match namespace="^noctalia-(bar-main|notification|dock|panel)$"
    background-effect {
        blur true
        xray false
    }
}

blur {
    passes 2
    offset 3.0
    noise 0.03
    saturation 1.0
}

debug {
    honor-xdg-activation-with-invalid-serial
}

binds {
    Mod+Shift+Slash { show-hotkey-overlay; }
    Mod+T { spawn "wezterm"; }
    Mod+D { spawn "fuzzel"; }
    Super+Alt+S allow-when-locked=true { spawn-sh "pkill orca || exec orca"; }
    XF86AudioPlay allow-when-locked=true { spawn-sh "playerctl play-pause"; }
    XF86AudioStop allow-when-locked=true { spawn-sh "playerctl stop"; }
    XF86AudioPrev allow-when-locked=true { spawn-sh "playerctl previous"; }
    XF86AudioNext allow-when-locked=true { spawn-sh "playerctl next"; }
    Mod+O repeat=false { toggle-overview; }
    Mod+Shift+E { spawn "noctalia" "msg" "panel-toggle" "session"; }
    Mod+L { spawn "noctalia" "msg" "screen-lock"; }
    Mod+B { spawn "noctalia" "msg" "bar-toggle"; }
    Mod+Shift+T { spawn "noctalia" "msg" "theme-mode-toggle"; }
    XF86AudioRaiseVolume allow-when-locked=true { spawn "noctalia" "msg" "volume-up"; }
    XF86AudioLowerVolume allow-when-locked=true { spawn "noctalia" "msg" "volume-down"; }
    XF86AudioMute allow-when-locked=true { spawn "noctalia" "msg" "volume-mute"; }
    XF86AudioMicMute allow-when-locked=true { spawn "noctalia" "msg" "mic-mute"; }
    XF86MonBrightnessUp allow-when-locked=true { spawn "noctalia" "msg" "brightness-up"; }
    XF86MonBrightnessDown allow-when-locked=true { spawn "noctalia" "msg" "brightness-down"; }
    Mod+Q repeat=false { close-window; }
    Mod+Left { focus-column-left; }
    Mod+Down { focus-window-down; }
    Mod+Up { focus-window-up; }
    Mod+Right { focus-column-right; }
    Mod+H { focus-column-left; }
    Mod+J { focus-window-down; }
    Mod+K { focus-window-up; }
    Mod+L { focus-column-right; }
    Mod+Ctrl+Left { move-column-left; }
    Mod+Ctrl+Down { move-window-down; }
    Mod+Ctrl+Up { move-window-up; }
    Mod+Ctrl+Right { move-column-right; }
    Mod+Ctrl+H { move-column-left; }
    Mod+Ctrl+J { move-window-down; }
    Mod+Ctrl+K { move-window-up; }
    Mod+Ctrl+L { move-column-right; }
    Mod+Home { focus-column-first; }
    Mod+End { focus-column-last; }
    Mod+Ctrl+Home { move-column-to-first; }
    Mod+Ctrl+End { move-column-to-last; }
    Mod+Shift+Left { focus-monitor-left; }
    Mod+Shift+Down { focus-monitor-down; }
    Mod+Shift+Up { focus-monitor-up; }
    Mod+Shift+Right { focus-monitor-right; }
    Mod+Shift+H { focus-monitor-left; }
    Mod+Shift+J { focus-monitor-down; }
    Mod+Shift+K { focus-monitor-up; }
    Mod+Shift+L { focus-monitor-right; }
    Mod+Shift+Ctrl+Left { move-column-to-monitor-left; }
    Mod+Shift+Ctrl+Down { move-column-to-monitor-down; }
    Mod+Shift+Ctrl+Up { move-column-to-monitor-up; }
    Mod+Shift+Ctrl+Right { move-column-to-monitor-right; }
    Mod+Shift+Ctrl+H { move-column-to-monitor-left; }
    Mod+Shift+Ctrl+J { move-column-to-monitor-down; }
    Mod+Shift+Ctrl+K { move-column-to-monitor-up; }
    Mod+Shift+Ctrl+L { move-column-to-monitor-right; }
    Mod+Page_Down { focus-workspace-down; }
    Mod+Page_Up { focus-workspace-up; }
    Mod+U { focus-workspace-down; }
    Mod+I { focus-workspace-up; }
    Mod+Ctrl+Page_Down { move-column-to-workspace-down; }
    Mod+Ctrl+Page_Up { move-column-to-workspace-up; }
    Mod+Ctrl+U { move-column-to-workspace-down; }
    Mod+Ctrl+I { move-column-to-workspace-up; }
    Mod+Shift+Page_Down { move-workspace-down; }
    Mod+Shift+Page_Up { move-workspace-up; }
    Mod+Shift+U { move-workspace-down; }
    Mod+Shift+I { move-workspace-up; }
    Mod+WheelScrollDown cooldown-ms=150 { focus-workspace-down; }
    Mod+WheelScrollUp cooldown-ms=150 { focus-workspace-up; }
    Mod+Ctrl+WheelScrollDown cooldown-ms=150 { move-column-to-workspace-down; }
    Mod+Ctrl+WheelScrollUp cooldown-ms=150 { move-column-to-workspace-up; }
    Mod+WheelScrollRight { focus-column-right; }
    Mod+WheelScrollLeft { focus-column-left; }
    Mod+Ctrl+WheelScrollRight { move-column-right; }
    Mod+Ctrl+WheelScrollLeft { move-column-left; }
    Mod+Shift+WheelScrollDown { focus-column-right; }
    Mod+Shift+WheelScrollUp { focus-column-left; }
    Mod+Ctrl+Shift+WheelScrollDown { move-column-right; }
    Mod+Ctrl+Shift+WheelScrollUp { move-column-left; }
    Mod+1 { focus-workspace 1; }
    Mod+2 { focus-workspace 2; }
    Mod+3 { focus-workspace 3; }
    Mod+4 { focus-workspace 4; }
    Mod+5 { focus-workspace 5; }
    Mod+6 { focus-workspace 6; }
    Mod+7 { focus-workspace 7; }
    Mod+8 { focus-workspace 8; }
    Mod+9 { focus-workspace 9; }
    Mod+Ctrl+1 { move-column-to-workspace 1; }
    Mod+Ctrl+2 { move-column-to-workspace 2; }
    Mod+Ctrl+3 { move-column-to-workspace 3; }
    Mod+Ctrl+4 { move-column-to-workspace 4; }
    Mod+Ctrl+5 { move-column-to-workspace 5; }
    Mod+Ctrl+6 { move-column-to-workspace 6; }
    Mod+Ctrl+7 { move-column-to-workspace 7; }
    Mod+Ctrl+8 { move-column-to-workspace 8; }
    Mod+Ctrl+9 { move-column-to-workspace 9; }
    Mod+BracketLeft { consume-or-expel-window-left; }
    Mod+BracketRight { consume-or-expel-window-right; }
    Mod+Comma { consume-window-into-column; }
    Mod+Period { expel-window-from-column; }
    Mod+R { switch-preset-column-width; }
    Mod+Shift+R { switch-preset-column-width-back; }
    Mod+Ctrl+Shift+R { switch-preset-window-height; }
    Mod+Ctrl+R { reset-window-height; }
    Mod+F { maximize-column; }
    Mod+Shift+F { fullscreen-window; }
    Mod+M { maximize-window-to-edges; }
    Mod+Ctrl+F { expand-column-to-available-width; }
    Mod+C { center-column; }
    Mod+Ctrl+C { center-visible-columns; }
    Mod+Minus { set-column-width "-10%"; }
    Mod+Equal { set-column-width "+10%"; }
    Mod+Shift+Minus { set-window-height "-10%"; }
    Mod+Shift+Equal { set-window-height "+10%"; }
    Mod+V { toggle-window-floating; }
    Mod+Shift+V { switch-focus-between-floating-and-tiling; }
    Mod+W { toggle-column-tabbed-display; }
    Print { screenshot; }
    Ctrl+Print { screenshot-screen; }
    Alt+Print { screenshot-window; }
    Mod+Escape allow-inhibiting=false { toggle-keyboard-shortcuts-inhibit; }
    Ctrl+Alt+Delete { quit; }
    Mod+Shift+P { power-off-monitors; }
}
EOF
```

### 3.2 Noctalia 配置

```bash
mkdir -p ~/.config/noctalia
cat > ~/.config/noctalia/config.toml << 'EOF'
[backdrop]
enabled = true
blur_intensity = 0.5
tint_intensity = 0.3

[theme]
mode = "dark"
source = "builtin"
builtin = "Noctalia"

[wallpaper]
enabled = true
directory = "~/Pictures/Wallpapers"
fill_mode = "crop"

[bar.main]
position = "top"
thickness = 55
background_opacity = 0.8
radius = 20

start = ["launcher", "wallpaper", "workspaces"]
center = ["clock"]
end = ["tray", "notifications", "volume", "battery", "session"]

[weather]
address = "harbin"

[dock]
enabled = true
position = "bottom"
icon_size = 25
auto_hide = false

[idle.behavior.lock]
timeout = 600
command = "noctalia:screen-lock"
enabled = true

[idle.behavior.screen-off]
timeout = 660
command = "noctalia:dpms-off"
resume_command = "noctalia:dpms-on"
enabled = true
EOF
```

---

## 4. 验证与生效

```bash
# 验证 niri 配置语法
niri validate

# 确保壁纸目录存在
mkdir -p ~/Pictures/Wallpapers
```

**最后一步**：注销当前会话，在显示管理器（如 SDDM/GDM/ly）中选择 **niri** 会话重新登录。Noctalia 会通过 `spawn-at-startup` 自动拉起，无需手动启动。

---

## 附：快速排错

| 现象 | 检查项 |
|------|--------|
| Noctalia 没自启 | 确认 `niri validate` 通过，且 `spawn-at-startup "noctalia"` 在配置最顶层 |
| 栏位/组件不显示 | 检查 `~/.local/state/noctalia/settings.toml` 是否覆盖了配置，可临时移走该文件后重登录 |
| 电池/亮度图标缺失 | 确认 `upower` 服务已启用，且用户已在 `power` 组（或安装 `brightnessctl`） |
| Qt 应用图标异常 | 运行 `qt6ct` 选择图标主题，并确保 niri `environment` 块中 `QT_QPA_PLATFORMTHEME` 已设置 |

## 5. 中文输入

```bash
  sudo pacman -S --needed \
    fcitx5 fcitx5-configtool fcitx5-gtk fcitx5-qt \
    fcitx5-chinese-addons fcitx5-pinyin-zhwiki \
    noto-fonts-cjk noto-fonts-emoji

  说明：

    • fcitx5-chinese-addons：包含拼音、双拼、五笔等中文输入法。
    • fcitx5-pinyin-zhwiki：中文维基词库，拼音体验会好很多。
    • fcitx5-gtk / fcitx5-qt：给 GTK/Qt/XWayland 应用兜底。
    • noto-fonts-cjk：避免中文显示方框。
    • noto-fonts-emoji：候选框 Emoji 正常显示。

  如果你想用 Rime / 雾凇拼音

  可额外安装：

  sudo pacman -S --needed fcitx5-rime

  雾凇拼音通常在 AUR：

  paru -S rime-ice-git

  2. 在 niri 中自启动 Fcitx5

  编辑：

    • ~/.config/niri/config.kdl

  加入：

  ~/.config/niri/config.kdl (EXCERPT)
  spawn-at-startup "fcitx5" "-d"

  如果你已经有很多 spawn-at-startup，放在 Noctalia 附近即可。例如：

  ~/.config/niri/config.kdl (EXCERPT)
  spawn-at-startup "fcitx5" "-d"
  spawn-at-startup "noctalia"

  3. 配置 niri 环境变量

  继续编辑 ~/.config/niri/config.kdl，加入或合并 environment 块。

  推荐配置：偏现代 Wayland，兼顾 Qt/XWayland

  ~/.config/niri/config.kdl (EXCERPT)
  environment {
      XMODIFIERS "@im=fcitx"
      QT_IM_MODULE "fcitx"
      QT_IM_MODULES "wayland;fcitx;ibus"
      GTK_IM_MODULE null
      SDL_IM_MODULE null
      GLFW_IM_MODULE null
  }

  解释一下：

    • XMODIFIERS "@im=fcitx"：让 XWayland/X11 程序能找到 Fcitx。
    • QT_IM_MODULE "fcitx"：照顾 Qt5、老 Qt、部分 XWayland Qt 应用。
    • QT_IM_MODULES "wayland;fcitx;ibus"：Qt 6.7+ 支持的 fallback 顺序，优先 Wayland，失败再用 fcitx。
    • GTK_IM_MODULE null：不要强行让 GTK 走 fcitx module，优先让 GTK3/GTK4 使用 Wayland text-input。
    • SDL_IM_MODULE null / GLFW_IM_MODULE null：避免部分游戏/SDL/GLFW 应用被全局 IM module 干扰。

  4. 验证 niri 配置

  保存后执行：

  niri validate
```

## 6. 字体配置

```bash
**在 Arch Linux + Niri 上安装并默认使用类似苹果的字体方案（SF Pro + 苹方 PingFang）**  
以下是完整整合指南，英文使用 **SF Pro**，中文优先使用 **苹方 (PingFang SC)**，实现 macOS-like 体验。

### 1. 安装字体（推荐 AUR）

```bash
# 安装 SF Pro 系列（包含 SF Pro Display/Text、SF Mono 等）
yay -S apple-sf-fonts

# 单独安装苹方（PingFang SC / HK 等）
yay -S otf-apple-pingfang
```

- 如果 `yay` 未安装：  
  `sudo pacman -S --needed git base-devel && git clone https://aur.archlinux.org/yay.git && cd yay && makepkg -si`
- 安装完后刷新字体缓存：  
  ```bash
  fc-cache -fv
  ```

**手动安装**（有 Mac 时）：从 Mac 的 `/System/Library/Fonts/` 或 `/Library/Fonts/` 复制 SF Pro 和 PingFang 相关 `.otf`/`.ttc` 文件到 `~/.local/share/fonts/`，然后 `fc-cache -fv`。

### 2. 综合 fontconfig 配置（核心）

创建用户配置文件：

```bash
mkdir -p ~/.config/fontconfig/conf.d
nano ~/.config/fontconfig/fonts.conf
```

**推荐完整配置**（SF Pro + 苹方优先）：

```xml
<?xml version='1.0'?>
<!DOCTYPE fontconfig SYSTEM 'urn:fontconfig:fonts.dtd'>
<fontconfig>

  <!-- 系统 UI / sans-serif 默认使用 SF Pro（英文/拉丁文字） -->
  <alias>
    <family>system-ui</family>
    <prefer>
      <family>SF Pro Display</family>   <!-- 大字号/UI -->
      <family>SF Pro Text</family>      <!-- 小字号/正文 -->
    </prefer>
  </alias>

  <alias>
    <family>sans-serif</family>
    <prefer>
      <family>SF Pro Display</family>
      <family>SF Pro Text</family>
      <family>PingFang SC</family>      <!-- 中文 fallback -->
    </prefer>
  </alias>

  <alias>
    <family>sans</family>
    <prefer>
      <family>SF Pro Display</family>
      <family>SF Pro Text</family>
      <family>PingFang SC</family>
    </prefer>
  </alias>

  <!-- 等宽字体（可选 SF Mono） -->
  <alias>
    <family>monospace</family>
    <prefer>
      <family>SF Mono</family>
    </prefer>
  </alias>

  <!-- 中文语言优先使用苹方（更精确控制） -->
  <match>
    <test name="lang" compare="contains">
      <string>zh</string>
    </test>
    <test name="family">
      <string>sans-serif</string>
    </test>
    <edit name="family" mode="prepend">
      <string>PingFang SC</string>
    </edit>
  </match>

  <match>
    <test name="lang" compare="contains">
      <string>zh</string>
    </test>
    <test name="family">
      <string>serif</string>
    </test>
    <edit name="family" mode="prepend">
      <string>PingFang SC</string>
    </edit>
  </match>

</fontconfig>
```

**应用配置**：
```bash
fc-cache -fv
```

### 3. 字体渲染优化（类似 macOS 清晰度）

```bash
nano ~/.config/fontconfig/conf.d/99-apple-rendering.conf
```

内容：

```xml
<?xml version='1.0'?>
<!DOCTYPE fontconfig SYSTEM 'urn:fontconfig:fonts.dtd'>
<fontconfig>
  <match target="font">
    <edit name="hinting" mode="assign"><bool>true</bool></edit>
    <edit name="hintstyle" mode="assign"><const>hintslight</const></edit>
    <edit name="antialias" mode="assign"><bool>true</bool></edit>
    <edit name="rgba" mode="assign"><const>rgb</const></edit>  <!-- 根据屏幕调整：rgb / bgr -->
    <edit name="lcdfilter" mode="assign"><const>lcddefault</const></edit>
  </match>
</fontconfig>
```

刷新：`fc-cache -fv`

### 4. 验证效果

```bash
fc-match sans-serif
fc-match system-ui
fc-match -s "你好，世界"   # 应该优先显示 PingFang SC
fc-list | grep -E "SF Pro|PingFang"
```

### 注意事项

- **Niri / Wayland**：大多数现代应用（Firefox、Chrome、Alacritty、GTK/Qt）会遵循 fontconfig。部分终端或 Electron 应用可能需单独设置字体。
- **语言切换**：中文环境下 `zh` 匹配会优先苹方；英文/其他语言优先 SF Pro。
- **备选**：如果不想用 Apple 字体，可用 `ttf-inter` + `adobe-source-han-sans-cn-fonts` 替代。
- **法律提醒**：这些是 Apple 版权字体，仅限个人使用。
- **高 DPI**：在 Niri 配置中合理设置缩放，效果最佳。

重启相关应用或注销重进 Niri 后即可看到接近 macOS 的字体观感（英文锐利、中文苹方和谐）。

如果验证后中文仍未优先，或有特定应用问题，运行 `fc-match --verbose sans-serif` 并提供输出，我可以进一步调整配置！
```