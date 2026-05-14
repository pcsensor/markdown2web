---
title: debian下LaTex安装、配置
slug: debian下latex安装-配置
summary: debian下LaTex安装、配置
category: []
tags: []
status: published
updated: 2026-05-01T17:58
aliases: []
---
# 0. 最终目标结构

最终你会得到这样一套环境：

```text
Debian
├── fish shell
├── 官方 TeX Live
│   └── /usr/local/texlive/2026/bin/x86_64-linux
├── TeXstudio AppImage
│   └── /home/pcsensor/Applications/texstudio.AppImage
├── fuzzel 启动项
│   └── /home/pcsensor/.local/share/applications/texstudio.desktop
└── 中文 LaTeX 编译方式
    └── latexmk + xelatex
```

最终使用流程：

```text
fuzzel → 搜 TeXstudio AppImage → 回车 → 写 main.tex → F5 编译 → 生成 PDF
```

---

# 1. 安装基础依赖

先更新系统：

```fish
sudo apt update
```

安装官方 TeX Live 安装器需要的工具：

```fish
sudo apt install perl wget xzdec fontconfig
```

建议额外安装一些桌面启动项相关工具：

```fish
sudo apt install desktop-file-utils
```

如果后面运行 AppImage 提示 FUSE 问题，再安装：

```fish
sudo apt install libfuse2
```

---

# 2. 安装官方 TeX Live

这里使用的是 **TeX Live 官方安装器**，不是 Debian 仓库里的 `texlive-full`。

## 2.1 下载官方网络安装器

进入临时目录：

```fish
cd /tmp
```

下载：

```fish
wget https://mirror.ctan.org/systems/texlive/tlnet/install-tl-unx.tar.gz
```

解压：

```fish
tar -xzf install-tl-unx.tar.gz
```

进入安装目录：

```fish
cd install-tl-*
```

---

## 2.2 启动安装器

执行：

```fish
sudo perl install-tl
```

进入交互界面后，如果你想默认安装，直接输入：

```text
i
```

然后回车。

安装完成后，官方 TeX Live 通常会被安装到类似：

```text
/usr/local/texlive/2026
```

具体年份以你安装时的 TeX Live 版本为准。

---

# 3. fish shell 配置 TeX Live 环境变量

你使用的是 **fish shell**，所以不要照搬 bash/zsh 的 `export PATH=...` 写法。

## 3.1 推荐方式：fish_add_path

执行：

```fish
fish_add_path /usr/local/texlive/2026/bin/x86_64-linux
```

然后设置 TeX Live 文档路径：

```fish
set -Ux MANPATH /usr/local/texlive/2026/texmf-dist/doc/man $MANPATH
set -Ux INFOPATH /usr/local/texlive/2026/texmf-dist/doc/info $INFOPATH
```

重新打开一个终端，验证：

```fish
which xelatex
which latexmk
which tlmgr
```

理想输出应该是：

```text
/usr/local/texlive/2026/bin/x86_64-linux/xelatex
/usr/local/texlive/2026/bin/x86_64-linux/latexmk
/usr/local/texlive/2026/bin/x86_64-linux/tlmgr
```

继续验证版本：

```fish
xelatex --version
latexmk --version
tlmgr --version
```

---

## 3.2 也可以写入 config.fish

如果你想手动写进 fish 配置文件：

```fish
mkdir -p ~/.config/fish
nano ~/.config/fish/config.fish
```

加入：

```fish
# TeX Live 2026
set -gx PATH /usr/local/texlive/2026/bin/x86_64-linux $PATH
set -gx MANPATH /usr/local/texlive/2026/texmf-dist/doc/man $MANPATH
set -gx INFOPATH /usr/local/texlive/2026/texmf-dist/doc/info $INFOPATH
```

加载：

```fish
source ~/.config/fish/config.fish
```

不过我更建议你用：

```fish
fish_add_path /usr/local/texlive/2026/bin/x86_64-linux
```

更符合 fish 的习惯。

---

# 4. 安装中文字体

为了中文 LaTeX 文档正常显示，安装中文字体：

```fish
sudo apt install fonts-noto-cjk fonts-noto-cjk-extra fonts-noto-color-emoji
```

