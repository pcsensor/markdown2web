---
title: clutters
slug: clutters
summary: 各种问题......
tags:
- issues
- study
status: published
aliases: []
---
# 静默启动`wsl`
```
Start-Process wsl -ArgumentList "--exec sleep infinity" -WindowStyle Hidden
```

# Linux 根目录结构

| 目录 | 全拼 | 功能作用 |
|------|------|----------|
| `/bin` | **bin**aries | 基本用户命令二进制文件，如 `ls`、`cp`、`cat` |
| `/sbin` | **s**ystem **bin**aries | 系统管理命令，如 `fdisk`、`ifconfig`，通常需要 root |
| `/etc` | **et** **c**etera | 系统全局配置文件，如 `/etc/hosts`、`/etc/passwd` |
| `/home` | **home** | 普通用户的主目录，每个用户有独立子目录 |
| `/root` | **root** | root 超级用户的主目录 |
| `/var` | **var**iable | 可变数据，如日志、缓存、邮件队列、数据库文件 |
| `/tmp` | **t**e**mp**orary | 临时文件，重启后通常清空 |
| `/usr` | **U**nix **S**hared **R**esources | 用户级程序和数据，是最大的目录之一 |
| `/usr/bin` | — | 非必要用户命令，如 `git`、`python` |
| `/usr/local` | — | 本地手动安装的软件（不由包管理器管理） |
| `/lib` | **lib**raries | 系统启动和 `/bin`、`/sbin` 所需的共享库文件 |
| `/lib64` | **lib**raries **64**-bit | 64 位共享库 |
| `/dev` | **dev**ices | 设备文件，如 `/dev/sda`（硬盘）、`/dev/null` |
| `/proc` | **proc**esses | 虚拟文件系统，反映内核和进程的实时状态 |
| `/sys` | **sys**tem | 虚拟文件系统，暴露内核设备和驱动信息 |
| `/mnt` | **m**ou**nt** | 临时挂载点，手动挂载外部文件系统用 |
| `/media` | **media** | 自动挂载的可移动设备，如 U 盘、光盘 |
| `/opt` | **opt**ional | 第三方独立软件包，如商业软件 |
| `/boot` | **boot** | 启动引导文件，如内核 `vmlinuz`、`grub` |
| `/run` | **run**time | 系统运行时数据，如 PID 文件、socket，重启清空 |
| `/srv` | **s**e**rv**ices | 服务数据目录，如 Web 服务器的站点文件 |
| `/lost+found` | — | 文件系统修复时找回的碎片文件 |

## 记忆小结

```
/etc   → 配置
/var   → 动态数据（日志）
/usr   → 程序资源
/proc  → 内核实时信息
/dev   → 设备抽象
/tmp   → 临时
/home  → 用户家
```

# docker安装
## 下载官方安装脚本
```
curl -fsSL https://get.docker.com -o get-docker.sh
```

## 执行脚本并指定使用阿里云镜像源
```
sudo sh get-docker.sh --mirror Aliyun
```

# Ubuntu 远程桌面连接到 macOS

## 一、VNC安装设置

### 方法二：安装和配置 TigerVNC（更稳定、性能更好）

如果第一种方法效果不佳（比如卡顿、黑屏、无法连接），或者你需要更高级的配置，安装一个独立的 VNC 服务软件是更好的选择。TigerVNC 是一个性能优秀且广受欢迎的选择。

#### 步骤 1：在 Ubuntu 上安装 TigerVNC

```bash
sudo apt update
sudo apt install tigervnc-standalone-server tigervnc-xorg-extension -y
```

#### 步骤 2：设置 VNC 密码

第一次运行时，你需要为 VNC 连接设置一个专门的密码（这个密码独立于你的系统登录密码）。

```bash
vncpasswd
```

按照提示输入并验证密码。它会把加密后的密码保存在 `~/.vnc/passwd` 文件中。

#### 步骤 3：配置 VNC 启动脚本

你需要告诉 VNC 服务器启动时要加载哪个桌面环境。

1.  创建并编辑 `~/.vnc/xstartup` 文件：

    ```bash
    nano ~/.vnc/xstartup
    ```

