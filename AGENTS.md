# AGENTS.md

## 项目说明

本项目 ebook-studio 是一个为电子书管理提供便利的跨平台桌面应用，基于 Tauri 2 + SvelteKit 5 + TypeScript + Rust 构建。已集成系统托盘、全局快捷键、开机自启、单实例、自动更新、对话框、文件系统、系统信息、剪贴板、多语言、日志、窗口状态记忆等桌面应用常见能力，当前保留首页、项目、设置、关于四个核心页面（项目含列表与新建/编辑子页面，列表项含作者与标签 Badge、右侧文件夹打开，详情底部为编辑+删除）及工作空间（`workspace/[id]` 独立于 `(main)`，标题栏复用 `WindowControl` + `Breadcrumb` 5 阶段：检查>拆分>清理>构建>打包，`inspect/split/clean/build/package` 已落地含自定义正则校核、一键重整、拆分编辑、OR 筛查、未压缩 EPUB 构建与 EPUB3.3 打包校验，`package` 输出 `outputs/${书名}.{epub/txt/封面}`），演示模块已移除，核心业务 `projects/inspect/split/clean/build/package` 已落地。

- 项目名：ebook-studio
- 简介：A cross platform desktop application designed to provide convenience for e-book manage
- 仓库：https://github.com/AlfredClark/ebook-studio
- 窗口标题：Ebook Studio（配置项 `tauri.conf.json` 的 `windows[0].title`），其余标识统一为 `ebook-studio`（`productName`/`identifier`/`package.json name`）
- 版本：0.1.0（`package.json` / `src-tauri/Cargo.toml` / `src-tauri/tauri.conf.json` 三处同步，`scripts/bump-version.mjs` 统一提升）

## 技术栈

- **前端**：SvelteKit 5 / Svelte 5 / TypeScript / Vite / Tailwind CSS v4 / shadcn-svelte，包管理器 bun；业务依赖 paneforge（Resizable）、`@internationalized/date`（Calendar）
- **后端**：Tauri 2 / Rust（edition 2024），Cargo workspace（成员为 `src-tauri`）；业务依赖 `chrono`（RFC3339 `Secs` 无毫秒，`dcterms:modified CCYY-MM-DDThh:mm:ssZ`）、`uuid`（`urn:uuid`）、`base64`（封面 data URL）、`regex`（卷章正则）、`encoding_rs`（content GBK 容错）、`chinese-number`（0.8.2 中文数字↔阿拉伯互转，`ChineseVariant::Simple` + `ChineseCountMethod::TenThousand`）、`zip`（EPUB 压缩，`mimetype` Stored 首条）、`epubveri`（0.9 EPUB3.3 校验，纯 Rust，AGPL-3.0）、`quick-xml`（0.41 2 空格 XML 美化）、`styloria`（0.11 CSS 美化，EPUB `base.css`，Tokenizer span 保留式：空格/注释不删）
- **国际化**：前端 Paraglide（inlang），后端 rust-i18n
- **集成能力**：系统托盘、全局快捷键、开机自启、单实例、自动更新、对话框、文件系统、系统信息、剪贴板、文件拖放、应用菜单（macOS）、通知、日志、窗口状态记忆、窗口置顶（环境能力探测）、内容检查与重整（inspect）、拆分（split→`split.json`）、清理筛查（clean OR 过滤）、构建（build→未压缩 EPUB 目录 + `format.json` 持久化 + 2 空格格式化 `xhtml/opf/css`）、打包（package→`outputs` EPUB 压缩 + txt/封面 + epubveri 校验 + 异步校验/打包 `spawn_blocking`）、文件打开/揭示（opener）、SQLite 本地数据库（tauri-plugin-sql，能力保留，业务表待扩展；projects 真相源为文件系统，见后）
- **质量工具**：ESLint / Stylelint / Prettier / Clippy / rustfmt / Vitest 单测 / Husky + lint-staged
- **授权**：GPL-3.0-only

## 项目结构

```
├── .github/                        # CI / Release 工作流（ci.yml / release.yml）
├── AGENTS.md                       # 开发规范（本文件）
├── docs/                           # 文档（README_zh-CN.md 中文完整说明 / images/ 截图）
├── src/                            # 前端（SvelteKit + TypeScript）
│   ├── components/                 # 业务组件（按性质分层：layouts/pages/ui/widgets）
│   │   ├── layouts/                # 布局系统（LayoutContainer 容器 + 布局注册表 + parts/ 布局部件）
│   │   │   └── parts/              # 布局骨架部件（TabsNavBar 导航条 + WindowControl 窗口控制 + AppSidebar 侧边栏 + SidebarTrigger + nav-items 导航数据含 getActiveNavItem）
│   │   ├── pages/                  # 页面级组件（与 routes 对应）
│   │   │   ├── about/              # 关于页组件（AppAbout / SystemAbout）
│   │   │   ├── projects/           # 项目页组件（ProjectPage 列表+详情双栏含作者/Badge/文件夹打开 + ProjectForm 复用创建/编辑 + CreateProjectPage/EditProjectPage 薄壳）
│   │   │   ├── workspace/          # 工作空间组件（stages.ts 阶段常量含 getActiveWorkspaceStage）
│   │   │   └── settings/           # 设置页组件（Appearance / SystemSettings）
│   │   ├── ui/                     # shadcn-svelte 生成组件（仅经 CLI 添加，components.json 管理，当前 26 个：alert-dialog/badge/breadcrumb/button/calendar/card/checkbox/command/dialog/input/input-group/label/popover/resizable/scroll-area/select/separator/sheet/sidebar/skeleton/slider/sonner/switch/tabs/textarea/tooltip，其中 calendar 依赖 @internationalized/date，resizable 依赖 paneforge，breadcrumb 为 workspace 标题栏导航）
│   │   └── widgets/                # 通用小组件（自包含、可插拔，按功能分子目录）
│   │       ├── icons/              # 品牌/自定义图标（GithubIcon）
│   │       └── overlay/            # 浮层组件（ConfirmDialog 确认对话框 + TooltipButton 提示按钮，复合组件式）
│   ├── features/                   # 业务功能模块（与后端 src-tauri/src/features 命名镜像——前端每功能一个目录，后端为扁平 .rs 模块，结构不镜像，当前 projects/{core,types,mock,index}.ts + inspect/{core,types,index}.ts + split/{core,types,index}.ts + clean/{core,types,index}.ts + build/{core,types,index}.ts + package/{core,types,index}.ts）
│   ├── libs/                       # 前端模块库
│   │   ├── drag-drop/              # 文件拖放（封装核心 API onDragDropEvent，窗口级监听，免权限）
│   │   ├── errors/                 # 错误处理
│   │   ├── events/                 # 事件契约（事件名常量 + payload 类型 + listenEvent 封装）
│   │   ├── hooks/                  # shadcn-svelte 生成区（如 is-mobile.svelte.ts，components.json hooks 别名；同 components/ui 豁免规则）
│   │   ├── i18n/                   # 国际化（Paraglide 编译产物、消息文件与 inlang 项目配置，消息源 src/libs/i18n/messages/{locale}.json）
│   │   ├── ipc/                    # Tauri 命令调用封装（invokeCommand + 类型定义）
│   │   ├── logger/                 # 日志（对接 tauri-plugin-log）
│   │   ├── notifications/          # 通知（对接 tauri-plugin-notification）
│   │   ├── overlay/                # 浮层（toast 统一出口）
│   │   ├── process/                # 进程能力（重导出 tauri-plugin-process 的 exit/relaunch）
│   │   ├── sql/                    # SQL 数据层（封装 tauri-plugin-sql：initSql 建表 + 类型化读写，业务表在此扩展，当前为通用空壳，projects 未使用 sql）
│   │   ├── stores/                 # 全局状态（settings 偏好 + store 工厂）
│   │   ├── system/                 # 系统配置（config 快照缓存 + toggle 业务 + 共享响应式状态）
│   │   ├── updater/                # 自动更新（check/install + 模块级状态 state.svelte.ts）
│   │   └── utils/                  # 可复用散装工具（跨模块通用，如 system-fonts 系统字体列表）
│   ├── routes/                     # 页面与布局（(main) 分组为主窗口：/(home)、/projects、/projects/create、/projects/edit/[id]、/settings、/about；workspace 分组为独立工作空间：/workspace/[id]/inspect|split|clean|build|package，+layout 为标题栏[data-tauri-drag-region + 返回 + Breadcrumb(5阶段全量可点) + WindowControl]，inspect/split/clean/build/package 已落地见工作空间节）
│   ├── styles/                     # 样式（app.css 为唯一入口；themes/ 存放主题文件）
│   │   └── themes/                 # 主题（index.css 聚合 import；index.ts 导出 themeNames/ThemeName）
│   ├── app.html                    # 应用 HTML 模板（首帧 lang 硬编码）
│   └── hooks.client.ts             # 客户端钩子
├── src-tauri/                      # 后端（Rust，Cargo workspace 成员）
│   ├── capabilities/               # Tauri 权限配置（default / plugins，含 opener:default 供 workspace 打开文件夹/重整）
│   ├── locales/                    # rust-i18n 语言文件（en.yml/zh-CN.yml，含 menu.projects）
│   ├── src/
│   │   ├── commands/               # Tauri 命令（config / env / projects / inspect / split / clean / build / package）
│   │   ├── cores/                  # 核心模块
│   │   │   ├── autostart.rs        # 开机自启
│   │   │   ├── config.rs           # 配置管理
│   │   │   ├── env.rs              # 环境信息
│   │   │   ├── events.rs           # 事件契约层（事件名常量 + payload 类型 + 类型化 emit）
│   │   │   ├── instance.rs         # 单实例
│   │   │   ├── locale.rs             # 系统语言
│   │   │   ├── logger.rs             # 日志
│   │   │   ├── menu.rs               # 应用菜单（macOS 专属，#[cfg(target_os = "macos")]）
│   │   │   ├── panic.rs              # panic 处理
│   │   │   ├── response.rs         # 统一响应
│   │   │   ├── shortcut.rs         # 全局快捷键
│   │   │   ├── tray.rs             # 系统托盘
│   │   │   └── window_state.rs     # 窗口状态记忆
│   │   ├── features/               # 业务功能模块（projects.rs：Dublin Core 文件系统；inspect.rs：content.txt 结构识别+编号校核+中文数字(委托 chinese-number)+卷简介判定+重置/连续启发式+空卷/无效检测 + 自定义正则 + reorder_chapters 重整保留标题与风格；split.rs：content.txt→split.json 结构化拆分；clean.rs：split.json 行级 OR 筛查；build.rs：split.json+metadata.json→build/<书名>/ 未压缩 EPUB + 模板渲染；package.rs：build+split+metadata→outputs/${书名}.{epub/txt/封面} EPUB3.3 压缩+epubveri 校验）
│   │   ├── lib.rs                  # 应用初始化
│   │   └── main.rs                 # 程序入口
│   ├── templates/EPUB33-NOVEL/       # EPUB 3.3 模板（mimetype/META-INF/container.xml + EPUB/{content.opf,nav.xhtml,cover/titlepage,styles/base.css,text/*-template.xhtml}，经 bundle.resources 打包）
│   ├── build.rs                    # Tauri 构建脚本
│   └── tauri.conf.json             # Tauri 应用配置（decorations:false + dragDropEnabled，含 img-src asset: http://asset.localhost data: 支持封面预览 data URL/asset）
├── scripts/                        # Node 工具脚本（bump-version.mjs 版本提升）
├── static/                         # 前端静态资源
├── .editorconfig                   # 编辑器统一风格
├── .gitattributes                  # Git 属性
├── .gitignore                      # Git 忽略规则
├── .husky/pre-commit               # 提交钩子（lint-staged）
├── .prettierignore                 # Prettier 忽略规则
├── .prettierrc                     # Prettier 配置
├── .stylelintignore                # Stylelint 忽略规则
├── .stylelintrc.json               # Stylelint 配置
├── bun.lock                        # bun 依赖锁定
├── Cargo.lock                      # Rust 依赖锁定
├── Cargo.toml                      # workspace 根：成员、lints、profile
├── cliff.toml                      # git-cliff 变更日志生成配置
├── components.json                 # shadcn-svelte 组件配置
├── eslint.config.ts                # ESLint 配置
├── LICENSE                         # GPL-3.0-only
├── package.json                    # 前端依赖与脚本（bun）、lint-staged
├── README.md                       # 项目说明（完整：特性/技术栈/快速开始/结构/版本/许可，见 docs/README_zh-CN.md）
├── rust-toolchain.toml             # Rust 工具链渠道固定（stable 滚动，不做版本锁定）
├── rustfmt.toml                    # rustfmt 配置
├── svelte.config.ts                # SvelteKit 配置
├── tsconfig.json                   # TypeScript 配置
├── vite.config.ts                  # Vite 配置
└── vitest.config.ts                # Vitest 单测配置（独立于 Vite，纯 node 环境）
```

