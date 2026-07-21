# AutoInspection — 跨平台主机自动巡检桌面客户端

> 基于 Rust + Tauri v2 + Vue3 构建，整合 [Linux_Auto_Inspection](https://github.com/Aidan-996/Linux_Auto_Inspection) 与 [Windows_Auto_Inspection](https://github.com/Aidan-996/Windows_Auto_Inspection) 开源巡检脚本，一套客户端覆盖 Windows / Linux / macOS 主机巡检。

---

## 为什么做这个

运维巡检这件事，脚本早就有了，但用起来始终隔着一层命令行。Linux 要 SSH 上去敲 `./linux_inspect.sh`，Windows 要开 CMD 跑 `python win_inspection_html.py`，报告生成在哪个目录得自己找，远程主机还得手动 scp 拉回来。对不熟悉终端的人来说，这套流程本身就是门槛。

AutoInspection 要解决的就是这层门槛：**打开软件，选系统，填参数，点一下，报告直接预览导出**。巡检脚本内核不变，外面套一层可视化桌面客户端，把进程调度、日志推送、报告解析、文件管理全部交给 Rust 后端处理。

---

## 核心能力

### 巡检引擎

| 目标系统 | 脚本来源 | 语言 | 检查维度 |
|---------|---------|------|---------|
| Linux | Linux_Auto_Inspection v2.4 | Bash | 17 大类 25+ 项（CPU/内存/磁盘/网络/安全/Docker/SSL/日志等） |
| Windows | Windows_Auto_Inspection | Python | 27 项（硬件/网络/安全审计/事件日志/风险评估等） |
| macOS | 规划中 | — | 后续迭代 |

### 运行模式

- **本机巡检**：客户端所在主机直接执行，自动识别操作系统类型
- **远程巡检**（扩展）：通过 SSH 远程执行 Linux 巡检、WinRM 远程执行 Windows 巡检

### 桌面客户端功能

- 图形化操作界面，无需终端命令
- 系统类型选择（Windows / Linux / macOS 预留）
- 巡检模块自定义勾选（磁盘、内存、进程、网络、日志、系统服务等）
- 实时日志控制台，滚动展示脚本执行输出
- 任务启停控制，随时终止正在运行的巡检进程
- 巡检报告在线预览，支持 HTML / Markdown / TXT 格式
- 历史巡检记录管理，一键打开历史报告
- 软件设置：脚本路径配置、默认输出目录、主题切换
- 权限检测：Windows 管理员权限、Linux/macOS root/sudo 提示

---

## 技术架构

```
┌─────────────────────────────────────────────┐
│                  Vue3 前端                    │
│   Element Plus · Vite · Tauri IPC 调用        │
│  ┌──────────┐ ┌──────────┐ ┌──────────────┐ │
│  │ 巡检配置  │ │ 日志面板  │ │ 报告预览/导出 │ │
│  └──────────┘ └──────────┘ └──────────────┘ │
└──────────────────┬──────────────────────────┘
                   │ Tauri IPC (invoke / event)
┌──────────────────┴──────────────────────────┐
│                Rust 后端                     │
│  ┌──────────┐ ┌──────────┐ ┌──────────────┐ │
│  │ 脚本释放  │ │ 进程调度  │ │ 日志实时推送  │ │
│  └──────────┘ └──────────┘ └──────────────┘ │
│  ┌──────────┐ ┌──────────┐ ┌──────────────┐ │
│  │ 路径处理  │ │ 权限检测  │ │ 报告文件读取  │ │
│  └──────────┘ └──────────┘ └──────────────┘ │
└──────────────────┬──────────────────────────┘
                   │ std::process::Command
        ┌──────────┴──────────┐
        │                     │
   ┌────┴────┐          ┌────┴────┐
   │ PowerShell│         │  Bash   │
   │ (Windows) │         │ (Linux) │
   └────┬────┘          └────┬────┘
        │                     │
  win_inspection         linux_inspect
  _html.py               .sh
```

### 技术栈

| 层 | 技术 | 说明 |
|----|------|------|
| 前端 | Vue 3 + Vite + Element Plus | 桌面端适配的组件库，深色主题 |
| 桌面框架 | Tauri v2 | 打包单可执行文件，无额外运行时依赖 |
| 后端 | Rust | 进程调度、文件 IO、跨平台路径、权限处理 |
| 巡检脚本 | Bash / Python | 内置打包，运行时释放到工作目录 |
| 通信 | Tauri IPC | 前端 invoke 调用 Rust 命令，Rust event 推送日志 |

### 交互流程

1. 前端通过 `invoke()` 调用 Rust 后端命令（启动巡检、停止任务、读取报告等）
2. Rust 后端通过 `std::process::Command` 创建子进程执行对应平台巡检脚本
3. 异步读取子进程 stdout/stderr，通过 `app.emit()` 实时推送日志事件到前端
4. 巡检完成后，Rust 读取生成的报告文件返回前端预览
5. 前端可选导出格式，调用 Rust 写入指定目录

---

## 项目结构

```
AutoInspection/
├── src/                          # Vue3 前端源码
│   ├── App.vue                   # 根组件（侧边栏导航 + 页面路由）
│   ├── main.ts                   # 应用入口
│   ├── styles/theme.css          # 全局样式（浅色/暗色双主题 CSS 变量）
│   ├── store/app.ts              # 全局响应式状态管理
│   ├── api/inspection.ts         # Tauri IPC 调用封装
│   ├── components/               # 公共组件
│   │   ├── PermissionBanner.vue  # 权限状态横幅
│   │   ├── LogConsole.vue        # 实时执行日志控制台
│   │   └── ReportPreview.vue     # HTML 报告预览（iframe）
│   └── views/                    # 页面组件
│       ├── LocalInspection.vue   # 本机巡检
│       ├── RemoteInspection.vue  # 远程巡检
│       ├── HistoryReport.vue     # 历史报告
│       └── Settings.vue          # 系统设置
├── src-tauri/                    # Rust 后端源码
│   ├── src/
│   │   ├── lib.rs                # Tauri 命令注册 + 应用入口
│   │   ├── inspection.rs         # 巡检核心管理器
│   │   ├── models.rs             # 前后端共享数据结构
│   │   └── convert.rs            # HTML→TXT/MD 报告转换
│   ├── resources/                # 内置资源（打包进二进制）
│   │   ├── scripts/              # 巡检脚本
│   │   │   ├── linux_inspect.sh
│   │   │   └── win_inspection_html.py
│   │   └── python/               # 内置便携 Python（.gitignore）
│   ├── icons/                    # 全平台应用图标（由 npx tauri icon 自动生成）
│   ├── Cargo.toml
│   └── tauri.conf.json           # Tauri 配置（窗口/图标/资源/权限）
├── doc/                          # 依赖的开源巡检脚本参考文档
├── public/
│   └── logo.svg                  # Logo SVG 源文件
├── package.json
├── vite.config.ts
└── README.md
```

---

## 快速开始

### 环境要求

| 依赖 | 版本 | 说明 |
|------|------|------|
| Rust | 1.70+ | `rustup` 安装 stable 工具链 |
| Node.js | 18+ | 推荐使用 LTS 版本 |
| pnpm | 9+ | `npm install -g pnpm` |
| Tauri CLI | v2 | 随项目依赖安装 |
| Windows 巡检 | 内置便携 Python | **无需用户安装**，应用已打包官方 embeddable Python |

> **Windows 零依赖运行**：客户端内置官方 Windows embeddable Python（`src-tauri/resources/python/`），
> 目标机器即使未安装 Python 也能直接巡检。运行时优先使用内置解释器，缺失时才回退系统 Python。
> 该目录体积较大、已在 `.gitignore` 中忽略，克隆仓库后打包前请先执行 `bash scripts/fetch_python.sh` 下载补齐。

### 开发环境运行

```bash
# 安装前端依赖
pnpm install

# （首次）下载内置便携 Python，供 Windows 巡检零安装运行
bash scripts/fetch_python.sh

# 启动开发模式（同时启动 Vite 和 Tauri）
pnpm tauri dev
```

### 构建打包

```bash
# 构建当前平台可执行文件
pnpm tauri build

# 输出目录：src-tauri/target/release/
# Windows: .exe / .msi
# Linux: .deb / .AppImage / .rpm
# macOS: .dmg / .app
```

### 开发常用命令

```bash
# ---- 启停 ----
pnpm tauri dev          # 启动开发模式（Vite + Tauri 同时启动）
pnpm tauri build        # 构建当前平台可执行文件

# ---- 分别编译 ----
cargo check             # 仅检查 Rust 后端编译（快）
pnpm build              # 仅构建前端（vue-tsc 类型检查 + vite 打包）

# ---- 图标 ----
npx tauri icon public/logo.svg   # 从 SVG 一键生成全平台应用图标

# ---- Python ----
bash scripts/fetch_python.sh     # 下载内置便携 Python（Windows 巡检零安装运行）

# ---- 版本号 ----
# 版本号统一在 src-tauri/tauri.conf.json 中维护（同步修改 Cargo.toml / package.json）
# 前端通过 get_app_version Tauri 命令运行时读取显示

# ---- 截图 ----
# 安装 playwright-cli（仅首次）
npm install -g @playwright/cli@latest

playwright-cli open http://localhost:1422    # 打开应用
playwright-cli screenshot --filename=images/01-local-light.png  # 截图
playwright-cli eval "document.documentElement.classList.add('dark')"  # 切换深色
playwright-cli screenshot --filename=images/05-local-dark.png
playwright-cli close                        # 关闭浏览器

# ---- Git 版本管理 ----
# 发版流程：改版本号 → 提交 → 打 tag → 推送 → CI 自动构建发布

# 1. 更新版本号（三处同步）
#    - src-tauri/tauri.conf.json  ← 主版本号
#    - src-tauri/Cargo.toml
#    - package.json

# 2. 提交并打标签
git add -A
git commit -m "release: v1.0.0"
git tag v1.0.0

# 3. 推送（tag 推送后 GitHub Actions 自动触发构建）
git push origin main
git push origin v1.0.0

# ---- 本地打包 ----
bash scripts/fetch_python.sh     # Windows 需先下载内置 Python
pnpm tauri build                 # 构建当前平台安装包
# 输出：src-tauri/target/release/bundle/
```

### 发版流程

1. **更新版本号**：修改 `src-tauri/tauri.conf.json` 中的 `version` 字段（同步改 `Cargo.toml` 和 `package.json`）
2. **本地验证**：`cargo check` + `pnpm build` 确保前后端编译通过
3. **更新截图**：如需更新文档截图，用 `playwright-cli` 重新截取放入 `images/`
4. **提交打标签**：`git tag v1.0.0`，推送后 GitHub Actions 自动矩阵构建三平台安装包
5. **发布**：Actions 完成后生成 Draft Release，检查产物无误后手动 Publish

---

## 使用指南

### 本机巡检

1. 打开客户端，左侧导航选择「本机巡检」
2. 上方下拉框选择目标系统（自动检测当前操作系统并默认选中）
3. 勾选需要巡检的模块（可全选或按需选择）
4. 点击「开始巡检」，下方日志面板实时展示执行进度
5. 巡检完成后，报告预览区自动渲染结果
6. 点击「导出报告」选择格式（HTML / Markdown / TXT）和保存路径

### 远程巡检（扩展功能）

1. 左侧导航选择「远程巡检」
2. 选择目标系统类型
3. 填写连接信息：IP 地址、端口、账号、密码/密钥
4. SSH 连接 Linux 主机 / WinRM 连接 Windows 主机
5. 其余流程与本机巡检一致

### 历史报告

- 左侧导航选择「历史报告」
- 查看历次巡检记录列表（时间、主机、系统类型、状态）
- 点击任意记录可直接打开报告文件

---

## 关键设计说明

### 脚本内置与释放

巡检脚本通过 Tauri 的 `resources` 配置打包进二进制文件。首次运行时，Rust 后端将脚本从资源目录释放到可读写的工作目录（各平台用户数据目录），后续执行直接调用释放后的副本。这样做的原因：

- 打包后的资源目录在部分平台只读，无法直接执行
- 工作目录可写，方便脚本自身生成临时文件和报告
- 支持用户在设置中自定义脚本路径，覆盖内置版本

### 进程生命周期管理

Rust 后端维护巡检子进程的句柄，提供以下保障：

- **实时输出**：异步读取 stdout/stderr，逐行推送前端，不等到进程结束才显示
- **强制终止**：用户点击停止时，杀死整个进程树，避免子进程后台残留
- **超时保护**：长时间无输出的巡检任务自动判定为异常，提示用户
- **编码处理**：Windows 下 PowerShell 输出可能为 GBK，Rust 侧做编码转换避免中文乱码

### 敏感信息保护

远程巡检的连接凭据（密码、密钥）本地存储时加密处理，不明文落盘。具体方案使用操作系统的密钥管理服务：

- Windows：DPAPI / Credential Manager
- macOS：Keychain
- Linux：libsecret / GNOME Keyring

### 跨平台路径处理

Rust 后端统一处理各平台路径差异：

| 用途 | Windows | Linux / macOS |
|------|---------|---------------|
| 工作目录 | `%APPDATA%\autoinspection` | `~/.local/share/autoinspection` |
| 报告输出 | 工作目录 `\reports\` | 工作目录 `/reports/` |
| 脚本释放 | 工作目录 `\scripts\` | 工作目录 `/scripts/` |

---

## 相关项目

本项目整合以下开源巡检脚本作为巡检引擎：

- [Linux_Auto_Inspection](https://github.com/Aidan-996/Linux_Auto_Inspection) — Linux 服务器一键巡检脚本（Bash，17 大类检查维度，支持 HTML/JSON 输出）
- [Windows_Auto_Inspection](https://github.com/Aidan-996/Windows_Auto_Inspection) — Windows 系统自动巡检工具（Python，27 项检查，自动风险评估）

感谢原作者的开源贡献。

---

## 风险与注意事项

- **Windows 巡检需管理员权限**：部分检查项（审计策略、BitLocker、Defender）需要管理员权限才能完整采集，客户端启动时会检测并提示提权
- **Linux 本机巡检需 sudo**：建议以 root 或 sudo 用户运行，否则部分检查项（系统日志、服务状态）会显示为"无法获取"
- **远程巡检依赖网络**：SSH/WinRM 连接需确保网络可达、端口开放、凭据有效
- **脚本输出编码**：Windows PowerShell 默认 GBK 编码，Rust 后端已做 UTF-8 转换；如仍出现乱码，检查系统语言设置
- **进程残留防护**：异常退出时可能有子进程残留，Rust 后端提供进程清理逻辑，下次启动时自动检查并清理

---

## 开发计划

- [x] 项目初始化（Tauri v2 + Vue3 + Element Plus）
- [x] Rust 后端：脚本释放、进程调度、实时日志推送、进程终止
- [x] 前端界面：本机巡检页面（系统选择 + 模块勾选 + 日志面板 + 报告预览）
- [x] Windows 巡检脚本集成与测试
- [x] Linux 巡检脚本集成与测试
- [x] 历史报告管理（列表 + 预览 + 打开目录）
- [x] 远程巡检（Linux 通过 SSH + scp，密钥/密码认证）
- [ ] macOS 巡检脚本支持（UI 已预留入口）
- [x] 报告导出（Markdown / HTML / TXT）
- [x] 系统设置（脚本路径、输出目录、主题切换）
- [ ] 敏感信息加密存储（当前远程凭据仅运行时使用，未落盘；后续接入系统密钥库）

> 说明：本机巡检（Windows / Linux）已可端到端使用；远程巡检当前支持 Linux（SSH + scp），Windows/macOS 远程与 macOS 本机巡检为后续迭代，UI 已预留入口。

---

## License

MIT License

本项目内置的巡检脚本版权归原作者所有，各自遵循其原始开源协议。