2.  将以下内容粘贴进去。这是一个适用于标准 Ubuntu (GNOME) 桌面的配置。

    ```bash
    #!/bin/sh
    # unset SESSION_MANAGER
    # unset DBUS_SESSION_BUS_ADDRESS
    export XKL_XMODMAP_DISABLE=1
    export XDG_CURRENT_DESKTOP="GNOME"
    export XDG_SESSION_TYPE=x11
    dbus-launch gnome-session &
    ```

3.  保存并退出 (按 `Ctrl + X`，然后按 `Y`，最后按 `Enter`)。

4.  给这个文件添加执行权限：

    ```bash
    chmod +x ~/.vnc/xstartup
    ```

#### 步骤 4：启动 TigerVNC 服务器

执行以下命令来启动 VNC 服务：

```bash
vncserver -localhost no -geometry 1920x1080 -depth 24
```

  * `-localhost no`: **非常关键**。默认情况下 VNC 只允许本机连接，这个参数允许其他计算机（比如你的 Mac）从网络连接。
  * `-geometry 1920x1080`: 设置远程桌面的分辨率，你可以根据需要修改。
  * `-depth 24`: 设置颜色深度。

命令执行后，它会告诉你启动了一个新的桌面，通常是 `:1`。这意味着 VNC 服务运行在 `5901` 端口上 (端口号 = 5900 + 桌面号)。

#### 步骤 5：从 macOS 连接

连接方式和方法一几乎一样，只是现在可能需要指定端口号。

1.  打开 **访达 (Finder)** -\> **前往 (Go)** -\> **连接服务器 (Connect to Server...)** (`Command + K`)。
2.  输入服务器地址，并带上端口号：
    ```
    vnc://192.168.1.10:5901
    ```
3.  点击连接，并输入你在 **步骤 2** 中用 `vncpasswd` 命令设置的密码。

-----

### 常见问题排查 (Troubleshooting)

1.  **无法连接**：

      * **检查防火墙**：Ubuntu 的防火墙 (`ufw`) 可能阻止了 VNC 端口。你可以允许 VNC 端口的流量：
        ```bash
        # 允许所有 VNC 端口 (5900-5906)
        sudo ufw allow 5900:5906/tcp
        # 重新加载防火墙规则
        sudo ufw reload
        ```
      * **IP 地址错误**：确保你输入的 IP 地址是正确的，并且 Ubuntu 和 Mac 在同一个局域网内。

2.  **连接后黑屏或灰屏**：

      * 这通常是显示服务的问题。请务必确认你已经从 Wayland 切换到了 **Xorg** (参考方法一的步骤 2)。
      * 如果使用 TigerVNC，这通常是 `~/.vnc/xstartup` 脚本配置不正确导致的。

3.  **安全性提示**：

      * VNC 协议本身在数据传输加密方面比较弱。在可信的家庭或办公室内网中使用是安全的。
      * **绝对不要**在没有额外安全措施（如 VPN 或 SSH 隧道）的情况下，将 VNC 端口直接暴露在公网上。

总的来说，对于内网使用，**方法一** 已经足够满足需求，建议你先从它开始。

好的，要将 `vncserver` 命令设置为开机自启，在现代的 Linux 系统（如 Ubuntu）中，最推荐、最标准的方法是使用 `systemd` 来创建一个服务。

这种方法比传统的 `rc.local` 或 cron job (`@reboot`) 更加稳定和可管理。下面是详细的步骤。

我们将创建一个 `systemd` 的模板单元文件（template unit file），这样你就可以轻松地为不同的用户或不同的显示端口（如 `:1`, `:2`）启动服务。

-----

## 二、vmware启动虚拟机失败

根据你提供的 VMware 日志文件，虚拟机启动失败的**根本原因是虚拟磁盘文件已损坏**。

日志中明确指出了这个问题。

### 关键错误日志

1.  **磁盘需要修复**:

    ```
    DISKLIB-LINK : DiskLinkOpen: Failed to open 'D:\Ubuntu1804\Ubuntu1804.vmdk': : The specified virtual disk needs repair
    ```

    这行日志说明基础虚拟磁盘文件 `Ubuntu1804.vmdk` 已损坏，需要修复。

