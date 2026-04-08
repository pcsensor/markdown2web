# markdown2web 产品与技术方案（PRD）

- 日期：2026-04-08
- 模式：$plan（direct）
- 当前仓库状态：仓库基本为空，尚未开始实现；以下方案同时作为初始项目结构建议。

## 1. 需求摘要

构建一个“Markdown 笔记 -> 网站展示”的 Rust 全栈项目，核心要求：

1. 使用 **Rust** 实现网站后台与管理端。
2. 支持 **动态上传、更新 Markdown 笔记文件**。
3. 网站整体风格 **简洁、现代、结构清晰**。
4. 理想状态下，当笔记目录中的文件变化时，系统可 **自动增量构建并更新网页**。
5. 笔记内部的 **互相跳转链接**，在网页端也能正常工作。
6. 笔记里的 **图片与其他资源** 能正确嵌入网页。
7. 自动补足事前没考虑到的事项，并提前规避典型设计漏洞。

## 2. 建议产品定位

建议将项目定位为：

> “带后台管理能力的个人知识库 / 数字花园（Digital Garden）发布系统”

它既不是纯静态站点生成器，也不是重 CMS，而是：

- **文件为主**：Markdown 文件和附件放在内容目录中，便于长期维护与迁移。
- **后台可控**：通过管理后台上传、编辑、预览、发布。
- **自动构建**：文件变化后增量更新站点。
- **网站友好**：支持目录、标签、反向链接、搜索、资源嵌入。

## 3. 关键决策（ADR）

### 决策

采用 **Rust 单体应用 + 文件系统为内容真源 + SQLite 作为索引/后台元数据 + 增量构建管线** 的方案。

### 关键驱动因素

1. 用户明确要求 Rust 作为后台主技术。
2. Markdown 笔记天然适合保存在磁盘，而不是只存数据库。
3. “上传更新”和“监听文件夹变更”必须并存，不能让两套数据源互相打架。
4. 网站展示强调清晰、现代、轻量，不需要一上来就做 SPA。
5. 需要在“长期可维护性”和“首版可落地性”之间取平衡。

### 备选方案

#### 方案 A：文件系统为真源 + SQLite 做索引与后台数据（推荐）

优点：
- 最契合 Markdown 内容管理习惯。
- 笔记可被外部编辑器直接维护。
- 后台上传与文件夹监听可以统一落盘到同一内容目录。
- 更容易做增量构建、资源拷贝、Git 备份。

缺点：
- 需要处理文件监听抖动、路径规范、并发写入。

#### 方案 B：数据库为真源，Markdown 只作为导入导出格式

优点：
- 后台 CRUD 设计直接。
- 多用户协作扩展更自然。

缺点：
- 与“笔记文件夹自动监听更新”的目标冲突大。
- 文件与数据库容易出现双向同步复杂度。
- 迁移性和可读性更差。

### 选择原因

选择方案 A，因为它最符合“笔记目录变化 -> 自动更新网页”的核心诉求，也最能同时满足“外部编辑”和“后台上传”两条内容流。

### 后果与后续

- 首版必须优先解决 **文件路径规范、链接解析、资源映射、增量构建**。
- 若后续有多用户协作需求，可在 SQLite 元数据层之上扩展权限与审计。

## 4. 核心假设与边界

### 首版假设

1. 系统以 **单管理员 / 小团队内容维护** 为主，不做复杂多人协同冲突编辑。
2. 内容主目录可由系统写入，也允许外部编辑器修改。
3. 网站以 **SSR + 少量前端增强** 为主，而不是前后端完全分离 SPA。
4. 图片、附件、PDF 等静态资源都放在内容目录或其附件目录下。
5. 首版支持中文内容、常见 Markdown 能力、基础搜索与分类。

### 非目标（首版先不做）

- 实时协同编辑
- 多租户
- 复杂工作流审批
- 大规模对象存储抽象（如首版直上 S3）
- 超复杂所见即所得编辑器

## 5. 推荐技术方案

## 5.1 后端与渲染

