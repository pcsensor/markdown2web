---
title: Arch Btrfs 快照与回滚指南
slug: arch-btrfs-快照与回滚指南
summary: 没啥用，越来越讨厌折腾了，还是回归macOS的怀抱吧
category: []
tags: []
status: published
updated: 2026-05-13T17:59
aliases: []
---
### 🌟 方案工作原理总结

这套方案由四个核心部分精密配合，构建出一套完美的“时光倒流”机制：

1. **Btrfs 结合 Snapper**：利用写时复制（CoW）的特性瞬间定格系统状态。Snapper 作为管家，负责按时间线自动创建快照，并根据配置策略（如保留最近 7 天、近 4 周）自动清理旧快照，防止磁盘爆满。
2. **Pacman 钩子集成**：通过 `snap-pac` 插件，在每次安装、更新或卸载软件前后自动触发快照。一旦系统“滚挂”，你随时有刚更新前的恢复点。
3. **GRUB 菜单动态集成**：`grub-btrfsd` 守护进程会监听快照变动，只要生成新快照，它就立刻将其作为启动项塞入 GRUB 的 `Arch Linux snapshots` 子菜单中。
4. **Overlayfs 内存叠加层（神级魔法）**：为了保证数据安全，Snapper 的快照都是**只读**的。如果强行以只读模式启动系统，桌面环境往往会因为无法写入日志和临时文件而崩溃。通过在 `mkinitcpio` 中打入 `grub-btrfs-overlayfs` 钩子，系统会在只读快照上叠加一个存在于内存中的“可写层”。你在快照系统里的任何修改都会留在内存里（重启即焚），让你能像正常系统一样进入图形界面进行排错和数据备份。

---

### 🛠️ 详细配置流程与原理解析

前提条件：本指南基于 Archinstall 风格的子卷布局（根目录为 `@`，家目录为 `@home`，快照为独立的 `@snapshots` 子卷）。

#### 第 1 步：建立物理防线（创建应急手工快照）

在动手配置复杂工具前，先打个底。

```bash
sudo mount -o subvolid=5,rw,noatime,compress=zstd:3,ssd,discard=async,space_cache=v2 /dev/nvme0n1p6 /mnt
sudo btrfs subvolume snapshot -r /mnt/@ "/mnt/@manual-pre-snapper-$(date +%F-%H%M)"
sudo umount /mnt

```

* **操作含义**：将 Btrfs 的最顶层（ID 5）临时挂载到 `/mnt`，然后给当前的根子卷 `@` 纯手工拍一个只读快照。这个快照不归属 Snapper 管理，就算后续配置把 Snapper 玩炸了，通过 LiveUSB 也能用这个快照将系统救回当前状态。

#### 第 2 步：安装快照核心组件

```bash
sudo pacman -S --needed snapper snap-pac grub-btrfs inotify-tools btrfs-assistant

```

* **操作含义**：安装全套工具链。`snapper` 管理快照；`snap-pac` 绑定包管理器；`grub-btrfs` 提供 GRUB 集成；`inotify-tools` 供守护进程监听文件变化；`btrfs-assistant` 则是方便你后续使用鼠标管理快照的 GUI 神器。

#### 第 3 步：初始化 Snapper 并“偷梁换柱”修复嵌套

这一步极其关键，用来适配你现有的子卷布局。

```bash
# 1. 临时移除现有的快照挂载
sudo umount /.snapshots
sudo rmdir /.snapshots

# 2. 生成 Snapper 根配置
sudo snapper -c root create-config /

# 3. 删掉 Snapper 自动建的嵌套子卷，换回真正的独立子卷
sudo btrfs subvolume delete /.snapshots
sudo mkdir /.snapshots
sudo mount /.snapshots

```

* **操作含义**：Snapper 默认极其“霸道”，它会在你的 `@` 子卷内部强行创建一个名为 `@/.snapshots` 的嵌套子卷。如果放任不管，不仅违背了我们平铺子卷的初衷，后续回滚时还会导致快照丢失。这一步操作先让 Snapper 建立配置文件，然后立刻斩杀它建的嵌套子卷，把我们在 `fstab` 里写好的独立 `@snapshots` 挂载回来。

#### 第 4 步：写入推荐的保留策略与权限配置

考虑到 `fish` 终端对部分语法解析的特殊性，使用最稳妥的命令组合。

