---
title: debian系统KDE桌面下微信缩放异常问题
slug: debian系统kde桌面下微信缩放异常问题
summary: ''
category: []
tags: []
status: published
updated: 2026-04-21T09:44
aliases: []
---
## 编辑.desktop文件

```
cat /usr/share/applications/wechat.desktop

编辑这一行设置环境变量
Exec=env QT_SCALE_FACTOR=1.5 /usr/bin/wechat %U
```

## 刷新KDE桌面缓存

```
kbuildsycoca6 --noincremental
```