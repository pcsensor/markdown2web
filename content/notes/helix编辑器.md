---
title: helix编辑器
slug: helix编辑器
summary: helix好用
category: []
tags: []
status: published
updated: 2026-05-20T12:03
aliases: []
---
# 安装

1. macOS

```
# 1. 使用 Homebrew 一键安装
brew install helix

# 2. 验证是否安装成功
hx --version
```

2. arch

```
sudo pacman -S helix
```

# LSP服务

```
hx --health
```

按需安装需要的lsp服务

# 优化配置

```
mkdir -p ~/.config/helix
touch ~/.config/helix/config.toml
```

```
# ==============================================================================
# 🎨 视觉主题与色彩
# ==============================================================================
# 极力推荐的现代高颜值主题（可根据喜好更换）：
# "tokyonight" (东京夜) / "catppuccin_mocha" (猫粮) / "onedark" (经典一暗) / "amberwood"
theme = "catppuccin_mocha"

# ==============================================================================
# 🛠️ 核心编辑器行为
# ==============================================================================
[editor]
line-number = "relative"      # 开启相对行号（写代码/跳行神器，当前行显示绝对行号）
mouse = true                  # 完美支持鼠标点击、滚动和选择
scrolloff = 5                 # 光标距离屏幕顶部/底部 5 行时自动滚动，视野更宽阔
color-modes = true            # 在右下角用不同的颜色高亮显示当前模式（Normal/Insert/Select）

# ==============================================================================
# 🪟 现代界面元素增强
# ==============================================================================
[editor.cursor-shape]
insert = "bar"                # 插入模式下光标变成一条竖线（现代化流线型）
normal = "block"              # 普通模式下光标是方块
select = "underline"          # 选择模式下光标是下划线

[editor.file-picker]
hidden = false                # 文件搜索器（Space+f）默认会搜索隐藏文件（如 .gitignore 里的项目）

[editor.indent-guides]
render = true                 # 💡 开启彩色代码缩进对齐线，层级一目了然
character = "│"               # 缩进线的样式
skip-levels = 1               # 第一层缩进不渲染线，保持左侧整洁

# ==============================================================================
# 📊 状态栏（Statusline）像素级魔改
# ==============================================================================
# 让你的终端底部拥有类似 Neovim 华丽状态栏的现代彩色色块
[editor.statusline]
left = ["mode", "spinner", "file-name", "file-modification-indicator"]
center = []
right = ["diagnostics", "selections", "register", "position", "file-encoding", "file-type"]
mode.normal = "NORMAL"
mode.insert = "INSERT"
mode.select = "SELECT"
```