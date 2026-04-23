---
title: wezterm配置
slug: wezterm配置
summary: ''
category: []
tags: []
status: published
updated: 2026-04-23T10:55
aliases: []
---
## 配置文件

编辑 `~/.config/wezterm/wezterm.lua`

### macOS

```lua
-- ~/.wezterm.lua
local wezterm = require 'wezterm'
local config = wezterm.config_builder()
local act = wezterm.action

-- ========== 字体设置 ==========
config.font = wezterm.font_with_fallback({
    { family = "Maple Mono NF CN", weight = "Medium" },
})
config.font_size = 14.0
config.line_height = 1.2

-- ========== 窗口外观（macOS 专属） ==========
config.initial_cols = 120
config.initial_rows = 35
config.window_padding = { left = 8, right = 8, top = 50, bottom = 8 }  -- top 从 8 改为 50

-- macOS 毛玻璃效果 + 透明度
config.window_background_opacity = 0.92
config.macos_window_background_blur = 20

-- 隐藏标题栏但保留圆角（RESIZE: 仅保留调整大小边框；INTEGRATED_BUTTONS: 内置按钮）
config.window_decorations = "INTEGRATED_BUTTONS|RESIZE"

-- ========== 配色方案 ==========
-- 查看所有内置主题：wezterm ls-fonts 或访问 https://wezfurlong.org/wezterm/colorschemes/
config.color_scheme = "Catppuccin Mocha"
-- 其他推荐：Catppuccin Mocha, Dracula, Gruvbox Dark, Nord

-- ========== 标签栏设置 ==========
config.enable_tab_bar = true
config.use_fancy_tab_bar = false      -- 简洁模式，类似 iTerm2
config.tab_bar_at_bottom = true       -- 标签栏放底部
config.show_tab_index_in_tab_bar = true
config.tab_max_width = 25

-- ========== 光标设置 ==========
config.default_cursor_style = "BlinkingBar"
config.cursor_blink_rate = 2000

-- ========== 滚动与性能 ==========
config.scrollback_lines = 10000
config.max_fps = 120
config.animation_fps = 60

-- ========== 键盘快捷键（macOS 风格） ==========
config.keys = {
    -- 复制粘贴
    { key = "c", mods = "CMD", action = act.CopyTo("Clipboard") },
    { key = "v", mods = "CMD", action = act.PasteFrom("Clipboard") },

    -- 新建/关闭标签页
    { key = "t", mods = "CMD", action = act.SpawnTab("CurrentPaneDomain") },
    { key = "w", mods = "CMD", action = act.CloseCurrentTab{ confirm = true } },

    -- 切换标签页
    { key = "]", mods = "CMD|SHIFT", action = act.ActivateTabRelative(1) },
    { key = "[", mods = "CMD|SHIFT", action = act.ActivateTabRelative(-1) },
    { key = "1", mods = "CMD", action = act.ActivateTab(0) },
    { key = "2", mods = "CMD", action = act.ActivateTab(1) },
    { key = "3", mods = "CMD", action = act.ActivateTab(2) },
    { key = "4", mods = "CMD", action = act.ActivateTab(3) },
    { key = "5", mods = "CMD", action = act.ActivateTab(4) },

    -- 分屏（类似 iTerm2）
    { key = "d", mods = "CMD", action = act.SplitHorizontal{ domain = "CurrentPaneDomain" } },
    { key = "d", mods = "CMD|SHIFT", action = act.SplitVertical{ domain = "CurrentPaneDomain" } },
    { key = "x", mods = "CMD", action = act.CloseCurrentPane{ confirm = false } },

    -- 切换窗格
    { key = "LeftArrow", mods = "CMD|ALT", action = act.ActivatePaneDirection("Left") },
    { key = "RightArrow", mods = "CMD|ALT", action = act.ActivatePaneDirection("Right") },
    { key = "UpArrow", mods = "CMD|ALT", action = act.ActivatePaneDirection("Up") },
    { key = "DownArrow", mods = "CMD|ALT", action = act.ActivatePaneDirection("Down") },

    -- 全屏
    { key = "f", mods = "CMD|CTRL", action = act.ToggleFullScreen },

    -- 清屏
    { key = "k", mods = "CMD", action = act.ClearScrollback("ScrollbackAndViewport") },

    -- 快速选择模式（类似 Vim 的 f/F）
    { key = "f", mods = "CMD|SHIFT", action = act.QuickSelect },

    -- 重载配置
    { key = "r", mods = "CMD|SHIFT", action = act.ReloadConfiguration },

    -- 字体大小调整
    { key = "=", mods = "CMD", action = act.IncreaseFontSize },
    { key = "-", mods = "CMD", action = act.DecreaseFontSize },
    { key = "0", mods = "CMD", action = act.ResetFontSize },
}

-- ========== 鼠标绑定 ==========
config.mouse_bindings = {
    -- 右键粘贴
    {
        event = { Down = { streak = 1, button = "Right" } },
        mods = "NONE",
        action = act.PasteFrom("Clipboard"),
    },
    -- Cmd+点击打开链接
    {
        event = { Up = { streak = 1, button = "Left" } },
        mods = "CMD",
        action = act.OpenLinkAtMouseCursor,
    },
}

-- ========== 启动设置 ==========
config.default_prog = { "/opt/homebrew/bin/fish", "-l" }

-- 启动时最大化窗口
wezterm.on("gui-startup", function(cmd)
    local tab, pane, window = wezterm.mux.spawn_window(cmd or {})
    window:gui_window():maximize()
end)

-- ========== 状态栏自定义 ==========
wezterm.on("update-status", function(window, pane)
    local date = wezterm.strftime("%m/%d %H:%M")
    window:set_right_status(wezterm.format({
        { Foreground = { Color = "#7aa2f7" } },
        { Background = { Color = "#1a1b26" } },
        { Text = " " .. date .. " " },
    }))
end)

-- ========== 其他实用设置 ==========
config.automatically_reload_config = true   -- 自动重载配置
config.enable_scroll_bar = true             -- 启用滚动条
config.warn_about_missing_glyphs = false    -- 不警告缺失字形
config.audible_bell = "Disabled"            -- 关闭响铃

return config
```