再安装文泉驿字体：

```fish
sudo apt install fonts-wqy-microhei fonts-wqy-zenhei
```

刷新字体缓存：

```fish
fc-cache -fv
```

---

# 5. 测试 TeX Live 中文编译

创建测试目录：

```fish
mkdir -p ~/tex-test
cd ~/tex-test
```

创建测试文件：

```fish
nano main.tex
```

写入：

```tex
\documentclass[UTF8]{ctexart}

\title{Debian TeX Live 测试}
\author{Justin}
\date{\today}

\begin{document}

\maketitle

你好，TeX Live。

这是一个中文 XeLaTeX 测试文档。

\[
E = mc^2
\]

\end{document}
```

使用 XeLaTeX 编译：

```fish
xelatex main.tex
```

或者使用 latexmk：

```fish
latexmk -xelatex main.tex
```

如果生成：

```text
main.pdf
```

说明官方 TeX Live 安装成功。

清理中间文件：

```fish
latexmk -c
```

完整清理，包括 PDF：

```fish
latexmk -C
```

---

以下是针对 **Debian 13 + niri (Wayland) + fcitx5** 环境的 **VSCode + LaTeX Workshop** 最新完整配置方案。核心思路是：**latexmk 自动化编译 + xelatex 中文支持 + 内置 PDF 标签页预览 + 辅助文件隔离到 out 目录**。

---

# Vscode编写

## 一、环境准备

```bash
# 1. 确认 TeX Live 已安装（你之前已装，此处仅检查）
xelatex --version
latexmk --version

# 2. 安装 latexindent（格式化用，依赖 Perl）
sudo apt update
sudo apt install latexindent

# 3. 安装 VSCode（如未安装）
sudo apt install code  # 或从官方仓库/flatpak 安装
```

---

## 二、安装 VSCode 扩展

在 VSCode 中按 `Ctrl+Shift+X`，安装：

1. **LaTeX Workshop**（`James-Yu.latex-workshop`）—— 唯一必需的扩展，提供编译、预览、补全、大纲等功能。
2. **LTeX**（可选）—— 语法拼写检查。
3. **latex-utilities**（可选）—— 额外辅助工具。

---

## 三、settings.json 完整配置

按 `Ctrl+Shift+P` → 输入 `Open User Settings (JSON)`，将以下配置**追加**到你的 `settings.json` 中。这份配置覆盖了中文 LaTeX 写作所需的全部工具链、PDF 预览、SyncTeX 双向搜索和自动清理。