2.  **无法打开父磁盘**:

    ```
    DISKLIB-CHAIN : DiskChainOpen: "D:\Ubuntu1804\Ubuntu1804-000001.vmdk": Failed to open parent "D:\Ubuntu1804\Ubuntu1804.vmdk": The specified virtual disk needs repair
    ```

    这行日志表明，快照文件 `Ubuntu1804-000001.vmdk` 无法启动，因为它依赖的父磁盘（也就是上面那个基础磁盘）已损坏。

3.  **最终启动失败信息 (中文)**:

    ```
    Power on failure messages: 指定的虚拟磁盘需要进行修复
    打不开磁盘“D:\Ubuntu1804\Ubuntu1804-000001.vmdk”或它所依赖的某个快照磁盘。
    模块“Disk”启动失败。
    未能启动虚拟机。
    ```

    这是 VMware 在日志末尾汇总的启动失败原因，直接说明了问题所在：磁盘损坏导致“Disk”模块无法启动，最终虚拟机启动失败。

-----

### 如何解决 🔧

你需要使用 VMware Workstation 自带的磁盘管理工具 `vmware-vdiskmanager.exe` 来尝试修复这个损坏的磁盘。

1.  **以管理员身份打开命令提示符 (CMD)**。
2.  **切换到 VMware 的安装目录**。默认情况下，它通常在这里：
    ```cmd
    cd "C:\Program Files (x86)\VMware\VMware Workstation"
    ```
3.  **运行修复命令**。你需要指向损坏的 `.vmdk` 文件的完整路径。根据日志，应该先修复父磁盘 `Ubuntu1804.vmdk`。
    ```cmd
    vmware-vdiskmanager.exe -R "D:\Ubuntu1804\Ubuntu1804.vmdk"
    ```

执行命令后，程序会尝试修复磁盘文件。修复完成后，再尝试启动你的 Ubuntu 虚拟机。

---

# Cadence OrCAD X and Allegro X SPB 23.10的DRC不能正常使用的解决办法

![Cadence](Cadence.png)

在orCefSetting.ini文件 新增一行“lang=en-US”，即可。

# 终端光标更改问题

**对于竖线（线状）光标**：

```bash
    echo -ne "\\e[6 q"
```

**如果想切回块状光标**，可以尝试：

```bash
    echo -ne "\\e[2 q"
```

* `0`或`1`： 闪烁的块状（Block）
* `2`： 不闪烁的块状
* `3`： 闪烁的下划线（Underscore）
* `4`： 不闪烁的下划线
* `5`： 闪烁的竖线
* `6`： 不闪烁的竖线
---

# CLion远程开发

好的，这个方案是**在 Ubuntu 上设置 Samba (SMB) 文件共享，然后在 macOS 上挂载这个共享目录**。

这可以让你在 macOS 上的 CLion 直接编辑 Ubuntu 上的文件，省去 Git 同步的麻烦。

> **⚠️ 友情提示：** 正如你之前所说，这个方案**主要解决的是 Git 流程繁琐的问题**。但由于 CLion 仍然需要通过网络（Samba）来读取和索引文件，当项目很大时，IDE 的索引速度**可能仍然会感觉卡顿**（这取决于局域网的 I/O 性能）。

以下是具体配置步骤：

### 步骤一：在 Ubuntu 虚拟机上安装和配置 Samba

1.  **更新并安装 Samba：**
    打开你的 Ubuntu 终端：

    ```bash
    sudo apt update
    sudo apt install samba
    ```

2.  **创建（或确认）你要共享的目录：**
    假设你的所有项目都存放在 `~/projects` 目录（即 `/home/你的用户名/projects`）。如果它不存在，请创建它：

    ```bash
    mkdir -p ~/projects
    ```

    *(请将 `projects` 换成你实际的项目根目录)*

3.  **为 Samba 设置一个独立密码：**
    Samba 需要一个专门的密码库。你需要为你自己的 Ubuntu 用户（假设你的用户名是 `liu`）添加一个 Samba 密码：

    ```bash
    sudo smbpasswd -a liu
    ```

    *(请将 `liu` 换成你**真实的 Ubuntu 用户名**)*

    系统会提示你输入并确认一个新密码。为了方便记忆，你可以使用与你 Ubuntu 登录密码相同的密码。