- **Web 框架**：`axum`
- **异步运行时**：`tokio`
- **模板引擎**：`askama`
- **数据库**：`SQLite`（建议配合 `sqlx`）
- **认证**：Cookie Session + Argon2 密码散列
- **Markdown 解析**：`comrak` 或 `pulldown-cmark` + 自定义链接/资源重写
- **文件监听**：`notify`
- **序列化**：`serde`
- **Front Matter**：YAML / TOML 二选一，建议 YAML

推荐理由：

- `axum` + `askama` 足够现代、清晰、可维护，适合 SSR 内容站。
- SSR 可以天然满足 SEO、首屏速度、目录页面与笔记页展示。
- 少量交互（上传、预览、搜索建议、主题切换）再用轻 JS 或 HTMX/Alpine 补足。

## 5.2 内容与构建模型

### 真源目录

```text
content/
  notes/
  assets/
```

### 系统衍生目录

```text
generated/
  site/
  search/
  cache/
data/
  app.db
```

### 真源原则

- `content/notes` 与 `content/assets` 是 **内容真源**
- `SQLite` 保存：
  - 用户信息
  - 构建状态
  - 笔记索引
  - 链接图
  - 搜索索引元数据
  - 审计日志
- `generated/` 是构建产物，不是手工编辑源

这能避免“数据库内容”和“文件系统内容”双真源冲突。

## 5.3 推荐项目结构（首版）

```text
Cargo.toml
src/
  main.rs
  app.rs
  config.rs
  error.rs
  web/
    mod.rs
    public.rs
    admin.rs
    auth.rs
  content/
    mod.rs
    front_matter.rs
    markdown.rs
    links.rs
    assets.rs
    graph.rs
  build/
    mod.rs
    pipeline.rs
    watcher.rs
    cache.rs
  store/
    mod.rs
    sqlite.rs
    filesystem.rs
  search/
    mod.rs
    index.rs
templates/
  base.html
  home.html
  note.html
  notes.html
  tag.html
  admin/
    login.html
    dashboard.html
    note_edit.html
static/
  css/
    app.css
  js/
    admin.js
content/
  notes/
  assets/
generated/
data/
tests/
  integration/
```

说明：

- 首版建议保持 **单仓 Rust 单体**，不要一开始就拆成多 crate。
- 当内容管线和后台逻辑明显膨胀后，再拆 crate。

## 6. 功能范围设计

## 6.1 管理后台

必须具备：

1. 管理员登录
2. 上传 Markdown 文件
3. 上传图片/附件
4. 新建与更新笔记
5. 删除/归档笔记
6. 构建状态展示
7. 预览页面
8. 手动触发全量重建

建议补充：

9. 草稿 / 已发布 状态
10. 标签管理
11. 别名（alias）管理
12. 链接校验报告
13. 最近构建日志

## 6.2 公共站点

必须具备：

1. 首页 / 简介页
2. 笔记列表页
3. 单篇笔记页
4. 标签页 / 分类页
5. 上下篇或相关推荐
6. 正文目录（TOC）
7. 站内跳转链接
8. 图片/附件展示

建议补充：

9. 反向链接（Backlinks）
10. 全文搜索
11. 主题切换（浅色 / 深色）
12. sitemap.xml / RSS
13. 404 页面

## 6.3 Markdown 能力

首版建议支持：

- 标题、列表、引用、表格、代码块
- Front Matter
- 标准 Markdown 链接
- 相对路径链接
- 图片内嵌
- 附件下载链接
- 代码高亮
- 标题锚点

建议增强：

- `[[Wiki Link]]` 风格链接
- `![[image.png]]` 风格资源嵌入
- 数学公式（后续可选）
- Mermaid（后续可选）

## 7. 内容模型

## 7.1 笔记实体

建议字段：

- `id`
- `slug`
- `title`
- `source_path`
- `summary`
- `tags`
- `created_at`
- `updated_at`
- `published_at`
- `status`（draft/published/archived）
- `html_cache`
- `toc`
- `outbound_links`
- `inbound_links`
- `asset_refs`
- `hash`

## 7.2 Front Matter 建议

```yaml
title: Rust 后端设计
slug: rust-backend-design
summary: 项目后台模块设计说明
tags: [rust, backend]
status: published
aliases: [backend-design]
order: 10
```

## 8. 链接与资源处理设计

## 8.1 链接解析

必须同时兼容两类跳转：

1. 标准 Markdown/相对路径链接  
   例如：`[详情](../notes/design.md)`

