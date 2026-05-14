---
title: archinstall 前的分区
slug: archinstall-前的分区
summary: ''
category: []
tags: []
status: published
updated: 2026-05-13T18:06
aliases: []
---
## 方案分析

你的方案有一些问题，先说清楚再给推荐方案。

### 你的方案的问题

```
/dev/sdaX  FAT32  1GB   →  /boot        ← 自建
/dev/sdaY  FAT32  ?     →  /boot/efi    ← Windows ESP
/dev/sdaZ  btrfs  ?GB   →  /            ← 根分区
```

**问题：**
- `/boot` 是 FAT32，Linux 内核、initramfs 都放这里，但 FAT32 **不支持软链接**，`mkinitcpio` 和部分工具会出问题
- `/boot` 下再挂载 `/boot/efi`，层级混乱，GRUB 安装路径容易搞错
- 单独建 1GB FAT32 的 `/boot` 意义不大，这是 XBOOTLDR 的变体用法，但你用的是 GRUB 的话没必要

---

## 推荐方案

直接复用 Windows 的 ESP，不新建 `/boot` 分区：

```
Windows ESP  FAT32  (已有)  →  /boot/efi    ← 共享 ESP，只写入 Arch 的引导文件
/dev/sdaZ    btrfs  ?GB    →  /            ← 根分区（含 btrfs 子卷）
```

**优点：**
- `/boot` 在 btrfs 根分区内（普通目录），无格式限制
- ESP 只放 EFI 引导文件，不碰 Windows 的文件
- GRUB 自动探测 Windows，`os-prober` 一键生成双启动菜单

---

## 具体操作步骤

### 第一步：确认现有分区

```bash
lsblk -f
fdisk -l
```

找到 Windows 的 ESP（通常 100~500MB，FAT32，类型 `EFI System`），记下设备名，例如 `/dev/sda1`。

---

### 第二步：创建并格式化 btrfs 根分区

```bash
# 将空闲空间创建为新分区（假设空闲在 sda 上）
cfdisk /dev/sda
# 新建分区，类型选 Linux filesystem，写入退出

# 格式化为 btrfs
mkfs.btrfs -L "arch" /dev/sdaX    # 替换为实际分区名
```

---

### 第三步：创建 btrfs 子卷（推荐结构）

```bash
# 先挂载顶层
mount /dev/sdaX /mnt

# 创建子卷
btrfs subvolume create /mnt/@
btrfs subvolume create /mnt/@home
btrfs subvolume create /mnt/@snapshots
btrfs subvolume create /mnt/@var_log

# 卸载顶层
umount /mnt
```

---

### 第四步：按子卷重新挂载

```bash
# 挂载选项（SSD 用 ssd，HDD 去掉）
OPTS="rw,noatime,compress=zstd:3,space_cache=v2,ssd,discard=async"

mount -o ${OPTS},subvol=@ /dev/sdaX /mnt

mkdir -p /mnt/{home,.snapshots,var/log,boot/efi}

mount -o ${OPTS},subvol=@home      /dev/sdaX /mnt/home
mount -o ${OPTS},subvol=@snapshots /dev/sdaX /mnt/.snapshots
mount -o ${OPTS},subvol=@var_log   /dev/sdaX /mnt/var/log

# 挂载 Windows ESP（只读风险小，但安装 GRUB 时需要读写）
mount /dev/sda1 /mnt/boot/efi
```

---

### 第五步：用 archinstall 安装

```bash
archinstall
```

关键选项：
```
Disk configuration
  └─> Use a pre-mounted configuration
        └─> /mnt

Bootloader → GRUB
```

其余正常配置（用户、时区、桌面环境等）。

---

### 第六步：安装后配置 GRUB 探测 Windows

进入 chroot 后（archinstall 会自动进，或手动 `arch-chroot /mnt`）：

```bash
# 安装 os-prober（探测 Windows）
pacman -S os-prober

# 编辑 GRUB 配置，启用 os-prober
nano /etc/default/grub
# 找到这行，取消注释：
# GRUB_DISABLE_OS_PROBER=false

# 重新生成 GRUB 配置
grub-mkconfig -o /boot/grub/grub.cfg
```

输出中应该看到类似：
```
Found Windows Boot Manager on /dev/sda1
```

---

### 最终分区结构

| 分区 | 文件系统 | 挂载点 | 说明 |
|---|---|---|---|
| `/dev/sda1` | FAT32 | `/boot/efi` | Windows ESP，共享 |
| `/dev/sdaX` | btrfs `@` | `/` | Arch 根 |
| `/dev/sdaX` | btrfs `@home` | `/home` | |
| `/dev/sdaX` | btrfs `@snapshots` | `/.snapshots` | Snapper 快照用 |
| `/dev/sdaX` | btrfs `@var_log` | `/var/log` | 避免日志影响快照回滚 |

---

### 注意事项

- **不要动 Windows ESP 里的 `EFI/Microsoft/` 目录**，GRUB 只会写入 `EFI/GRUB/`
- 如果 Windows 更新后启动项消失，进 BIOS/UEFI 把 GRUB 调回第一启动项即可
- 建议装完后用 `btrfs-assistant` 或 `snapper` 配置快照策略