## 后端开发规范（src-tauri）

### 架构分层

- **main.rs**：仅委托调用 `lib.rs::run()`，不含业务逻辑
- **lib.rs**：薄层——仅声明模块、组装 Builder、注册命令；不写业务逻辑
- **commands/**：IPC 命令薄层——参数校验 → 调 features/cores → 转 `Response<T>`；不写业务逻辑
- **features/**：业务功能模块——**新增功能的业务逻辑一律放此处**，每功能一个模块、单一职责
- **cores/**：核心功能模块——仅保留系统级核心能力（配置、日志、托盘、快捷键、单实例、panic、环境、语言、窗口状态记忆、事件契约、统一响应），不承载业务
- **依赖方向**：单向 `lib.rs → commands → features → cores`（commands 可直接调用 cores 的系统能力）

### 命令（commands/）

- **命令签名**：所有 IPC 命令一律 `#[tauri::command]` + `pub fn` + 返回 `Response<T>`
- **业务入口**：需要前端交互的业务，commands 作为入口薄层调用 features 的业务函数
- **系统能力编排**：toggle 系列（读当前值取反 → OS 副作用 → 写回 config）等对 cores 系统能力的编排可直接写于命令层（依赖方向允许 commands 直调 cores）；纯业务逻辑编排才下沉 features
- **命令注册**：新增命令后追加到 `commands/mod.rs` 的 `invoke_handlers!` 宏（lib.rs 无需改动）
- **环境探测**：`env::is_always_on_top_supported` 查询窗口置顶能力（Linux Wayland 下 GTK keep_above 无效，前端据此隐藏置顶按钮；前端 `WindowControl`/`SystemSettings` 的置顶与关闭行为 UI 依赖该命令）
- **项目命令**：`projects` 域提供 `list_projects` / `create_project`（`payload: CreateProjectInput`）/ `update_project`（`payload: UpdateProjectInput` 含 `removeCover/removeContent`） / `get_project` / `delete_project` / `batch_delete_projects` / `resolve_project_asset` / `get_file_stats` / `read_image_as_data_url`，前端经 `invokeCommand` 调用（如 `invokeCommand("create_project", { payload: { title, language } })`）
- **检查命令**：`inspect` 域提供 `inspect_content`（`{ identifier, structure, volumeRegex, chapterRegex }` → `InspectResult`） / `reorder_chapters`（同参，强制重整卷+章保留标题与风格（阿拉伯/中文）后自动重扫） / `get_content_path`（解析 `content.txt` 绝对路径供 opener），`inspect` 的 `structure: auto|volume_chapters|pure_chapters|volume_intro`，`volumeRegex/chapterRegex` 空即回退 `DEFAULT_*_RE`（`compile_regex` 校验，非法返 `CODE_ERROR` 正则无效）
- **拆分命令**：`split` 域提供 `get_split_content`（`{ identifier }` → `SplitResult|null`，`split.json` 不存在返 null） / `split_content`（`{ identifier }` → `SplitResult`，解析 `content.txt` → 写 `split.json`） / `save_split_chapter`（`{ identifier, volumeOrder: Option<i32>, chapterOrder, contents: string[] }` → `SplitResult`，落盘 `split.json`，`volumeOrder=null` 时为纯章节），前端经 `split_content/saveSplitChapter/getSplitContent`
- **清理命令**：`clean` 域提供 `filter_clean`（`{ identifier, filters: CleanFilters, customRegex: Option<String> }` → `CleanResult`，`CleanFilters{halfPunct,specialSymbol,unclosedPairs,repeatedPunct,abnormalWhitespace,ellipsisDash}` + 自定义 `customRegex` 任一命中即行命中、任一行命中即章 `matched`，仅点击触发，`customRegex` 非法返 `CODE_ERROR`）
- **构建命令**：`build` 域提供 `get_build`（`{ identifier }` → `BuildResult|null`） / `build_epub`（`{ identifier, chapterTitleFormat?, volumeTitleFormat?, numberFormat? }` → `BuildResult`，基于 `split.json+metadata.json` 生成 `build/<书名>/` 未压缩 EPUB，`numberFormat: arabic|arabic_padded|chinese_lower|chinese_upper`，`{order}/{title}` 占位，`sanitize_title` 去非法字符，落盘 `format.json` 供打包复用，异步 `spawn_blocking`） / `remove_build`（`{ identifier }` → `bool`，异步 `spawn_blocking`） / `read_build_file`（`{ identifier, relPath }` → `string`，仅文本文件） / `write_build_file`（`{ identifier, relPath, content }` → `bool`） / `get_build_path`（`{ identifier }` → `string|null` 供 `revealItemInDir`） / `get_format`（`{ identifier }` → `FormatConfig|null`） / `format_build_all`（`{ identifier }` → `FormatBuildResult`，整目录 2 空格格式化 `xhtml/opf/css`，`quick-xml` + `styloria`（CSS 为 Tokenizer span 保留式美化，失败不落盘，异步 `spawn_blocking`）），前端经 `getBuild/buildEpub/removeBuild/readBuildFile/writeBuildFile/getBuildPath/getFormat/formatBuildAll`
- **打包命令**：`package` 域提供 `get_package`（`{ identifier }` → `PackageResult|null`，轻量不校验，`outputs` 无 epub 返 null） / `package_epub`（`{ identifier }` → `PackageResult`，压缩 `build/<书名>/` 为 `outputs/${书名}.epub`（`mimetype` Stored 首条）+ 生成 `outputs/${书名}.txt`（含 metadata 头，卷章编号按 `format.json`）+ 拷贝 `outputs/${书名}.${ext}` 封面，均以 `sanitize_title` 命名并清空 `outputs` 重建 + `epubveri::validate_path` 校验，异步 `spawn_blocking`） / `verify_package`（`{ identifier }` → `PackageResult`，按需校验，避免加载卡顿，异步） / `remove_package`（`{ identifier }` → `bool`） / `get_package_path`（`{ identifier }` → `string|null` 供 `revealItemInDir`），前端经 `getPackage/packageEpub/verifyPackage/removePackage/getPackagePath`
- **文档示例**：函数文档注明前端调用示例（如 `invokeCommand("set_locale", { locale: "zh-CN" })`）

### 统一响应协议

- **响应协议**：所有命令返回 `Response<T>`——`code=0` 成功（data 有值）、`code!=0` 失败，invoke 永不 reject
- **类型转换**：cores/features 层返回 `AppResult<T>`（`Result<T, AppError>`），命令层经 `From` 自动转为 `Response<T>`
- **错误码**：一律使用常量（`CODE_OK` / `CODE_ERROR`），不写魔法数字

### 事件契约（cores/events.rs）

- **契约集中**：事件名常量（`EVENT_*`）+ payload 类型 + 类型化 emit 函数集中于 `cores/events.rs`，与前端 `libs/events` 镜像对齐（serde 序列化 ↔ TS 类型）；静态契约模块（无 plugin()/setup()，同 response.rs）
- **命名规范**：事件名一律 `域:事件`（如 `menu:navigate`），payload 为类型化值（枚举/结构体），不 emit 裸字符串
- **发射约定**：emit 失败仅 `log::error!`（如前端尚未就绪），不阻断调用方流程
- **传输机制决策**：后端 → 前端单次通知用事件；流式/进度/批量用 Channel（有序、完成语义、背压）；前端 → 后端请求用 command，不经事件
- **退出请求**：托盘退出菜单 / Ctrl+Q 不直接 `app.exit`——经 `app:request-exit` 事件交前端按 closeBehavior 偏好编排退出流程（前端监听于 `WindowControl`，布局注册表保证单实例），尊重关闭行为偏好
- **菜单导航**：`MenuPage` 枚举含 `Home/Projects/Settings/About`（`serde lowercase` 与前端 `MenuPage` 一致），`emit_menu_navigate` 用于 macOS 菜单及全局快捷键
- **升级阈值**：事件 ≤5 个维持轻量契约层；超过后升级全量契约（payload 全 serde 结构体 + AGENTS.md 事件清单章节）；**不建自建事件总线/中间件**

### features 模块约定

- **模块结构**：每个功能一个模块，模块内函数返回 `AppResult<T>`，不直接构造 `Response`
- **能力复用**：可复用 cores 的系统能力（配置、日志、i18n），只调其公开接口，不重写
- **模块文档**：`//!` 说明职责与涉及的真相源
- **项目业务**：`features/projects.rs` 真相源为 `APPDATA/Projects/<uuid>/metadata.json + sources/{cover.<ext>,content.txt}`（`uuid` 为 `urn:uuid:<uuid>` 的后段，目录名即 uuid），Dublin Core 字段 `title/language` 必填、`creator/contributor/publisher/date/subjects(/分割)/description(换行分割→Vec<String>)/rights/source/relation/coverage` 选填、`identifier/created/modified/modifiedMs/cover/content` 系统生成；`subjects` 以 `/` 切分、`description` 以换行切分存 `Vec<String>`；封面仅 `png/jpg/jpeg/webp/gif/bmp` 且限 `10MB`（`read_image_as_data_url`），正文仅 `txt`，拷贝经 `std::fs::copy` 固定命名 `cover.<ext>/content.txt`，失败回滚整目录；列表扫描 `Projects` 目录解析 `metadata.json` 并按 `modifiedMs desc` 排序，损坏项 `log::warn` 跳过
- **检查业务**：`features/inspect.rs` 真相源为同一 `content.txt`（`APPDATA/Projects/<uuid>/sources/content.txt`，UTF-8→GBK→lossy，限 100MB）；`StructureType::{Auto,VolumeChapters,PureChapters,VolumeIntro}` + `DEFAULT_VOL_RE/CHAP_RE`（`第X卷/章` 中阿拉伯与中文数字），`compile_regex` 支持前端自定义（空回退，非法返错）；中文数字经 `chinese-number:0.8.2` `ChineseToNumber::to_number(TenThousand)`（预处理 `兩→两/〇→零`）`parse_chapter_number` 判 `>0`；`format_number` 保留原风格（全数字→阿拉伯，否则 `NumberToChinese::to_chinese(Simple,Lower,TenThousand)`）；流程：行扫描→卷章 token（`HashSet` 去重）→关联卷索引→卷简介判定（卷后至下一卷/章间非空正文）→自动识别（`intro>0 ⇒ volume_intro` / `vol>0 ⇒ volume_chapters` / else `pure`）→ `effective` → 卷序/空卷/无效数字 → 章节 `pure:全局 1..n / volume: is_reset(首章皆1) ? 按卷 1..k : 全局 1..n` 全量收集 `missing/duplicate/out_of_order/invalid_number/empty_volume/no_chapters`；`reorder_chapters` 按同 `effective/is_reset` 生成 `lineNo→newRaw`（卷 `1..n`、章 `1..k`或`1..n`）仅替换捕获组1保留标题/空白，写回 `content.txt` 后更新 `metadata.json:modified/modifiedMs` 并重调 `inspect_content` 返回新 `InspectResult`；单元测试 `parse_chapter_number` 覆盖 `一/十二/二十/二十三/一百零三/两千` 等
- **拆分业务**：`features/split.rs` 真相源为 `APPDATA/Projects/<uuid>/sources/content.txt → APPDATA/Projects/<uuid>/split.json`，复用 `inspect` 的 `DEFAULT_VOL_RE/DEFAULT_CHAP_RE + parse_chapter_number`（含 `兩→两/〇→零` 与 `TenThousand`）与结构自动识别（`intro>0⇒volume_intro / vol>0⇒volume_chapters / else pure`）；`order` 为解析编号回退按出现顺序 `1..n`、`title` 去编号 `trim`、`contents` 按行 `trim` 去空存 `Vec<String>`、`intro` 同理 `Option<Vec<String>>`、`SplitResult{type_, volumes?, chapters?}` 落盘 `pretty JSON`；`split_content` 全量重算并写盘、`get_split_content` 不存在返 `null` 的 `Option`、`save_split_chapter(volumeOrder/chapterOrder/contents)` 原位替换章节并落盘；单元测试 `test_cn_number/split_result_serde/pure_serde` 覆盖中文数字与序列化
- **清理业务**：`features/clean.rs` 真相源为 `APPDATA/Projects/<uuid>/split.json`（`split` 阶段产物）；`CleanFilters{halfPunct,specialSymbol,unclosedPairs,repeatedPunct,abnormalWhitespace,ellipsisDash}+customRegex(Option<String>)` OR 语义——任一勾选命中或 `customRegex` 正则命中即行命中，任一行命中即章 `matched`（`CleanMatched{volumeOrder,chapterOrder}`），仅 `filter_clean` 点击触发才筛查；`CleanIssue{volumeOrder,chapterOrder,lineIndex(0基),kind,message,context,matched?}` 按 6 类（`half_punct/special_symbol/unclosed_pairs/repeated_punct/abnormal_whitespace/ellipsis_dash/custom_regex`）逐行产出；`is_unclosed` 检 7 对成对标点 distinct open/close；`customRegex` 经 `Regex::new` 校验非法返 `CODE_ERROR`；单元测试 `test_unclosed/test_filters_serde`
- **构建业务**：`features/build.rs` 真相源为 `APPDATA/Projects/<uuid>/split.json + metadata.json + sources/cover.*`，产物 `APPDATA/Projects/<uuid>/build/<sanitize_title(bookTitle)>/` 未压缩 EPUB（`mimetype` 无换行 + `META-INF/container.xml + EPUB/{content.opf,nav.xhtml,cover/titlepage.xhtml,styles/base.css,text/*.xhtml}` 模板渲染，模板源 `src-tauri/templates/EPUB33-NOVEL/` 经 `resourceDir`/`resolve` 兜底拷贝，`bundle.resources: ["templates"]`）；`sanitize_title` 非法字符 `/\:*?"<>|`→`_`、去尾点、100 字符截断、空回退 `untitled`；`format_number_display(order,numberFormat,padWidth)` 支持 `arabic|arabic_padded(按 count 补零)|chinese_lower(Simple Lower)|chinese_upper(Simple Upper)`（`NumberToChinese`），`apply_title_format("{order}/{title}"占位)` 与 `compute_pad_width`；`build_epub(chapterTitleFormat?,volumeTitleFormat?,numberFormat?)` 清历史 `build/` 后生成并 `build_file_tree+collect` 产 `BuildResult{epubPath(绝对), files: Vec<BuildFile{path,name,isDir,children?}>, bookTitle}`，落盘 `format.json` 供打包复用，`dcterms:modified` 统一 `Secs` 无毫秒（`chrono::SecondsFormat::Secs`，旧值归一化），`manifest/spine` 按阅读序交错（`vol→其章` 循环，`opus` 非 `[all vols][all chaps]`），`landmarks` 仅 3 项 `cover/titlepage/bodymatter(首章)`；`get_build` 读取同结构、`remove_build` 删目录（均 `spawn_blocking` 异步）、`read/write_build_file` 仅文本文件（`is_text_file: xhtml/html/xml/opf/css/txt/json`，`relPath` 含 `..` 拒）与转义 `escape_xml/media_type_for_ext`；`get_format` 读 `format.json`，`format_build_all` 整目录 2 空格格式化 `xhtml/opf/css`（`quick-xml` XML 2 空格；CSS 经 `styloria` Tokenizer span 保留式美化——token 从源文本原样切片、空白只归一化不删除（保 `0 auto`/后代选择器/`calc(100% - 20px)` 语义空格）、注释从 token 间隙提取原样保留、声明名值冒号后补空格而伪类冒号（`a:hover`）原样、`url(data:...)`/字符串/`@media` 天然无损、幂等，失败不落盘，异步 `spawn_blocking`）；单元测试 `test_escape/sanitize/pad_width/format_number/apply_format/format_xml/format_css`
- **打包业务**：`features/package.rs` 真相源为 `APPDATA/Projects/<uuid>/build/<书名>/` + `split.json` + `metadata.json` + `format.json`，产物 `APPDATA/Projects/<uuid>/outputs/${sanitize_title}.{epub/txt/封面}`（`mimetype` 首条 Stored + 其余 Deflated，经 `zip::ZipWriter`，清空 `outputs` 重建，三文件均以书名命名）；`read_format_config` 取 `format.json`（缺失回退默认 `第{order}章/卷 + arabic`）供 `build_txt_content`，`build_txt_content(fmt)` 拼 `metadata{title/creator/language/contributor/publisher/date/rights/subjects/description}` 头 + `split` 卷章编号格式化（`format_number_display`/`apply_title_format` 按 `fmt`）的 `title+contents/intro`；`package_epub` 校验 `build/split` 存在→`read_format_config`→生成 `txt`（带编号）→拷贝封面 `${sanitize_title}.${ext}`→压缩 `mimetype Stored`→走查 `epubveri::validate_path` 产 `PackageResult{epubPath,txtPath,coverPath,files,bookTitle,verified,issues,epubVersion}`；`get_package` 轻量不校验（避免加载卡顿，`verified=false` 占位），`verify_package` 按需校验，`remove_package/get_package_path` 为 `outputs` 目录操作；`package_epub/verify_package` 均 `spawn_blocking` 异步；单元测试 `test_sanitize/txt_pure/txt_volume`（编号 `第1章 风起`）

### cores 模块约定

- **模块三要素**：`plugin()` 插件装配、`setup()` 初始化、业务函数；`setup` 统一注册进 `cores/mod.rs::setup_cores`
- **启动前置例外**：`env::init_env()`（Linux Wayland DMABUF + AppImage EGL 渲染 workaround，勿删）与 `panic::init_hook()` 无 plugin()/setup()，须在 Builder 创建前于 lib.rs 显式调用，不纳入 setup_cores
- **错误分级**：可恢复错误不阻断启动（`log::warn!` 后继续，如自启/快捷键同步失败）；关键错误返回 `Err` 阻断
- **损坏恢复**：配置损坏备份为 `*.corrupt` 后重建，不阻断启动
- **插件装配**：需业务配置/事件的插件经 cores 的 `plugin()` 统一封装（如 `config::plugin()` 装配 store、`logger::plugin()` 配置日志目标、`shortcut::plugin()` 注册快捷键 handler），lib.rs 仅链式调用，不写插件细节
- **官方插件**：无需定制的插件（opener/clipboard-manager/process/notification/system-fonts/dialog/fs/os/updater/sql）直接在 lib.rs 以 `tauri_plugin_xxx::init()` 注册；`sql` 为通用能力保留，业务表由前端 `libs/sql` 幂等建表（`projects` 未使用 sql，选用文件系统）；`opener:default` 已用于 workspace `revealItemInDir/openPath`（重整后打开文件夹）
- **注册顺序**：单实例插件置于链首——尽早注册单例锁，避免窗口建好后回调竞态
- **职责分离**：事件/回调逻辑放 plugin()（如快捷键 handler、单实例聚焦回调），setup() 只做初始化与状态同步，不混写
- **权限同步**：新增插件且前端需调用其 API 时，同步在 `capabilities/plugins.json` 追加权限（如 `global-shortcut:default`，`opener:default` 已覆盖文件打开）
- **窗口状态记忆**：经 tauri-plugin-window-state 记录/恢复尺寸、位置与最大化状态（`.window-state.json` 于应用配置目录）；开关为 config.json 的 `window_state` key（默认关）；`cores/window_state.rs` 以 `skip_initial_state("main")` 关闭插件自动恢复，恢复改由 setup 按配置门控——跟踪与退出保存（RunEvent::Exit）为插件内置行为不受开关影响（关闭期间仍记录，重开恢复最近一次，即"暂停记忆"语义）；已知边界：Wayland 下位置恢复无效（合成器决定摆放），尺寸/最大化正常；强杀进程（无 Exit 事件）不落盘

### 系统级配置持久化

- **真相源**：`config.json`（应用数据目录）为系统级配置唯一真相源；前端 UI 偏好归前端 stores（localStorage）模块，两类配置不混用
- **持久化机制**：经 tauri-plugin-store 读写（`app.store()`），`ConfigState` 缓存于 Tauri State 注入，避免重复读文件
- **key 定义**：一律经类型化枚举定义（如 `ConfigKey`，变体与配置快照字段一一对应，编译期防拼写错），跨层使用 `pub(crate)`，不写字符串字面量
- **读取约定**：必须带默认回退（缺失/非法值返回默认值），如 `read_bool` / `read_locale`
- **写入约定**：写后立即落盘；落盘失败回滚内存缓存，保证内存态与持久化一致；多 key 更新（如 reset_config 全量重置）经 `set_many` 单次落盘 + 整体回滚，避免逐 key 落盘的中途失败部分持久化
- **副作用顺序**：需同步 OS 的配置（自启/托盘）先 OS 生效再写回 config，失败不落盘，避免两侧不一致
- **损坏恢复**：损坏不阻断启动——备份为 `*.corrupt` 后重建默认配置

### 日志约定

- **日志库**：使用 `log` crate（前端经 tauri-plugin-log 共用同一链路）
- **消息前缀**：日志消息带 `[模块名]` 前缀（如 `[config]`、`[tray]`、`[panic]`、`[projects]`、`[inspect]`）
- **级别**：`info` 正常事件 / `warn` 可恢复失败 / `error` 出错

### 文档注释

- **模块注释**：`//!` 职责 + 真相源约定 + 已知边界
- **函数注释**：功能描述 + `@param` + `@returns`
- **决策注释**：关键决策写"为什么"注释（如"先 OS 生效，失败直接返回，不写回 config"）

### 国际化

- **注册**：`rust_i18n::i18n!("locales", fallback = "en")` 宏在 lib.rs 顶部注册，新增语言只需新增 `locales/{lang}.yml`
- **文案**：一律经 `t!("key")` 取，不硬编码中英文；消息源加在 `locales/*.yml`（缺失回退 `en`）。例外：tauri 预设项/系统固定文案保持原文（预设文案固定不可本地化，如 macOS 菜单 Edit 组，见 cores/menu.rs 已知边界）；`menu.projects` 已在 `locales/en.yml`/`zh-CN.yml` 同步
- **语言校验**：语言标签经 `Locale` 新类型校验，非法值拒绝写入；`system` 为「跟随系统」模式哨兵值（cores/locale.rs 的 `SYSTEM_LOCALE`），config 存此值时运行期经 `Locale::from_system` 解析（tauri-plugin-os 取系统标签，完整标签/主语言子标签精确匹配，不匹配回退默认）
- **跟随系统模式**：locale 为 `system`（首启/重置默认）时语言跟随系统，解析留待运行期；固定标签则按存储值；已知边界——运行期系统语言变更不自动跟随（无 OS 监听），重启应用后生效

### 质量门槛

- **全局校验**：修改代码后运行 `bun run validate`（见「校验约定」）
- **格式化与检查**：提交前通过 `cargo fmt` 与 `cargo clippy -- -D warnings`
- **单元测试**：涉及状态/副作用的逻辑可加 `#[cfg(test)]` 单元测试（参考 `cores/panic.rs` / `features/inspect.rs::parse_chapter_number`）

### 注意事项

- **依赖添加**：新增 Rust 依赖统一加到 `src-tauri/Cargo.toml`；与前端成对的 Tauri 能力需同步 npm 包与 capabilities 权限（见前端注意事项）；`projects` 新增 `chrono/uuid/base64`，`inspect` 新增 `regex/encoding_rs/chinese-number@0.8.2`（含 `chinese-to-number` 与 `number-to-chinese` 特性）均为纯 Rust 逻辑，`opener:default` 已覆盖文件打开

## 前端开发规范（src）

### 架构与模块

- **SPA 模式**：`+layout.ts` 关闭 SSR（`ssr = false`）；adapter-static + fallback 单页渲染，适配 Tauri 本地文件加载
- **routes/**：分组路由——`(main)` 组存放主窗口页面（首页 `/`、项目 `/projects`、新建 `/projects/create`、编辑 `/projects/edit/[id]`、设置 `/settings`、关于 `/about`）；`workspace` 组为独立工作空间（`[id]/+layout.svelte` 标题栏 `data-tauri-drag-region` + 返回 + `Breadcrumb`(5阶段全量可点) + `WindowControl` 复用，`[id]/+page.svelte` 重定向至 `inspect`，`[id]/{inspect,split,clean,build,package}/+page.svelte` 各阶段占位，当前 `inspect` 已实现封面+信息+结构选择+自定义正则+左右分栏结果）；页面内容经 `(main)/+layout.svelte` 监听 `menu:navigate` 统一 `goto(resolve(...))`，workspace 脱离 `LayoutContainer`
- **components/**：业务组件目录——按性质分层：`pages/` 页面级组件（仅被对应路由消费，与页面一一对应，当前含 `projects/ProjectPage + ProjectForm + CreateProjectPage/EditProjectPage` 与 `workspace/stages.ts + Inspect` 逻辑）、`widgets/` 自包含可插拔小组件、`layouts/` 布局系统、`ui/` shadcn 生成组件（svelte.config.ts 已预留 `$components` 别名）
- **components/ui/**：shadcn-svelte 生成组件（`$components/ui` 别名）——经 `bunx shadcn-svelte add <name>` 拉取，源码即项目代码，允许按需修改（尽可能不修改，本地修改后升级组件时须注意差异）；**生成区禁手动添加组件**，需定制的基础组件放 components 对应功能分类；别名配置见 components.json（ui=$components/ui、utils=$libs/utils/shadcn），当前 26 个（`alert-dialog/badge/breadcrumb/button/calendar/card/checkbox/command/dialog/input/input-group/label/popover/resizable/scroll-area/select/separator/sheet/sidebar/skeleton/slider/sonner/switch/tabs/textarea/tooltip`，其中 `calendar` 依赖 `@internationalized/date`，`resizable` 依赖 `paneforge`，`breadcrumb` 供 workspace）
- **libs/**：前端模块库，每模块的文件约定——`index.ts` 统一出口、`core.ts` 实现、`types.ts` 类型契约；跨组件共享的 runes 模块级状态放 `state.svelte.ts`（如 updater 的 `update` 状态，ESM 仅加载一次）
- **模块出口**：`index.ts` 为统一出口 + 组装点——重导出各文件（`export { x } from "./core"` + `export * from "./types"`），并组装跨文件的实例（如 stores 的 `settings`），具体实现仍留在各功能文件；无自有类型契约可省略 `types.ts`（如 logger/updater 复用 npm 包类型）
- **类型归属**：`types.ts` 仅存放模块通用类型（跨文件/跨模块复用，如 stores 的 `Store` / `StoreDefinition` / `ColorScheme` / `LayoutName`）；少数文件内部使用的类型直接在文件内定义，不写入 types.ts
- **散装工具**：跨模块通用、无业务归属的小函数放 `$libs/utils`（复用性强的独立函数，不绑定具体业务模块）
- **目录命名判据**：内容是"多个同类成员的集合"→ 复数（`layouts/` `pages/` `widgets/` `commands/` `cores/` `features/` `stores/` `utils/` `events/` `errors/` `parts/` `themes/` `icons/`）；是"抽象域/单一能力"→ 单数（`system/` `updater/` `logger/` `ipc/` `overlay/` 及后端全部 `.rs` 模块）；边界模糊时按"集合"处理（目录容纳多成员即复数）；新增模块沿用此判据，不随个人习惯摇摆
- **文件命名**：TS/JS 源文件一律 kebab-case（如 `package-info.ts` / `system-fonts.ts` / `state.svelte.ts`），不混用 camelCase；框架/工具约定的保留命名不受约束——SvelteKit 保留文件（`+layout.ts`、`hooks.client.ts`）、vite 约定（`vite-env.d.ts`）、根配置文件（`eslint.config.ts` / `svelte.config.ts` / `vite.config.ts`，单词名无连字符）、shadcn 生成区（`components/ui/**`）与 paraglide 编译产物（不入库）
- **注释风格**：TS/JS 文件注释一律 JSDoc（`/** */` / `//`），不沿用 Rust 的 `//!` 模块文档风格（`//!` 在 TS 中无特殊语义，混用破坏一致性）
- **别名**：`$libs` → `src/libs`、`$components` → `src/components`、`$features` → `src/features`、`$styles` → `src/styles`（svelte.config.ts）