```json
{
    "editor.accessibilitySupport": "on",
    "workbench.colorTheme": "Atom One Light",
    // ==================== 1. 编译工具 (Tools) ====================
    "latex-workshop.latex.tools": [
        {
            "name": "latexmk-xelatex",
            "command": "latexmk",
            "args": [
                "-synctex=1",
                "-interaction=nonstopmode",
                "-file-line-error",
                "-xelatex",
                "-outdir=%OUTDIR%",
                "%DOC%"
            ],
            "env": {}
        },
        {
            "name": "latexmk-lualatex",
            "command": "latexmk",
            "args": [
                "-synctex=1",
                "-interaction=nonstopmode",
                "-file-line-error",
                "-lualatex",
                "-outdir=%OUTDIR%",
                "%DOC%"
            ],
            "env": {}
        },
        {
            "name": "latexmk-pdflatex",
            "command": "latexmk",
            "args": [
                "-synctex=1",
                "-interaction=nonstopmode",
                "-file-line-error",
                "-pdf",
                "-outdir=%OUTDIR%",
                "%DOC%"
            ],
            "env": {}
        },
        {
            "name": "xelatex",
            "command": "xelatex",
            "args": [
                "-synctex=1",
                "-interaction=nonstopmode",
                "-file-line-error",
                "-output-directory=%OUTDIR%",
                "%DOC%"
            ],
            "env": {}
        },
        {
            "name": "pdflatex",
            "command": "pdflatex",
            "args": [
                "-synctex=1",
                "-interaction=nonstopmode",
                "-file-line-error",
                "-output-directory=%OUTDIR%",
                "%DOC%"
            ],
            "env": {}
        },
        {
            "name": "bibtex",
            "command": "bibtex",
            "args": [ "%OUTDIR%/%DOCFILE%" ],
            "env": {}
        },
        {
            "name": "biber",
            "command": "biber",
            "args": [ "%OUTDIR%/%DOCFILE%" ],
            "env": {}
        },
        {
            "name": "tectonic",
            "command": "tectonic",
            "args": [
                "--synctex",
                "--keep-logs",
                "--keep-intermediates",
                "--outdir", "%OUTDIR%",
                "%DOC%.tex"
            ],
            "env": {}
        }
    ],

    // ==================== 2. 编译配方 (Recipes) ====================
    "latex-workshop.latex.recipes": [
        {
            "name": "latexmk (xelatex)",
            "tools": [ "latexmk-xelatex" ]
        },
        {
            "name": "latexmk (lualatex)",
            "tools": [ "latexmk-lualatex" ]
        },
        {
            "name": "latexmk (pdflatex)",
            "tools": [ "latexmk-pdflatex" ]
        },
        {
            "name": "xelatex → biber → xelatex×2",
            "tools": [ "xelatex", "biber", "xelatex", "xelatex" ]
        },
        {
            "name": "tectonic",
            "tools": [ "tectonic" ]
        }
    ],

    // ==================== 3. 默认行为 ====================
    // 中文文档默认用 xelatex；若写英文可改为 "latexmk (pdflatex)"
    "latex-workshop.latex.recipe.default": "latexmk (xelatex)",
    // 保存时自动编译；若嫌烦可改为 "never"，手动按 Ctrl+Alt+B
    "latex-workshop.latex.autoBuild.run": "onSave",
    // 关闭 Magic Comments 优先，完全由 settings.json 控制编译器[^39^]
    "latex-workshop.latex.build.enableMagicComments": false,
    // 输出目录：在项目根目录创建 out/ 文件夹隔离辅助文件
    "latex-workshop.latex.outDir": "%DIR%/out",

    // ==================== 4. PDF 预览 & SyncTeX ====================
    // 在 VSCode 标签页内预览（无需外部 PDF 阅读器）
    "latex-workshop.view.pdf.viewer": "tab",
    // 双击 PDF 反向定位到代码
    "latex-workshop.view.pdf.internal.synctex.keybinding": "double-click",
    // 启用 synctex 支持
    "latex-workshop.synctex.synctexjs.enabled": true,

    // ==================== 5. 格式化 ====================
    "latex-workshop.formatting.latexindent.path": "latexindent",
    "latex-workshop.formatting.latexindent.args": [
        "-c", "%DIR%/.latexindent",
        "%TMPFILE%",
        "-y=defaultIndent: '%INDENT%'"
    ],

    // ==================== 6. 智能感知 & 清理 ====================
    "latex-workshop.intellisense.package.enabled": true,
    "latex-workshop.intellisense.update.aggressive.enabled": false,
    "latex-workshop.intellisense.update.delay": 1000,
    // 编译失败时自动清理临时文件
    "latex-workshop.latex.autoClean.run": "onFailed",
    "latex-workshop.latex.clean.subFolder.enabled": true,
    "latex-workshop.latex.clean.fileTypes": [
        "*.aux", "*.bbl", "*.blg", "*.idx", "*.ind", "*.lof", "*.lot",
        "*.out", "*.toc", "*.acn", "*.acr", "*.alg", "*.glg", "*.glo",
        "*.gls", "*.ist", "*.fls", "*.log", "*.fdb_latexmk", "*.synctex.gz",
        "*.run.xml", "*.xdv"
    ],

    // ==================== 7. 消息控制 ====================
    "latex-workshop.message.error.show": true,
    "latex-workshop.message.warning.show": true,
    "latex-workshop.message.information.show": false,
    "latex-workshop.message.log.show": false,

    // ==================== 8. 编辑器优化 (仅对 .tex 生效) ====================
    "[latex]": {
        "editor.formatOnSave": false,
        "editor.wordWrap": "on",
        "editor.quickSuggestions": {
            "comments": "off",
            "strings": "on",
            "other": "on"
        },
        "editor.bracketPairColorization.enabled": true
    },

    // ==================== 9. 数学公式实时预览 ====================
    "latex-workshop.hover.preview.enabled": true,
    "latex-workshop.mathpreviewpanel.enabled": true
}
```

