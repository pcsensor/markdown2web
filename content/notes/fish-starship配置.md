---
title: fish+starship配置
slug: fish-starship配置
summary: ''
category: []
tags: []
status: published
updated: 2026-04-22T09:31
aliases: []
---
## starship安装

```
curl -sS https://starship.rs/install.sh | sh
```

## fish配置

配置文件 `~/.config/fish/config.fish` 。

```
if status is-interactive
    set -gx STARSHIP_CONFIG /home/pcsensor/.config/starship.toml
    starship init fish | source
end

# 环境变量
fish_add_path /home/pcsensor/.cargo/bin
fish_add_path /home/pcsensor/.nvm/versions/node/v24.15.0/bin/npm
fish_add_path /home/pcsensor/.nvm/versions/node/v24.15.0

# 别名
abbr -a la 'eza -lah'
```

## nushell

`$nu.config-path`

```
# ====================== Starship ======================
mkdir ($nu.data-dir | path join "vendor/autoload")

# 生成 starship 初始化文件
starship init nu | save -f ($nu.data-dir | path join "vendor/autoload/starship.nu")

# 加载 Starship
use ($nu.data-dir | path join "vendor/autoload/starship.nu")
```

## 主题

```
starship preset --list

starship preset catppuccin-powerline -o ~/.config/starship.toml
```

## 配置文件相关

### 关闭打印下一行提示符之前自动插入一个空行

打开 `~/.config/starship.toml` 

```toml,ini
"$schema" = 'https://starship.rs/config-schema.json'

# 关闭提示符之前的默认空行
add_newline = false

format = """
[](red)\
$os\
# ... 保持原有的配置不变 ...
```

## 生效

```
source ~/.config/fish/.fish
```