### 业务功能（features）

- **归属判据**：通用可复用 → `libs/`；与具体业务绑定、不通用 → `features/`（与后端 `src-tauri/src/features` 命名镜像、结构不镜像——前端每功能一个目录，后端为扁平 `.rs` 模块，当前 `projects` 已落地 `{core,types,mock,index}.ts` + `inspect` 已落地 `{core,types,index}.ts`（`InspectResult{detected,requested,volumes,chapters,issues,stats,absPath}`，`InspectStructure`，自定义正则直通）+ `split` 已落地 `{core,types,index}.ts`（`SplitResult{type, volumes?:SplitVolume{order,title,intro?,chapters:SplitChapter{order,title,contents[]}}, chapters?:SplitChapter[]}`，`SplitType`，`getSplitContent/splitContent/saveSplitChapter`） + `clean` 已落地 `{core,types,index}.ts`（`CleanResult{matched:CleanMatched[],issues:CleanIssue{volumeOrder,chapterOrder,lineIndex,kind,message,context}}`，`CleanFilters+customRegex`，`filterClean` OR） + `build` 已落地 `{core,types,index}.ts`（`BuildResult{epubPath,files:BuildFile[],bookTitle}`，`BuildFile{path,name,isDir,children?}`，`NumberFormat+BuildOptions`，`FormatConfig+FormatBuildResult`，`getBuild/buildEpub/removeBuild/readBuildFile/writeBuildFile/getBuildPath/getFormat/formatBuildAll`） + `package` 已落地 `{core,types,index}.ts`（`PackageResult{epubPath,txtPath,coverPath,files,bookTitle,verified,issues,epubVersion}`，`PackageFile+PackageIssue`，`getPackage/packageEpub/verifyPackage/removePackage/getPackagePath`））
- **IPC 直调**：features 可直接调 `invokeCommand`（等同后端 commands+features 合并层，不复刻 commands 薄层）；失败返回 null，调用方 `?? 兜底`
- **模块约定**：沿用 libs——`index.ts` 统一出口、`core.ts` 实现、`types.ts` 契约、`state.svelte.ts` runes 状态
- **消费关系**：页面级组件（components/pages/）调 features；features 可调 libs（ipc/logger/stores）与官方插件；`inspect` 的 `inspectContent/reorderChapters/getContentPath` 与 `split` 的 `getSplitContent/splitContent/saveSplitChapter`、 `clean` 的 `filterClean`、`build` 的 `getBuild/buildEpub/removeBuild/readBuildFile/writeBuildFile/getBuildPath`、`package` 的 `getPackage/packageEpub/removePackage/getPackagePath` 均经 `plugin-opener` 的 `revealItemInDir` 协同（clean/build 的保存复用 split 章节落盘，package 揭示 outputs）