---

## 四、Wayland + fcitx5 输入法适配

VSCode 基于 Electron，在 niri 这类纯 Wayland 环境下，fcitx5 输入可能因 Ozone 平台后端选择而异常。建议按以下优先级排查：

### 方案 A：让 VSCode 原生走 Wayland（推荐先尝试）

创建/编辑 `~/.config/code-flags.conf`（若使用 OSS 版可能是 `code-oss`）：

```bash
--ozone-platform-hint=auto
--enable-features=WaylandWindowDecorations
```

确保 niri 的 `text-input-v3` 已启用（niri 基于 wlroots 0.18+，默认支持）。同时在你的 shell 配置文件（`~/.config/fish/config.fish` 或 `~/.bashrc`）中确认：

```bash
export XMODIFIERS=@im=fcitx
export GTK_IM_MODULE=fcitx
# QT_IM_MODULE 不要全局设，否则会影响其他 Qt 应用
```

### 方案 B：强制 VSCode 回退 XWayland（如果方案 A 输入法不工作）

直接修改启动方式，让 VSCode 运行在 XWayland 下，fcitx5 的 XIM/GTK 模块可完美兼容：

```bash
# 启动 VSCode 时附加参数
code --ozone-platform=x11
```

或者将 `~/.config/code-flags.conf` 改为：

```bash
--ozone-platform=x11
```

> 实测在 niri 下，VSCode 走 XWayland 对 LaTeX Workshop 的 PDF 预览和 SyncTeX 没有任何负面影响，仅窗口由 XWayland 托管。

---

## 五、日常使用 Workflow

| 操作 | 快捷键 / 方式 |
|------|--------------|
| **编译** | `Ctrl + Alt + B`（调用默认 recipe） |
| **查看 PDF** | `Ctrl + Alt + V`（在右侧/新标签页打开 PDF） |
| **正向搜索** | 在 `.tex` 中光标定位 → `Ctrl + Alt + J`（跳转到 PDF 对应位置） |
| **反向搜索** | 在 PDF 预览中 **双击** 文字（跳回 `.tex` 对应行） |
| **切换编译器** | 点击左侧 TeX 面板 → 顶部 Recipe 下拉菜单，选择 `latexmk (lualatex)` 等 |
| **清理辅助文件** | `Ctrl + Alt + C` |

---

## 六、（可选）项目级 `.latexmkrc` 进阶

如果你希望**不依赖 VSCode 也能复现编译行为**，可在项目根目录放 `.latexmkrc`：

```perl
# 使用 xelatex 作为默认引擎
$pdf_mode = 5;

# 传递给 xelatex 的参数
$xelatex = "xelatex -shell-escape -file-line-error -halt-on-error -interaction=nonstopmode -synctex=1 %O %S";

# 输出目录
$out_dir = 'out';

# 编译后自动清理（可选）
$clean_ext = "aux bbl bcf blg idx ind loa lof lot out toc acn acr alg glg glo gls ist fls log fdb_latexmk spl run.xml";
```

这样即便在终端执行 `latexmk`，行为也与 VSCode 中完全一致。

---

## 七、常见问题速查

| 现象 | 解决 |
|------|------|
| 编译报错 `xelatexmk not found` | 说明你之前的 `settings.json` 覆盖了默认 tools，请直接复制上方完整配置 |
| 中文显示为方框/空白 | 检查文档是否使用 `\usepackage{xeCJK}` 或 `ctex` 宏包，且系统有对应字体 |
| PDF 预览空白或打不开 | 检查 `latex-workshop.view.pdf.viewer` 是否为 `tab`；niri 下 Webview 通常正常 |
| 保存时频繁自动编译 | 将 `latex-workshop.latex.autoBuild.run` 改为 `never` |
| 需要 `minted` 代码高亮 | 使用 `xelatex -shell-escape`，可额外添加一个 `latexmk-xelatex-shell-escape` 工具 |

配置完成后，打开任意 `.tex` 文件，左侧活动栏会出现 **TeX 图标**，点击即可看到文档大纲、Snippet 面板和编译状态。