2. Wiki Link（建议支持）  
   例如：`[[design-note]]`

构建阶段完成：

- 解析目标笔记
- 建立 note graph
- 统一生成站点 URL
- 记录反向链接
- 产出断链报告

## 8.2 路径策略

建议网页 URL 统一为：

```text
/notes/{slug}
```

不要直接把源文件路径暴露为最终 URL，避免目录调整后链接全部失效。

## 8.3 资源策略

图片与附件建议支持两种来源：

1. `content/assets/**`
2. 笔记旁路资源（如 `content/notes/foo/image.png`）

构建时：

- 复制资源到 `generated/site/assets/...`
- 重写页面中的资源 URL
- 记录资源 hash（避免重复复制）
- 检测丢失资源并输出告警

## 9. 自动构建方案

## 9.1 触发源

触发构建的来源：

1. 后台上传 Markdown
2. 后台修改 Markdown
3. 外部编辑器修改 `content/notes`
4. 外部新增/删除资源文件
5. 管理员手动触发全量构建

## 9.2 构建模式

建议采用：

- **默认增量构建**
- **必要时全量重建**

增量构建流程：

1. 监听文件变化
2. 做 debounce（避免短时间内重复触发）
3. 找到受影响的 note / asset
4. 重新解析变更项
5. 更新链接图
6. 重建当前页与受影响反向链接页
7. 更新搜索索引、sitemap、列表页缓存

## 9.3 关键防坑

必须规避：

1. **构建风暴**：保存一次触发多次监听事件  
   - 解决：debounce + build queue

2. **双写竞争**：后台写文件与 watcher 同时处理  
   - 解决：统一写入口 + 变更事件标记 + 幂等构建

3. **脏缓存**：笔记改了但反向链接页没更新  
   - 解决：维护 inbound/outbound graph

4. **路径漂移**：文件改名后旧链接失效  
   - 解决：slug 稳定化 + alias/redirect 机制

## 10. UI/UX 设计方向

整体风格建议：

- 极简留白
- 清晰层级
- 内容优先
- 现代排版
- 弱装饰、强可读性

建议公共站点布局：

- 顶部：站点名 / 搜索 / 主题切换
- 左侧或顶部：导航、标签入口
- 主内容区：正文 + TOC
- 右侧或底部：反向链接 / 相关文章 / 更新时间

建议后台布局：

- 左侧菜单：仪表盘 / 笔记 / 资源 / 构建 / 设置
- 主区：列表 + 编辑/预览双栏

建议前端策略：

- CSS 以自定义设计系统或轻量 Tailwind 风格为准
- JS 只处理上传、预览、局部交互，不做重型前端框架依赖

## 11. 安全与可靠性补充

首版必须加上：

1. 管理后台登录保护
2. 密码哈希（Argon2）
3. Session 安全配置
4. 上传文件大小限制
5. 上传类型白名单
6. Markdown 原始 HTML 安全策略
7. 基础审计日志
8. 备份策略（至少支持内容目录 + SQLite 定期备份）

### Markdown 安全建议

如果内容完全由受信任管理员维护，可允许有限 HTML；
若未来支持更多角色，建议默认做 HTML 清洗，避免 XSS。

## 12. 可观测性与运维

建议加入：

- `tracing` 日志
- 构建耗时记录
- 最近错误展示
- 健康检查接口
- 配置项：
  - 内容目录
  - 站点标题
  - 基础 URL
  - 上传限制
  - 监听开关
  - 全量重建开关

## 13. 验收标准（可测试）

1. 管理员可登录后台，并成功上传 Markdown 文件。
2. 上传后 5 秒内，站点可访问对应页面。
3. 外部修改 `content/notes/*.md` 后，系统自动增量更新网页。
4. Markdown 中的标准链接与 Wiki Link 都能正确跳转到目标页面。
5. 图片、PDF 等附件在网页中可正常显示或下载。
6. 笔记被删除或改名后，后台可报告断链或通过 alias 保持旧链接可访问。
7. 网站在桌面端与移动端均保持清晰、简洁、现代的展示效果。
8. 站点至少包含：首页、笔记列表页、单篇笔记页、标签页。
9. 构建失败时，后台能显示错误原因，不会静默失败。
10. 全量重建与增量构建都可执行，并生成一致的页面结果。