### UI 组件规范（shadcn-svelte）

- **优先复用**：为保证风格统一，UI 一律尽可能使用 shadcn-svelte 已有组件（`$components/ui`）；缺失的组件经 `bunx shadcn-svelte add <name>` 添加，确需定制的基础组件才手写（放 components 对应功能分类）
- **禁止覆盖**：`add` 添加组件时不覆盖已有组件（不使用 `-o/--overwrite`，避免冲掉本地修改）；已有组件的升级经 `bunx shadcn-svelte update` 时手动核对差异，或作为新组件引入
- **样式外置**：允许修改组件源码，但尽可能不修改——样式定制优先经组件 `class` 属性（cn 合并）与外部 class 解决，仅确需改变行为/修复缺陷时才改源码

### 状态管理（stores）

- **createStore**：基础 store 工厂（Writable 兼容 + `get`/`reset` 增强，可选持久化与值变更回调）
- **组合 store**：`storeDef<T>(initial, persist?, subscribe?)` 声明子 store（携带精确类型，避免字面量变窄丢失联合类型）+ `createStoreGroup({...})` 按对象属性名映射为分组 store（如 `settings = { layout, theme, closeBehavior, checkUpdates, font, fontSize, fontWeight }`）；新增偏好在此追加
- **值校验**：非法/残留持久化值的兜底在模块级显式处理（如 settings.ts 启动时校验主题残留回退 neutral、closeBehavior 残留回退 ask，经 set 同步修正 data-theme 与持久化）
- **类型定义**：模块通用类型（`Store` / `StoreDefinition` / `PersistOptions` / `StorageType` / `StorageAdapter` / `LayoutName` / `CloseBehaviorName`）写入 `types.ts`，单文件使用的类型可以直接定义到文件中
- **文件职责**：功能文件定义实例与订阅回调（如 settings.ts 的 `settings` 经 `storeDef` 的 subscribe 注入）；`index.ts` 为纯统一出口（re-export）——方法不反向依赖 index
- **副作用订阅**：store 副作用（如主题应用）经 `storeDef` 的 `subscribe` 参数声明式注入——创建时执行一次 + 每次变更触发，无需显式 init 调用；回调无法返回 cleanup（当前用例均为无状态回调，如主题监听器，无需清理）
- **持久化**：UI 偏好经 localStorage/sessionStorage（JSON）持久化，key 由各 persist 显式指定（与属性名解耦）；写入失败静默不影响内存；**系统级配置归后端 config.json，两类配置不混用**
- **读取兜底**：消费方对非法/未知值回退默认（如 LayoutContainer `layouts[$layout] ?? layouts.default`）
- **Svelte 5**：`$` 自动订阅仅支持标识符（不支持 `$obj.store` 成员表达式）——先 `const { layout } = settings` 解构再 `$layout`；响应式用 `$state` 声明；事件绑定用 `onclick` 属性；初始化放 onMount（Tauri IPC 在 load 阶段会触发 fetch 检查误报）