```bash
# 修改权限和普通保留数量
sudo sed -i \
  -e 's/^ALLOW_GROUPS=.*/ALLOW_GROUPS="wheel"/' \
  -e 's/^SYNC_ACL=.*/SYNC_ACL="yes"/' \
  -e 's/^NUMBER_LIMIT=.*/NUMBER_LIMIT="30"/' \
  -e 's/^NUMBER_LIMIT_IMPORTANT=.*/NUMBER_LIMIT_IMPORTANT="10"/' \
  /etc/snapper/configs/root

# 追加时间线保留策略
sudo sh -c 'printf "%s\n" \
"TIMELINE_LIMIT_HOURLY=\"12\"" \
"TIMELINE_LIMIT_DAILY=\"7\"" \
"TIMELINE_LIMIT_WEEKLY=\"4\"" \
"TIMELINE_LIMIT_MONTHLY=\"3\"" \
"TIMELINE_LIMIT_YEARLY=\"0\"" >> /etc/snapper/configs/root'

```

* **操作含义**：赋予 `wheel` 用户组管理快照的权限（方便日常无密码查看）；开启 ACL 同步。设置每次更新软件时的快照最多保留 30 个；并开启一套非常适合桌面端的高密度时间线：保留近 12 小时、近 7 天、近 4 周和近 3 个月的快照，更久远的自动抛弃，平衡安全与存储空间。

#### 第 5 步：修正目录权限并创建基准快照

```bash
sudo chown root:root /.snapshots
sudo chmod 755 /.snapshots
sudo snapper -c root create -d "initial root snapshot"

```

* **操作含义**：加固挂载点的安全性，防止普通用户误触底层的快照数据。随后拍下配置彻底完成后的第一张基准快照。

#### 第 6 步：激活自动化定时服务

```bash
sudo systemctl enable --now snapper-timeline.timer
sudo systemctl enable --now snapper-cleanup.timer
sudo systemctl enable --now grub-btrfsd.service

```

* **操作含义**：让 Systemd 接管日常工作。`timeline` 负责按小时打快照；`cleanup` 负责无情清理超限的旧快照；`grub-btrfsd` 则是幕后哨兵，一旦发现 `/.snapshots` 里多了一个快照，瞬间静默刷新你的 GRUB 启动菜单，无需手动 `grub-mkconfig`。

#### 第 7 步：注入 Overlayfs 使快照在启动时“可写”

```bash
# 备份配置
sudo cp /etc/mkinitcpio.conf /etc/mkinitcpio.conf.bak.$(date +%F-%H%M)
# 将 grub-btrfs-overlayfs 自动追加到 HOOKS 列表的末尾
sudo sed -i 's/^HOOKS=(\(.*\))$/HOOKS=(\1 grub-btrfs-overlayfs)/' /etc/mkinitcpio.conf
# 重新打包系统引导镜像
sudo mkinitcpio -P

```

* **操作含义**：将核心的内存读写叠加层驱动打入 Arch 的引导微内核（initramfs）中。这样以后在 GRUB 里选择从某一个快照启动时，系统就不会再报 "Read-only file system" 的错误了。

#### 第 8 步：更新 GRUB 菜单并生成最终测试快照

```bash
sudo grub-mkconfig -o /boot/grub/grub.cfg
sudo snapper -c root create -d "overlayfs-ready test snapshot"

```

* **操作含义**：初始化 GRUB 菜单中的 `Arch Linux snapshots` 模块。**注意：** 因为上一步重构了 initramfs，只有在那之后生成的快照，才能真正携带并享受 Overlayfs 的读写魔法。所以这里特地创建了一个包含最新 initramfs 的最终测试快照。

---

### 🔙 灾难恢复篇：如何真正进行回滚

当你的 Arch 系统因为某次激进的滚挂或者误删文件而无法正常使用时，你需要进行恢复。请严格区分以下两种场景：

#### 场景 A：我想先确认情况，抢救数据（临时救援启动）

1. 重启电脑，在引导出现时进入 GRUB 菜单。
2. 选择 **Arch Linux snapshots**。
3. 选择一个出问题前的快照时间点（比如 `overlayfs-ready test snapshot`）。
4. 系统会如同往常一样顺滑地启动进桌面环境。
* **此时的机制**：你正在运行于一个由 Overlayfs 支撑的临时维度。你可以复制数据到 U 盘，可以查看配置文件到底错在哪，但你**所有的修改重启后都会消失**，原快照依然是只读且纯洁的。



#### 场景 B：确定了某个快照没问题，我要彻底时光倒流（永久回滚）

```bash
# 挂顶层
mount -o subvolid=5 /dev/nvme0n1p6 /mnt

# 删除旧@
mv /mnt/@ /mnt/@.broken

# 创建新@
btrfs subvolume snapshot /mnt/@snapshots/120/snapshot /mnt/@

# 卸载
umount /mnt

# 挂真正根
mount -o subvol=@ /dev/nvme0n1p6 /mnt

# EFI
mkdir -p /mnt/boot/efi
mount /dev/nvme0n1p1 /mnt/boot/efi

# bind
for i in dev proc sys run; do
    mount --bind /$i /mnt/$i
done

# chroot
arch-chroot /mnt

# rebuild
mkinitcpio -P
grub-mkconfig -o /boot/grub/grub.cfg
```