4.  **配置 Samba 共享：**
    编辑 Samba 的配置文件：

    ```bash
    sudo nano /etc/samba/smb.conf
    ```

    在文件的**最底部**，添加以下内容（这是一个新的共享配置块）：

    ```ini
    [projects]
       comment = Ubuntu Project Files
       path = /home/liu/projects
       browseable = yes
       read only = no
       guest ok = no
       valid users = liu
       create mask = 0644
       directory mask = 0755
    ```

    **重要：**

      * `[projects]`：这是你共享的名称，一会儿 Mac 连接时会用到。
      * `path = /home/liu/projects`：**必须**将其中的 `liu` 和 `projects` 替换为你自己的实际用户名和项目路径。
      * `valid users = liu`：**必须**将其中的 `liu` 替换为你自己的用户名。

    编辑完成后，按 `Ctrl + X`，然后按 `Y`，最后按 `Enter` 保存并退出 `nano` 编辑器。

5.  **重启 Samba 服务：**
    为了让配置生效，重启 `smbd` 服务：

    ```bash
    sudo systemctl restart smbd
    ```

### 步骤二：获取 Ubuntu 虚拟机的 IP 地址

你需要知道虚拟机的 IP 地址，Mac 才能找到它。

```bash
ip addr show
```

在输出中找到你的主网卡（通常叫 `eth0` 或 `ens33` 之类的），查找 `inet` 后面的 IP 地址，例如 `192.168.1.105`。

### 步骤三：在 macOS 上连接到共享

1.  打开 **Finder**。

2.  点击屏幕顶部的菜单栏，选择 **"前往" (Go)** \> **"连接服务器..." (Connect to Server...)**。

      * 快捷键是 `Command + K`。

3.  在 "服务器地址" 栏中，输入：

    ```
    smb://192.168.1.105/projects
    ```

      * **重要：** 将 `192.168.1.105` 替换为你在步骤二中找到的 **Ubuntu IP 地址**。
      * `projects` 是你在 `smb.conf` 中设置的共享名称 `[projects]`。

4.  点击 "连接"。系统会提示你输入凭据。

      * **名称：** 你的 Ubuntu 用户名（例如 `liu`）。
      * **密码：** 你在步骤一的第 3 点中设置的 **Samba 密码**。
      * (建议勾选 "在我的钥匙串中记住此密码")

5.  连接成功后，这个 "projects" 共享目录会像一个U盘一样挂载到你的 Mac 上（你可以在 Finder 的侧边栏 "位置" 下看到它）。

### 步骤四：你的新工作流

1.  **在 Mac 上编码：**
      * 打开 macOS 上的 CLion。
      * 选择 "Open" (打开项目)。
      * 在文件选择器中，导航到你刚刚挂载的网络驱动器 (projects)，选择你的项目文件夹。
      * 现在你可以像在本地一样编辑代码了，文件会实时保存在 Ubuntu 虚拟机上。
2.  **在 Ubuntu 上编译：**
      * 单独打开一个 SSH 终端（例如 Mac 自带的 "终端" App）。
      * `ssh liu@192.168.1.105` 登录到你的虚拟机。
      * `cd` 到你的项目目录，然后照常执行 `cmake`, `make` 或运行你的程序。

# iStoreOS关于v2raya的问题

## geoip和geosite