### 布局系统（components/layouts）

- **注册表**：`index.ts` 导出 `layouts: Record<LayoutName, Component>`——新增布局追加组件与 `LayoutName` 变体（Record 约束编译期强制同步）；`LayoutName` 为跨模块通用类型（stores/types.ts，与 `settings.layout` 偏好值域一致）
- **容器**：`LayoutContainer` 订阅 `settings.layout` 经注册表动态渲染，非法/未知值回退 `layouts.default`；`(main)/+layout.svelte` 包裹 children 统一走容器
- **布局组件**：各布局（Default/Dashboard）仅实现基础骨架（header/nav/main/footer），children snippet 透传页面内容
- **布局部件**：骨架部件（TabsNavBar 导航条、WindowControl 窗口控制、AppSidebar 侧边栏、SidebarTrigger、nav-items 导航数据含 `getActiveNavItem`）放 `layouts/parts/`——被布局专属消费、非通用组件，与 `widgets/`（自包含可插拔）分离；各布局组件在此组合骨架；`nav-items.ts` 导出 `getActiveNavItem(pathname, items)` 按最长前缀匹配（`"/"` 精确，其余 `pathname===href || startsWith(href+"/")` 按 `href` 长度降序），使 `/projects/create` 仍高亮 `Projects`，`TabsNavBar`/`AppSidebar`/`Dashboard` 均经此判定

### 工作空间（workspace）

- **路由**：`src/routes/workspace/[id]/+layout.svelte` 为标题栏（`data-tauri-drag-region` + 返回 `/projects` + `Breadcrumb` 5 阶段全量可点 + `WindowControl` 复用），`+page.svelte` 重定向至 `inspect`，`inspect/split/clean/build/package` 各占位，`workspace` 与 `(main)` 同级脱离主布局
- **阶段常量**：`src/components/pages/workspace/stages.ts` 导出 `WorkspaceStage` 联合与 `WORKSPACE_STAGES: {value,label,href,desc}[]` 及 `getActiveWorkspaceStage(path)`（`split("/").filter(Boolean)[2]`）
- **检查页**：`src/routes/workspace/[id]/inspect/+page.svelte` 初始居中封面+信息+`Select: auto|volume_chapters|pure_chapters|volume_intro` + 自定义 `Input: volumeRegex/chapterRegex`（空回退默认 `DEFAULT_*_RE` 字面量占位）+ 检查；结果态 `Resizable.PaneGroup` 左右分栏（左信息+重扫，右 `Badge detected/requested` + 统计四宫 + `issues` 列表 + `hasReorder(missing/duplicate/out_of_order) ? ConfirmDialog(重整章节,Wrench) : 空 + 打开文件夹(revealItemInDir)`，重整 `reorder_chapters` 后自动重扫，卷同步 `1..n`、章 `pure/连续=全局1..n/重置=按卷1..k` 保留标题与阿拉伯/中文风格）
- **拆分页**：`src/routes/workspace/[id]/split/+page.svelte` `Resizable` 左右分栏——左卷章树（`SvelteSet collapsed` 折叠、搜索 `title` 过滤、纯章节/分卷两种 `filteredPureChapters/filteredVolumes`、选中 `selected {volumeOrder,chapterOrder}+draft` 脏检查）+ 右 `Textarea` 编辑（`dirty` 守卫），顶栏 `split_content`/`splitContent(pending)` + 重拆 `ConfirmDialog` + `saveSplitChapter(saving)`，空态 `no_split_hint` 引导
- **清理页**：`src/routes/workspace/[id]/clean/+page.svelte` 依赖 `split.json`（无则 `need_split` 引导 `go_split`）；顶栏 `6 Checkbox CleanFilters + Input customRegex + Filter(filterClean pending)/Reset` OR 语义（任一勾选或正则命中即行命中、任一行命中即章 `matched`，仅点击触发）；`Resizable` 左 `cleanResult.matched` 过滤卷章树（`matchedSet/CleanMatched`）、右编辑 `Textarea` + `issues` 明细（`lineIndex 0基、kind/message/context/matched`）+ `saveSplitChapter` 保存（复用 split 落盘）
- **构建页**：`src/routes/workspace/[id]/build/+page.svelte` 顶栏格式面板（`Input chapterTitleFormat/volumeTitleFormat("{order}/{title}"占位) + Select numberFormat: arabic|arabic_padded|chinese_lower|chinese_upper`）+ `build_epub(pending)`/`rebuild ConfirmDialog` + `format_build_all(Wand, 2 空格 xhtml/opf/css, 失败不落盘, 显式整目录)` + `remove_build` 重置 + `revealItemInDir(get_build_path)` 打开文件夹（`getFormat` 预加载回填）；`Resizable` 左 `BuildFile` 树（`SvelteSet collapsedDirs`、搜索过滤、纯文本 `isTextFile` 判定）、右 `Textarea` 编辑（仅文本文件可写 `read_build_file/write_build_file`，二进制 `binary_hint` 只读，`dirty` 守卫）；产物 `build/<书名>/` 未压缩 EPUB 经 `build_file_tree` 展示，空态 `no_build_hint` 引导先拆分
- **打包页**：`src/routes/workspace/[id]/package/+page.svelte` `build` 门控（无则 `need_build` 引导 `go_build`）+ 未打包态卡片（`package_action` / 三文件说明均以书名命名 + `package_no_cover_hint`）+ 已打包态卡片（`Badge verified/verify_failed` + `files: PackageFile[] + formatSize` + `hasChecked ? (verified ? no_issues : issues_title + ScrollArea issues{id/severity/location/position/text/rule}) : 校验占位(ShieldCheck虚线)` + `verify(ShieldCheck, verifyPending) + repackage ConfirmDialog(清空重建, files join) + open outputs(revealItemInDir(get_package_path))`）；操作 `packageEpub(pending, 校验后verified)` / `getPackage` 轻量预加载（不校验）+ `verifyPackage(verifyPending, 异步 spawn_blocking)` 按需校验 + 重打包确认
- **交互**：项目列表双击或文件夹图标 `handleOpen → /workspace/<uuid>`；`Inspect` 手动触发，结构与自定义正则随 `handleInspect/handleReorder` 透传；`Split/Clean/Build` 均 `onMount` 轮询 `getSplitContent/getBuild` 预加载，`Package` `onMount` 轮询 `getPackage/getBuild/getSplitContent`，`Breadcrumb` 按钮 `href={resolve(stage.href(id))}` 全量可点