## 14. 分阶段实施计划

## Phase 0：项目初始化

目标：搭建可运行骨架。

涉及文件（计划）：

- `Cargo.toml`
- `src/main.rs`
- `src/app.rs`
- `src/config.rs`
- `src/error.rs`

输出：

- Rust Web 服务启动
- 配置加载
- 健康检查路由
- 基础模板渲染

## Phase 1：公共站点最小闭环

目标：Markdown -> 页面渲染跑通。

涉及文件（计划）：

- `src/content/front_matter.rs`
- `src/content/markdown.rs`
- `src/web/public.rs`
- `templates/base.html`
- `templates/note.html`
- `templates/home.html`

输出：

- 读取 `content/notes`
- 渲染单篇页面
- 首页 / 列表页可访问
- 基础样式完成

## Phase 2：链接图与资源系统

目标：解决笔记互链与附件嵌入。

涉及文件（计划）：

- `src/content/links.rs`
- `src/content/assets.rs`
- `src/content/graph.rs`
- `generated/site/assets/`

输出：

- 标准链接与 Wiki Link 工作
- 反向链接生成
- 图片/附件拷贝与 URL 重写

## Phase 3：后台管理

目标：实现上传、编辑、预览、发布。

涉及文件（计划）：

- `src/web/admin.rs`
- `src/web/auth.rs`
- `src/store/sqlite.rs`
- `templates/admin/login.html`
- `templates/admin/dashboard.html`
- `templates/admin/note_edit.html`

输出：

- 后台登录
- 上传 / 修改 / 删除
- 构建日志可见
- 草稿 / 发布状态

## Phase 4：自动构建与增量更新

目标：实现监听目录变化并自动重建。

涉及文件（计划）：

- `src/build/pipeline.rs`
- `src/build/watcher.rs`
- `src/build/cache.rs`

输出：

- watcher
- debounce
- build queue
- 增量构建
- 全量重建

## Phase 5：体验完善与上线准备

目标：做成可长期使用的第一版。

涉及文件（计划）：

- `src/search/index.rs`
- `templates/tag.html`
- `static/css/app.css`
- `tests/integration/*`

输出：

- 搜索
- 标签页
- sitemap / RSS
- 移动端适配
- 上线配置

## 15. 主要风险与缓解

### 风险 1：文件监听不稳定

缓解：

- debounce
- 队列串行化
- 构建任务加版本号
- 出错时允许后台手动全量重建

### 风险 2：链接规则复杂，兼容性差

缓解：

- 首版明确支持规则
- 引入 alias
- 后台显示断链报告

### 风险 3：资源路径混乱

缓解：

- 统一资源根目录
- 构建时重写 URL
- 对缺失资源生成错误报告

### 风险 4：后台改内容与本地编辑器改内容冲突

缓解：

- 文件系统为真源
- 所有后台保存都落盘
- 使用文件 hash / updated_at 做冲突提示

### 风险 5：首版范围过大

缓解：

- 严格按 Phase 0~5 递进
- 先做闭环，再做增强
- 搜索 / RSS / 主题等作为后置优化项

## 16. 验证方案

1. 单元测试：Markdown 解析、Front Matter、slug、链接重写、资源映射。
2. 集成测试：上传笔记 -> 触发构建 -> 页面可访问。
3. watcher 测试：修改文件 -> 自动增量更新。
4. 页面验证：导航、目录、附件、断链、404。
5. 手动验收：桌面端 / 移动端浏览体验。

## 17. 建议的首版范围（最推荐）

如果希望更快落地，建议 **MVP 只包含**：

1. 公共站点 SSR
2. Markdown 渲染
3. 标准链接 + Wiki Link
4. 资源复制与嵌入
5. 管理后台上传/编辑
6. 文件监听自动增量构建
7. 基础搜索

先不要把时间投入在：

- 多用户协作
- 富文本编辑器
- 复杂主题系统
- Mermaid / 数学公式等高阶扩展

---

如果进入实施阶段，推荐下一步先产出：

1. `test-spec-markdown2web-2026-04-08.md`
2. 初始目录结构与 `Cargo.toml`
3. Phase 0 + Phase 1 的最小实现