[v2ray-rules-dat](https://github.com/Loyalsoldier/v2ray-rules-dat/releases)

```
# 创建目录（如果它不存在的话）
mkdir -p /usr/share/xray/

# 复制文件
cp /tmp/geoip.dat /usr/share/xray/
cp /tmp/geosite.dat /usr/share/xray/
```

## xray-core更换v2ray-core

我们需要用 iStoreOS (OpenWrt) 的方式来配置。

-----

### ⭐️ iStoreOS/OpenWrt 切换核心的正确步骤

请你通过 SSH 登录到你的 iStoreOS 路由器，然后一步一步执行以下命令：

#### 步骤一：安装 `v2fly-core`

首先，我们需要安装 `v2fly-core`。在 OpenWrt 的软件源中，它通常被称为 `v2ray-core`。

1.  **更新软件列表**：

    ```bash
    opkg update
    ```

2.  **安装 v2ray-core**：

    ```bash
    opkg install v2ray-core
    ```

    *（如果你已经安装了，它会提示你已安装，这没问题。）*

#### 步骤二：查找 `v2ray-core` 的安装路径

我们需要知道 `v2ray-core` 被安装到了哪里。

1.  **运行 `which` 命令**：

    ```bash
    which v2ray
    ```

2.  **记下这个路径**。它很可能返回 `/usr/bin/v2ray`。在下面的步骤中，我们将使用这个路径。

#### 步骤三：修改 v2rayA 配置，强制使用新核心

这是最关键的一步。我们将使用 `uci` 命令（OpenWrt 的统一配置接口）来修改 v2rayA 的配置文件，告诉它 `v2ray` 核心的准确位置。

1.  **执行 `uci set` 命令**：
    （请注意：把下面命令中的 `/usr/bin/v2ray` 替换为你**上一步找到的实际路径**）

    ```bash
    uci set v2raya.config.v2ray_bin='/usr/bin/v2ray'
    ```

2.  **保存配置**：

    ```bash
    uci commit v2raya
    ```

#### 步骤四：重启 v2rayA 服务

让配置生效。

```bash
/etc/init.d/v2raya restart
```

-----

### 步骤五：验证

现在，你已经成功将 v2rayA 的后端从（很可能是）`xray-core` 切换到了 `v2fly-core`。

1.  **刷新**你的 v2rayA 网页管理界面。
2.  现在，你应该可以**按住 `Ctrl` 键 (Windows) 或 `Cmd` 键 (Mac)**，然后**用鼠标左键点击节点的名字**（例如 "🇭🇰 Hong Kong | 01"、"🇭🇰 Hong Kong | 02" 等）。
3.  你会发现这些节点的名字（或整行）**会被高亮选中**。
4.  当你选中了所有你想要的节点后，点击顶部的“启动”按钮。

这样，v2rayA 就会在你所有选中的节点中自动进行负载均衡，连接延迟最低的那个了。

# Mac mini设置自动开机

1. 设置每天 5:35 自动开机

```
sudo pmset -a repeat poweron MTWRFSU 05:35:00
```

2. 设置来电自启（非正常关机）

```
sudo pmset -a autorestart 1
```

3. 检查设置是否生效

```
pmset -g sched
```

4. 关闭所有电源事件

```
sudo pmset repeat cancel
```

# vulkan问题

## 查看“真·后台”到底在用什么

光看属性（getprop）只是看“配置单”，我们要看“厨房里实际在做什么菜”。
我们可以通过查看**设置 (Settings)** 应用的渲染管线来判断。

在 Termux (Root) 下输入：

```bash
dumpsys gfxinfo com.android.settings | grep Pipeline
```

**请查看输出结果：**

  * **情况 A：** `Pipeline=Skia (OpenGL)`
      * **结论**：目前依然是 OpenGL，修改未生效。
  * **情况 B：** `Pipeline=Skia (Vulkan)`
      * **结论**：**恭喜！** 即使属性显示不对，或者你之前以为没成功，实际上系统 UI 已经在用 Vulkan 跑了。

-----

### 补充：如果我想让它永久生效且显示 skiavk？

如果你刚才测试 `resetprop` 后 `getprop` 变成了 `skiavk`，但一重启就变回 `hwui`，你需要把它做成一个 **Magisk 启动脚本**。

在 Termux (Root) 下直接复制粘贴这整段代码：

```bash
# 创建一个开机脚本
cat <<EOF > /data/adb/service.d/force_vulkan.sh
#!/system/bin/sh
resetprop debug.hwui.renderer skiavk
resetprop renderthread.skia.reduceopstasksplitting true
EOF

# 给予执行权限
chmod +x /data/adb/service.d/force_vulkan.sh
```

执行完上面这段后，**重启手机**。

运行以下命令查看是否成功

```bash
getprop debug.hwui.renderer

dumpsys gfxinfo com.android.systemui | grep Pipeline
```

> [!NOTE]
> 刷入更完善健全的模块更好

# 手机调度

walt

CPU 调速器（如 walt 或 schedutil）的核心工作，就是根据**负载**来调整**频率**，从而提供恰到好处的**算力**。

[调速器参数定制-Gemini问答](https://gemini.google.com/share/c10901e25ad2)

```
# 查看调速器
cat /sys/devices/system/cpu/cpufreq/policy*/scaling_governor

# 查看参数
cat /sys/devices/system/cpu/cpufreq/policy*/walt/*
```

# macOS启动openlist

[脚本文件](./macOS_openlist_fish.fish)

# wsl配置网络

编辑`C:\Users\<你的用户名>\.wslconfig`（没有就新建）

```
[wsl2]
networkingMode=mirrored          # 关键：开启镜像网络模式
dnsTunneling=true                # 让 WSL 能正确解析 windows 的本地 hosts 和域名
ipv6=true                        # 可选，如果你用 ipv6
localhostForwarding=true         # 兼容旧版，确保 localhost 一定通
```

# 海外测速

```
aria2c -x 16 -s 16 http://proof.ovh.net/files/10Gb.dat
```

# macOS打开软件显示损坏

因签名问题：

```
sudo xattr -rd com.apple.quarantine /Applications/example.app
```

要理解这个命令的原理，我们需要从 macOS 的底层文件系统和安全机制说起。

简单来说，这个命令的作用是撕掉 macOS 贴在这个软件上的“来自互联网的未知物品”标签。

下面详细拆解这个机制和命令背后的逻辑：

1. 核心机制：扩展属性 (Extended Attributes) 与 Gatekeeper

在 macOS 的 APFS（以及大多数现代文件系统如 Linux 的 ext4）中，文件除了包含本身的数据代码外，文件系统还允许给它附加一些额外的“元数据”（Metadata），这就叫做扩展属性 (Extended Attributes)。

通过浏览器（比如你截图提示中的 Vivaldi）或任何网络工具从互联网下载文件时，macOS 的下载服务会自动给这个文件写入一个特定的扩展属性标签：com.apple.quarantine（隔离属性）。

当你尝试双击运行一个带有 com.apple.quarantine 标签的 .app 时，macOS 的安全守护神 Gatekeeper 就会被触发。它会强行检查这个应用：

是否有 Apple 官方颁发的有效开发者签名？

应用代码的哈希值是否与签名匹配（证明没被篡改）？

是否经过了 Apple 的云端公证（Notarization）？

如果 ishellpro 没有缴纳“苹果税”获取合规的签名，或者它本身就是一个开源/小众项目，Gatekeeper 检查不通过，就会直接将其拦截。

2. 命令逐字拆解

执行的完整命令是：sudo xattr -rd com.apple.quarantine /Applications/ishellpro.app。它的每个参数都在执行极其精确的文件系统操作：

sudo：以超级管理员（Root）权限执行。由于 /Applications 目录下的应用文件可能包含不同级别的权限控制，修改其底层属性通常需要提权。

xattr：macOS 中用于读取、写入或删除文件扩展属性的专用命令行工具。

-r (recursive)：递归操作。macOS 里的 .app 表面上看是一个单一的图标，但在文件系统层面，它其实是一个包含大量可执行文件、动态链接库和资源文件的目录（App Bundle）。-r 参数确保隔离标签从 .app 根目录及其内部嵌套的每一个子文件上被彻底抹除。

-d (delete)：指示 xattr 工具执行“删除特定属性”的操作。

com.apple.quarantine：目标属性的名称，即我们要定点清除的“隔离”标签。

/Applications/ishellpro.app：目标路径。

3. 为什么系统会谎称软件“已损坏”？

这是一个非常有意思的现象，其实 DMG 里的文件本身完好无损。macOS 提示“已损坏，无法打开”，是 Apple 故意设计的一种防御性 UX（用户体验）策略。

为了防止普通用户被诱导运行恶意软件或勒索病毒，系统选择用“已损坏”这种极具劝退效果的措辞，并且隐藏了“强制打开”的按钮。这不仅保护了系统安全，客观上也迫使用户更倾向于在 Mac App Store 下载软件，或倒逼开发者去购买 Apple 的开发者证书。

当你用 xattr 删除了 quarantine 标签后，Gatekeeper 就会判定这个软件是你“在本地自己创建或编译”的，不再将其视为外来的高风险物品，因此直接放行。