### IPC 调用

- **封装**：一律经 `$libs/ipc` 的 `invokeCommand<T>(command, args?)`，不直接调 `invoke`
- **解包**：自动解包统一响应——业务失败返回 null 并写日志；调用处用 `?? 默认值` 兜底
- **参数**：args 键名与 Rust 命令参数一致（Tauri 驼峰转换）
- **类型对齐**：前端接口（`Response<T>` / `SystemConfig` / `Project` / `InspectResult` / `SplitResult` / `CleanResult` / `BuildResult` / `PackageResult`）与 Rust 侧 cores/features 一一对应，后端类型变更时同步更新 types.ts

### 系统配置（libs/system）

- **职责**：系统级配置业务层（与后端 cores/config 镜像）——`get_config` 快照缓存（in-flight 去重，并发调用只发一次 IPC）+ toggle 系列编排（toggle_* 命令成功后回填共享状态）+ 置顶能力探测 + 共享响应式状态（state.svelte.ts 单例）
- **消费方式**：系统配置的读取/切换一律经 `$libs/system`（`getSystemConfig` / `loadSystemConfig` / `toggle*` / `isAlwaysOnTopSupported`），**组件不直连 `invokeCommand`**（SystemSettings/WindowControl 已解耦）
- **响应式约定**：`$derived`/模板内经 `getSystemConfig()` 读值保持响应式跟踪；Svelte 5 限制跨模块不可重绑导入的 `$state` 变量，整体刷新经 `setSystemConfig`（属性原位赋值无此限制）
- **跨领域联动**：如托盘关闭 → closeBehavior 回退 ask 属「系统配置 × UI 偏好」协调，留在组件层（SystemSettings），不进 libs/system（不引入 stores 依赖）
- **与 stores 分界**：系统级配置（config.json 真相源）归 libs/system；UI 偏好（localStorage 真相源）归 stores/settings，两类不混用

### 事件系统（libs/events）

- **契约镜像**：事件名常量 + payload 类型集中于 `types.ts`，与后端 `cores/events.rs` 镜像对齐（TS 类型 ↔ serde 序列化）；新增事件两端同步
- **消费方式**：一律经 `$libs/events` 的 `listenEvent<T>(event, handler)` 监听（handler 收解包后 payload），不直接 import `@tauri-apps/api/event`；事件名用常量，不写字符串字面量
- **生命周期**：`listenEvent` 异步 resolve，调用方须管理 unlisten——组件销毁时调用；resolve 晚于销毁时先判定 disposed 再决定清理或留存（参考 `(main)/+layout.svelte`）
- **传输机制决策**：后端 → 前端单次通知用事件；流式/进度/批量用 Channel（有序、完成语义、背压）；前端 → 后端请求用 command，不经事件
- **升级阈值**：事件 ≤5 个维持轻量契约层；超过后升级全量契约（payload 全 serde 结构体 + AGENTS.md 事件清单章节）；**不建自建事件总线/中间件**

### 原生对话框（dialog）

- **能力来源**：原生文件选择/保存/消息/询问框经 `@tauri-apps/plugin-dialog` 提供的 `open` / `save` / `message` / `ask` / `confirm` API 调用，**不经 `invokeCommand`**——官方插件自带 IPC 封装，与 notification 同模式
- **权限**：`dialog:default`（capabilities/plugins.json）
- **返回约定**：`open`/`save` 用户取消时返回 `null`；`ask`/`confirm` 返回用户选择（boolean）；`message` 完成时 resolve
- **调用示例**：`const file = await open({ multiple: false, filters: [{ name: "文本", extensions: ["txt"] }] })`；`projects` 创建页封面/正文均经此 API（封面 `Images`、正文 `txt`），支持拖放（`libs/drag-drop` 窗口级 `onDragDropEvent`）+ 点击选择双模式

### 应用内确认对话框（ConfirmDialog）

- **组件来源**：`$components/widgets/overlay/ConfirmDialog`（复合组件式，shadcn Alert Dialog，WebView 内渲染、随主题联动）——**无全局单例**，调用方局部定义使用，经 `{#snippet trigger()}` 传入真实触发按钮（必传）
- **使用场景**：需要应用主题化/自定义排版的关键操作二次确认（如关闭窗口、删除项目、重整章节）；系统级交互仍走 `@tauri-apps/plugin-dialog`（原生 `ask`/`confirm`）或文件选择
- **props**：`trigger`（必传，接收 bits-ui 委托 props 须 `{...props}` 展开且勿覆盖 onclick）、`open`（可选 $bindable，仅需程序化控制时绑定）、`title`/`message`（调用处已 i18n）、`variant: "default" | "destructive"`（危险操作红色确认按钮）、`confirmLabel`/`cancelLabel`（默认 `m.common_confirm()` / `m.common_cancel()`）、`onConfirm`/`onCancel`
- **语义**：确认按钮 → `onConfirm`（对话框自动关闭）；取消按钮/ESC/遮罩点击 → 仅关闭并触发 `onCancel`；内部 `confirmed` 标志防止 Action 关窗误触 onCancel
- **双委托**：触发按钮同时需要其他 bits-ui 触发器（如 Tooltip）时，优先用 `TooltipButton` 的 `extraProps` 吸收外部委托 props（内部经 mergeProps 链式合并 ref/事件，勿用对象展开）；手写 `mergeProps` 仅保留给特殊场景
- **调用示例**：`<ConfirmDialog title={m.xxx()} message={m.xxx()} variant="destructive" onConfirm={() => ...}>{#snippet trigger({ props })}<Button {...props}>删除</Button>{/snippet}</ConfirmDialog>`；`inspect` 重整即 `ConfirmDialog(重整, Wrench)` 触发 `reorder_chapters`

### 文件系统（fs）

- **能力来源**：文件读写/查询经 `@tauri-apps/plugin-fs` 提供的 API 调用（如 `exists`），**不经 `invokeCommand`**——官方插件自带 IPC 封装，与 notification/dialog 同模式；`projects` 的实际持久化走后端 `std::fs`（`APPDATA/Projects/<uuid>`），前端仅经命令间接访问，不直连 `plugin-fs` 写；`workspace inspect` 的打开文件夹经 `plugin-opener: revealItemInDir`
- **权限**：`fs:default`（只读 + mkdir，无写入命令）——`read_dir`/`read_file`/`read_text_file`/`read_text_file_lines`/`read_text_file_lines_next`/`exists`/`mkdir`，scope 覆盖五个应用专属目录（$APPCONFIG/$APPDATA/$APPLOCALDATA/$APPCACHE/$APPLOG）及其递归子目录，默认拒绝 webview 数据目录（Linux $APPLOCALDATA、Windows $APPLOCALDATA/EBWebView）；**不含文件写入**，写文件（writeFile/remove/rename 等）须显式追加 `fs:allow-*` 权限；新增 fs 能力时按需扩展权限与 scope
- **路径约定**：`BaseDirectory.AppData` 展开即 `$APPDATA`（store 插件经 AppData 解析，config.json 真实落盘于此），调用路径须落在权限 scope 内，否则被拒绝；`projects` 的 `APPDATA/Projects` 由后端 `app.path().resolve("Projects", BaseDirectory::AppData)` 解析
- **调用示例**：`await exists("config.json", { baseDir: BaseDirectory.AppData })`

### 系统信息（os）

- **能力来源**：经 `@tauri-apps/plugin-os` 提供的 API 调用——`platform` / `version` / `type` / `arch` / `family` / `exeExtension` / `eol` 同步，`hostname` / `locale` 异步（Promise），**不经 `invokeCommand`**——官方插件自带封装，与 notification/dialog/fs 同模式
- **权限**：`os:default`（capabilities/plugins.json，覆盖全部系统信息命令）
- **注意事项**：`type()` 与 TS 关键字冲突，import 须重命名（`type as osType`）
- **调用示例**：`platform()` / `await hostname()`

