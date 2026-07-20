---
title: debian安装nvidia驱动
slug: debian安装nvidia驱动
summary: 下次记得在BISO屏蔽独显
category: []
tags: []
status: published
updated: 2026-04-21T00:01
aliases: []
---
## 查看当前正在使用的驱动

```
lspci

lspci -k -s 01:00.0
```

注：01:00.0 是你刚才 lspci 中 NVIDIA 显卡的编号

## 准备软件源

软件源文件确保每一行后面都有 `main contrib non-free non-free-firmware` 。

更新索引

```
sudo apt update
```

## 安装

安装对应头文件

```
sudo apt install linux-headers-$(uname -r)
```

1.  安装驱动和必要组件：
    ```bash
    sudo apt install nvidia-driver firmware-misc-nonfree
    ```
2.  **重启电脑**（非常重要，为了禁用 nouveau 并加载新驱动）：
    ```bash
    sudo reboot
    ```

---

### 验证安装
重启后，再次运行：
```bash
nvidia-smi
```
如果出现显卡参数表格，说明安装成功。

---

### 显卡切换 (Offloading)
在现代 Debian (12/13) 中，默认采用 **NVIDIA Prime Render Offload**。这意味着：
* **平时**：桌面环境由 AMD 集显驱动，节省电池。
* **按需**：你指定某个程序使用 NVIDIA 显卡运行。

#### 1. 命令行启动
如果你想用 NVIDIA 显卡运行某个程序（如游戏或 Blender），在命令前加上环境变量：
```bash
__NV_PRIME_RENDER_OFFLOAD=1 __GLX_VENDOR_LIBRARY_NAME=nvidia 你的程序名
```

#### 2. 设置别名（方便使用）
你可以把这串冗长的命令简化。编辑 `~/.zshrc`：
```bash
nano ~/.zshrc
```
在末尾添加：
```bash
alias nvrun='__NV_PRIME_RENDER_OFFLOAD=1 __GLX_VENDOR_LIBRARY_NAME=nvidia'
```
保存退出后运行 `source ~/.zshrc`。以后只需输入 `nvrun 程序名` 即可。

#### 3. 自动化安装（可选）
如果你觉得手动输入变量麻烦，可以安装 `nvidia-prime` 脚本：
```bash
sudo apt install nvidia-prime
```
安装后，直接使用 `prime-run 程序名` 即可达到同样效果。

---

### 注意事项
* **Secure Boot**：如果你的电脑开启了安全启动（Secure Boot），安装驱动后可能会因为内核模块没有签名而无法加载。建议在 BIOS 中暂时关闭 Secure Boot，或者配置 MOK 签名。
* **Docker 需求**：由于你之前提到需要部署 Docker，如果要在容器内调用这块 GTX 1650，你还需要安装 `nvidia-container-toolkit`。