### Linux

```lua
local wezterm = require 'wezterm'
local config = wezterm.config_builder()
local act = wezterm.action

-- ========== 字体设置 ==========
config.font = wezterm.font_with_fallback({
    { family = "Maple Mono NF CN", weight = "Medium" },
})
config.font_size = 12.0
config.line_height = 1.2

-- ========== 窗口外观 ==========
--config.initial_cols = 120
--config.initial_rows = 35
config.window_padding = { left = 8, right = 8, top = 8, bottom = 8 }

-- Linux 下透明度支持取决于 compositor（比如 picom / kwin）
config.window_background_opacity = 0.95

-- 标题栏
config.window_decorations = "TITLE|RESIZE"

-- ========== 配色方案 ==========
config.color_scheme = "Catppuccin Mocha"

-- ========== 标签栏 ==========
config.enable_tab_bar = true
config.use_fancy_tab_bar = false
config.tab_bar_at_bottom = true
config.show_tab_index_in_tab_bar = true
config.tab_max_width = 25

-- ========== 光标 ==========
config.default_cursor_style = "BlinkingBar"
config.cursor_blink_rate = 800

-- ========== 性能 ==========
config.scrollback_lines = 10000
config.max_fps = 120
config.animation_fps = 60

-- ========== Linux 风格快捷键 ==========
config.keys = {
    -- 复制粘贴（统一标准）
    { key = "c", mods = "CTRL|SHIFT", action = act.CopyTo("Clipboard") },
    { key = "v", mods = "CTRL|SHIFT", action = act.PasteFrom("Clipboard") },

    -- 标签页
    { key = "t", mods = "CTRL|SHIFT", action = act.SpawnTab("CurrentPaneDomain") },
    { key = "w", mods = "CTRL|SHIFT", action = act.CloseCurrentTab{ confirm = true } },

    -- 切换标签页
    { key = "RightArrow", mods = "CTRL|SHIFT", action = act.ActivateTabRelative(1) },
    { key = "LeftArrow", mods = "CTRL|SHIFT", action = act.ActivateTabRelative(-1) },
    { key = "1", mods = "CTRL", action = act.ActivateTab(0) },
    { key = "2", mods = "CTRL", action = act.ActivateTab(1) },
    { key = "3", mods = "CTRL", action = act.ActivateTab(2) },
    { key = "4", mods = "CTRL", action = act.ActivateTab(3) },
    { key = "5", mods = "CTRL", action = act.ActivateTab(4) },

    -- 分屏（接近 :contentReference[oaicite:0]{index=0} 习惯）
    { key = "d", mods = "CTRL|SHIFT", action = act.SplitHorizontal{ domain = "CurrentPaneDomain" } },
    { key = "d", mods = "CTRL|ALT", action = act.SplitVertical{ domain = "CurrentPaneDomain" } },
    { key = "x", mods = "CTRL|SHIFT", action = act.CloseCurrentPane{ confirm = false } },

    -- Pane 切换
    { key = "LeftArrow", mods = "ALT", action = act.ActivatePaneDirection("Left") },
    { key = "RightArrow", mods = "ALT", action = act.ActivatePaneDirection("Right") },
    { key = "UpArrow", mods = "ALT", action = act.ActivatePaneDirection("Up") },
    { key = "DownArrow", mods = "ALT", action = act.ActivatePaneDirection("Down") },

    -- 全屏
    { key = "F11", mods = "NONE", action = act.ToggleFullScreen },

    -- 清屏
    { key = "k", mods = "CTRL|SHIFT", action = act.ClearScrollback("ScrollbackAndViewport") },

    -- 快速选择
    { key = "f", mods = "CTRL|SHIFT", action = act.QuickSelect },

    -- 重载配置
    { key = "r", mods = "CTRL|SHIFT", action = act.ReloadConfiguration },

    -- 字体缩放
    { key = "=", mods = "CTRL", action = act.IncreaseFontSize },
    { key = "-", mods = "CTRL", action = act.DecreaseFontSize },
    { key = "0", mods = "CTRL", action = act.ResetFontSize },
}

-- ========== 鼠标 ==========
config.mouse_bindings = {
    -- 右键粘贴
    {
        event = { Down = { streak = 1, button = "Right" } },
        mods = "NONE",
        action = act.PasteFrom("Clipboard"),
    },
    -- Ctrl + 点击打开链接（Linux 常见习惯）
    {
        event = { Up = { streak = 1, button = "Left" } },
        mods = "CTRL",
        action = act.OpenLinkAtMouseCursor,
    },
}

-- ========== 启动 shell ==========
config.default_prog = { "powershell" }

-- 启动时最大化窗口
wezterm.on("gui-startup", function(cmd)
    local _, _, window = wezterm.mux.spawn_window(cmd or {})
    window:gui_window():maximize()
end)

-- ========== 状态栏 ==========
wezterm.on("update-status", function(window, pane)
    local date = wezterm.strftime("%Y-%m-%d %H:%M")
    window:set_right_status(wezterm.format({
        { Foreground = { Color = "#7aa2f7" } },
        { Background = { Color = "#1a1b26" } },
        { Text = " " .. date .. " " },
    }))
end)

-- ========== 其他 ==========
config.automatically_reload_config = true
config.enable_scroll_bar = true
config.warn_about_missing_glyphs = false
config.audible_bell = "Disabled"

return config
```

## 将 WezTerm 的 CLI 目录加入 PATH

```
# WezTerm CLI
fish_add_path /Applications/WezTerm.app/Contents/MacOS
```