### 剪贴板（clipboard）

- **能力来源**：经 `@tauri-apps/plugin-clipboard-manager` 提供的 API 调用——`writeText` / `readText` / `writeHtml` / `clear` / `writeImage` / `readImage`，**不经 `invokeCommand`**——官方插件自带 IPC 封装，与 notification/dialog/fs 同模式
- **权限**：`clipboard-manager:default` 为**空权限集**（官方刻意设计，读写须显式开启）；模板默认**不开启任何剪贴板权限**（能力已接线、零权限锁定，业务需要时在 `capabilities/plugins.json` 显式追加 `clipboard-manager:allow-*`：read/write-text、write-html、read/write-image、clear，按需裁剪）
- **已知边界**：Linux Wayland 下图片剪贴板能力取决于 arboard 后端支持，文本不受影响；`readImage` 返回原始 RGBA 字节（无编码格式），预览须经 canvas 转换
- **调用示例**：`await writeText("text")` / `const text = await readText()` / `await clear()`

### 错误处理

- **三层拦截**：window error（capture 阶段，含资源加载失败）+ unhandledrejection + svelte:boundary（渲染边界 + 手动重试按钮）；SvelteKit `handleError` 经 hooks.client.ts 接入
- **注册时机**：`initErrorHooks()` 在 hooks.client.ts 模块作用域调用（早于任何渲染，捕获最早异常）
- **防循环**：错误日志写入必须静默容错（`.catch(() => {})`），错误钩子带防重入守卫，避免日志失败触发 rejection 无限循环
- **边界 UI**：渲染边界回退提示文案经 `m.xxx()` 国际化

### 日志约定

- **日志库**：经 `$libs/logger`（重导出 @tauri-apps/plugin-log）写入，与后端共用同一链路（LogDir 落盘）
- **初始化**：应用启动（+layout.svelte onMount）调用 `initLogger()` 一次（attachConsole 控制台镜像）
- **消息前缀**：日志消息带 `[模块名]` 前缀，与后端风格对齐（如 `[ipc]`、`[updater]`、`[error]`、`[projects]`、`[inspect]`）
- **已知边界**：文件统计与封面预览由后端 `get_file_stats` / `read_image_as_data_url` 提供，非前端直读；`inspect` 的 `revealItemInDir` 失败仅 `toast` 不阻断

### 国际化

- **文案**：一律经 paraglide 编译产物 `m.xxx()` 取，不硬编码；动态文案用 `ParaglideMessage` 组件
- **键命名**：`<前缀>_<具体含义>`（全小写 snake_case），前缀按归属域——`nav_` 导航标签 / `window_control_` 窗口控制 / `settings_` 设置项 / `about_` 关于页 / `theme_` 主题 / `layout_` 布局 / `language_` 语言 / `footer_` 页脚 / `boundary_` 错误边界 / `common_` 通用文案（确认/取消按钮）/ `home_` 首页文案 / `sidebar_` 侧边栏 / `updater_` 更新提示 / `projects_` 项目域（含 `projects_create_*` 创建、`projects_detail_*` 详情、`projects_sort_*` 排序、`projects_delete_*` 删除、`projects_batch_*` 批量、`projects_open` 打开）/ `workspace_` 工作空间（`workspace_back` 返回，`workspace_stage_*` 5 阶段及 `_desc`，`workspace_load_failed`）/ `inspect_` 检查（`inspect_title/no_cover/no_content/structure_label/_auto/_volume_chapters/_pure_chapters/_volume_intro/volume_regex_label/chapter_regex_label/regex_invalid/check/checking/rescan/detected/requested/stats_*/no_issues/issues_title/issue_* /open_folder/reorder/reordering/reorder_confirm_* /reorder_success/failed/need_content`）/ `split_` 拆分（`split_action/splitting/resplit/_confirm_*/success/failed/need_content/no_split_hint/search/save/_save_* /no_chapters/select_hint/contents_*/stats_* /volume_/chapter_*`）/ `clean_` 清理（`clean_title/need_split/go_split/filter_*/regex_*/filtering/reset/no_filter_hint/no_matched/matched_count/no_chapters/select_hint/issues_title/issue_*/line_no/contents_count/save/_save_* /filter_failed/need_selection/edit_hint`）/ `build_` 构建（`build_need_split/go_split/action/building/rebuild/_confirm_*/success/failed/search/save/_save_* /open_folder/no_build_hint/select_hint/no_matched/readonly/edit/binary_hint/no_files/chapter/volume_title_format_*/number_format_*/remove_*/title_format_hint`）/ `package_` 打包（`package_need_build/go_build/action/packaging/repackage/_confirm_*/success/failed/open_outputs/no_package_hint/verified/verify_failed/issues_*/no_cover_hint/remove_*/no_files`）；禁止裸名词键（如 `welcome`）。已移除 `demo_` 演示前缀
- **消息源**：`src/libs/i18n/messages/{locale}.json`（`project.inlang/settings.json` 的 `pathPattern: ./messages/{locale}.json` 相对 `project.inlang` 目录）；新增语言需同步 `project.inlang/settings.json` 的 locales；改动后运行 `bun run i18n:compile`
- **locale 真相源**：config.json（后端）为准，存储模式值为 `system`（跟随系统）或具体标签；`changeLocale` 先写后端成功才切前端（双写，set_locale 返回解析后的具体标签喂 paraglide）；`initLocale` 启动时同步（system 模式经 `resolve_locale` 命令解析），失同步以 config 为准 reload 自愈
- **首帧**：app.html 硬编码 lang="en"，由 initLocale 运行期更新 `document.documentElement.lang`

### 注意事项

- **成对依赖**：前端用到的 Tauri 能力需 npm 包 + Rust 侧 tauri-plugin 依赖 + `capabilities/plugins.json` 权限三者齐备（如 notification/updater/system-fonts/clipboard-manager/sql——sql 已保留，含 `sql:default` + `sql:allow-execute`；`opener:default` 已覆盖 `revealItemInDir/openPath` 供 workspace）；`projects` 的 `dialog:default` 已用于封面/正文选择，`chrono/uuid/base64` 与 `inspect` 的 `regex/encoding_rs/chinese-number` 为纯 Rust 逻辑无需额外权限，`split/clean` 无新增 Rust 依赖（模板 `templates/EPUB33-NOVEL` 经 `bundle.resources` 打包），`build` 新增 `quick-xml 0.41 / styloria 0.11`（2 空格格式化，`styloria` 0.11 为 CSS 经 `epubveri` 引入，`quick-xml` 已在 lock 经 `plist` 引入），`package` 新增 `zip/epubveri`（`zip` 已在 lock 经 updater 引入，显式 4，`epubveri 0.9` 为 AGPL-3.0 纯 Rust，EPUB3.3 校验）
- **构建配置**：vite dev 端口固定 1420（strictPort），与 tauri.conf.json 的 devUrl/CSP 一致；watch 忽略 `src-tauri` 与根 `target`（Windows 上 watch 被 cargo 锁定的构建脚本 exe 会 EBUSY 崩溃）；改端口需同步改 tauri.conf.json
- **首帧性能**：SPA 白屏经「单入口打包」缓解——`svelte.config.ts` 配 `kit.output.bundleStrategy: "single"` 收敛 JS 单入口（消除 modulepreload/动态 import 请求链，JS 仍外链不受 CSP 约束）
- **全局常量注入**：经 vite `define` 整体注入配置对象（`__APP_TAURI_CONF__` 为整份 tauri.conf.json、`__APP_PKG__` 为整份 package.json），消费方按需取属性；类型在 `src/vite-env.d.ts` 经 `import type ... from "*.json"` 引用 JSON 字面量推导（天然同步）；新增配置须同步 eslint.config.ts 的 `viteDefineGlobals`；watch 忽略 src-tauri，改配置需重启 dev 生效
- **Tailwind v4**：经 `@tailwindcss/vite` 插件编译（vite.config.ts，无 postcss 配置）；`src/styles/app.css` 为唯一入口（`@import "tailwindcss"` + `@import "./themes/index.css"`）；**主题真相源在 `src/styles/themes`**（shadcn 语义 token，换主题只改主题文件）；Tailwind 变量映射（`@theme inline`，`--color-*` 桥接语义 token）集中在 app.css 单一真相源，主题文件只承载变量值；新增主题在 themes/ 下直接以名字命名（neutral.css、blue.css…），**经 `themes/index.css` 聚合 import + `themes/index.ts` 追加 `themeNames` + AppearanceSettings 的 label 映射（options 由 themeNames 驱动）**，运行期经 `data-theme` 切换；**主题可分完整 token 与局部覆盖两类——完整主题含全量语义 token（浅/深），局部覆盖主题基于 neutral 基底仅覆盖差异 token（如 primary/chart/sidebar），`data-theme` 未覆盖 token 回落基底值**；`@theme`/`@custom-variant`/`@apply` 等 at-rule 与 oklch 数字写法已在 stylelint 豁免（.stylelintrc.json）
- **CSP**：bits-ui 浮层组件（popover/dropdown/tooltip）经 floating-ui 内联 style 定位，生产 csp 的 style-src 必须含 `'unsafe-inline'`（已配置，勿删）；封面预览经后端 `read_image_as_data_url` 返回 `data:` URL，不走 `asset:` 协议，`tauri.conf.json` 的 `img-src` 已含 `asset: http://asset.localhost data:`（devCsp/csp 两侧）兼容 `convertFileSrc` 回退与 data URL——生产 csp 缺 `data:` 会导致打包后封面加载失败（`resource load failed: <img>`，dev 因不注入 CSP 不暴露），勿删
- **主题**：深色模式为 class 策略——`document.documentElement` 挂 `.dark`（styles/app.css `@custom-variant dark`）；**暗色偏好经 mode-watcher 管理**——根布局挂 `<ModeWatcher />`（应用/移除 `.dark` + `color-scheme`），偏好经 `userPrefersMode`（`system | light | dark`，持久化于 `mode-watcher-mode` key，system 走 prefers-color-scheme），切换用 `setMode`；消费组件直接 import mode-watcher（如 sonner 的 `theme={mode.current}`）
- **prettier**：prettier-plugin-tailwindcss 自动排序 Tailwind 类（`tailwindStylesheet` 指向 src/styles/app.css，插件顺序 svelte 在前）
- **eslint 配置**：`.svelte.ts` runes 模块纳入 svelte 解析器块（extraFileExtensions）；`scripts/**/*.mjs` 配置 Node globals；`src/components/ui/**` 关闭 `svelte/no-navigation-without-resolve`（按钮类组件 href 为动态绑定，规则误报）
- **质量门槛**：提交前通过 `bun run validate`（见「校验约定」）

## 校验约定

- **validate 命令**：每次修改代码后运行 `bun run validate`——包含 lint:all（eslint + stylelint + clippy -D warnings）、format:all:check（prettier + rustfmt --check）、check:rust（cargo check）、check（svelte-check）、test（vitest run）
- **单测约定**：Vitest 单测仅覆盖不涉及前后端交互与 Svelte 的纯逻辑工具（当前测试域 `libs/utils`，经 vitest.config.ts 的 include 限定）；测试文件与被测文件同目录 `*.test.ts`，显式导入 vitest API（不用 globals）；新增测试域时扩展 include；**测试统一经 `bun run test`（vitest）运行——`bun test` 是 bun 内建运行器，不加载 vitest.config.ts（define 注入缺失，package-info 测试会报 `__APP_PKG__ is not defined`），勿用**
- **提交门禁**：pre-commit 钩子（husky + lint-staged）自动修复暂存文件的格式；validate 作为改动完成后的最终校验

## Git 约定

- **提交规范**：提交信息遵循 Conventional Commits（英文），git-cliff 据此解析生成 changelog（cliff.toml）——类型为 `feat` / `fix` / `refactor` / `docs` / `style` / `test` / `perf` / `ci` / `chore` / `revert`，可带 scope；breaking 变更在 message 中标注 `!`
- **提交方式**：提交信息由开发者手动填写，AI 代理只完成代码改动、不代写提交

## 文档约定（README）

- **内容**：`README.md` 与 `docs/README_zh-CN.md` 均精简为三行：标题（`# ebook-studio`）+ 简介（`A cross platform desktop application designed to provide convenience for e-book manage`）+ 许可行（`[GPL-3.0-only](LICENSE)`）；不含技术栈/截图/快速开始等扩展内容
- **历史模板文档**：原模板的双语详细文档、技术栈与截图说明已归档于 `AGENTS.md`，README 仅保留最小标识
- **维护同步**：后续技术栈/插件变更同步更新 `AGENTS.md`；`README.md` 保持三行不变

## 版本发布

- **版本号同步**：`package.json`、`src-tauri/Cargo.toml`、`tauri.conf.json` 三处 version 保持一致（当前 0.1.0）；经 `scripts/bump-version.mjs` 提升——`bun run version:patch|minor|major` 按等级递增，或 `node scripts/bump-version.mjs 1.2.3` 直接指定版本，不手动改
- **发布流程**：推送 tag（如 `v0.2.0`）触发 release.yml——tauri-action 三平台构建（linux/macos/windows）+ git-cliff 生成 CHANGELOG 写入 release notes
- **产物证明**：构建产物经 `actions/attest@v4` 生成 SLSA 构建溯源证明（Sigstore 签名，Release 页面显示已验证徽章，`gh attestation verify` 可核验）；仅证明安装包本体，不含 `.sig`/`latest.json`；需工作流顶层 `id-token: write` + `attestations: write` 权限；subject-path glob 与 tauri.conf.json 的 bundle targets 保持一致，变更 targets 须同步更新 glob；已知边界——attest 失败时 release 已由 tauri-action 先行创建，job 标红提示
- **签名密钥**：自动更新安装包签名需在仓库配置 `TAURI_SIGNING_PRIVATE_KEY` secret（当前 `tauri.conf.json` 的 `plugins.updater.pubkey` 为模板占位，需 `tauri signer generate` 重新生成并填入，私钥入 secret，公钥填配置，`endpoints` 已指向 `https://github.com/AlfredClark/ebook-studio/releases/latest/download/latest.json`）
- **应用签名（macOS/Windows）**：模板默认**未配置**，留空位由使用者按需接入——macOS 未签名/未公证的应用受 Gatekeeper 拦截且 updater 不可用（tauri-action 经 `APPLE_CERTIFICATE` / `APPLE_CERTIFICATE_PASSWORD` / `APPLE_SIGNING_IDENTITY` / `APPLE_ID` / `APPLE_PASSWORD` / `APPLE_TEAM_ID` 环境变量启用签名与公证）；Windows 未签名触发 SmartScreen 警告（`tauri.conf.json` 的 `bundle.windows` 配 `certificateThumbprint` / `digestAlgorithm` / `timestampUrl`，或 Azure Trusted Signing）；签名证书与密钥一律入仓库 secret，不落库

## 新增功能流程

- **后端**：`features/` 写业务逻辑（返回 `AppResult<T>`）→ `commands/` 写命令（校验 + 调 features + 转 `Response<T>`）→ 追加 `invoke_handlers!` 宏 → 文案加 `locales/*.yml`；涉及新能力时同步 Cargo.toml 依赖与 capabilities 权限
- **前端**：业务逻辑写 `src/features/<功能>/`（可直接调 `invokeCommand`）→ 文案经 `m.xxx()` 并加入 `messages/*.json` → 运行 `bun run i18n:compile`；新 UI 偏好经 `storeDef` + `createStoreGroup` 组装进 `settings`（stores/index.ts），偏好残留校验等初始化在模块作用域显式执行（如 settings.ts 启动校验）；跨组件共享的瞬时状态（非持久化偏好）用 `state.svelte.ts` runes 模块；UI 基础组件经 `bunx shadcn-svelte add <name>` 拉取到 `$components/ui`（不覆盖已有组件）
- **初始化状态**：模板演示模块已移除，首个业务 `projects` 已落地（前端 `features/projects/{core,types,mock,index}.ts` + 后端 `src-tauri/src/features/projects.rs` + `commands/projects.rs`，真相源 `APPDATA/Projects/<uuid>/metadata.json + sources/{cover.<ext>,content.txt}`，Dublin 字段 `subjects` 以 `/` 切分、`description` 以换行切分存 `Vec<String>`，`home_*` 文案已替换为 ebook-studio 简介；列表支持单改/批删，详情含封面预览与更多信息折叠）+ `inspect` 已落地（`features/inspect/{core,types}` + `features/inspect.rs` + `commands/inspect.rs`，`content.txt` 结构 `volume_chapters/pure_chapters/volume_intro` 自动识别与 `missing/duplicate/out_of_order/empty_volume` 全量校核，中文数字经 `chinese-number`，支持自定义 `volumeRegex/chapterRegex`，`reorder_chapters` 按 `effective/is_reset` 保留标题与风格重排卷`1..n`/章`1..k`或`1..n`并重扫；工作空间 `workspace/[id]` 脱离主布局，标题栏 `Breadcrumb` 支持阶段切换，`inspect` 页初始居中封面信息，检后左右分栏 `Resizable` + `hasReorder?ConfirmDialog(Wrench) + 打开文件夹`）+ `split` 已落地（`features/split/{core,types}` + `features/split.rs` + `commands/split.rs`，`content.txt→split.json` 含 `order` 回退 `1..n`、`title` 去编号 `trim`、`contents/intro` 行 `trim` 去空，`split_content/get_split_content/save_split_chapter`；`split` 页左树 `SvelteSet` 折叠 + 搜索过滤 + `selected/draft` + 右 `Textarea` 保存） + `clean` 已落地（`features/clean/{core,types}` + `features/clean.rs` + `commands/clean.rs`，`split.json` 行级 OR 筛查 `CleanFilters+customRegex`→`CleanResult{matched,issues}`，`clean` 页 `Checkbox Filters + customRegex + Filter/Reset` 仅点击触发，左侧 `matchedSet` 过滤、右侧 `issues` 明细与保存复用 `split` 落盘） + `build` 已落地（`features/build/{core,types}` + `features/build.rs` + `commands/build.rs`，`split.json+metadata.json→build/<书名>/` 未压缩 EPUB + `templates/EPUB33-NOVEL` 渲染，`sanitize_title/format_number_display/apply_title_format`，`BuildResult{epubPath,files,bookTitle}` 与 `BuildFile` 树，`get_build/build_epub/remove_build/read_build_file/write_build_file/get_build_path/get_format/format_build_all`；`build` 页格式面板 `chapter/volumeTitleFormat + numberFormat(arabic|arabic_padded|chinese_lower|chinese_upper)` 含 `format.json` 持久化 + `Wand` 整目录 2 空格格式化 `xhtml/opf/css`（`quick-xml` + `styloria` Tokenizer span 保留式美化，空格/注释不删，失败不落盘） + `Resizable` 树+编辑、文本/二进制区分与 `dirty` 守卫，`dcterms:modified` `Secs` 无毫秒、`manifest/spine` 交错、`landmarks` 仅 `cover/titlepage/bodymatter`）+ `package` 已落地（`features/package/{core,types}` + `features/package.rs` + `commands/package.rs`，`build/<书名>/+split.json+metadata.json+format.json→outputs/${书名}.{epub/txt/封面}`，均以 `sanitize_title` 命名并清空 `outputs` 重建，`txt` 按 `format.json` 带卷章编号 + `zip::ZipWriter` `mimetype` Stored 首条 + `epubveri::validate_path` EPUB3.3 校验产 `PackageResult{epubPath,txtPath,coverPath,files,bookTitle,verified,issues,epubVersion}`；`package` 页 `build` 门控 + `package_action` 三文件说明 + `Badge verified` + `files+issues ScrollArea` + `verify(ShieldCheck, 异步 spawn_blocking, 轻量 get_package) + repackage ConfirmDialog + open outputs`）；`tauri.conf.json` 身份（identifier/productName/title/endpoints/version）与 `package.json`/`Cargo.toml`/`cliff.toml` 已同步为 `ebook-studio` 0.1.0，`updater.pubkey` 待手动替换（`tauri signer generate`）。
- **收尾**：运行 `bun run validate` 通过后，由开发者按 Conventional Commits 手动提交
