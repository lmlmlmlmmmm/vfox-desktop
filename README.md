# vfox-desktop

[vfox](https://github.com/version-fox/vfox) 的桌面 GUI 客户端。基于 Tauri 2 + Vue 3 构建，通过包装 `vfox` CLI 提供 SDK 与插件的图形化管理。

## 功能

- **SDK 管理**：查看已安装版本、切换当前版本、安装/卸载指定版本、搜索可用版本
- **插件市场**：浏览 vfox 官方插件源、安装/移除/更新插件、查看插件详情
- **vfox 配置**：图形化编辑 `~/.version-fox/config.yaml`，包括代理、SDK 安装路径、Registry 地址、兼容版本文件策略、缓存时长
- **首启检测**：未检测到 `vfox` 时给出引导界面
- **实时日志**：长任务（安装、更新等）以流式日志形式展示进度

## 技术栈

| 层 | 选型 |
| --- | --- |
| 前端框架 | Vue 3 + TypeScript + Vite |
| UI 组件库 | Naive UI |
| 状态管理 | Pinia |
| 路由 | Vue Router 4 |
| 桌面外壳 | Tauri 2（Rust） |
| 异步运行时 | Tokio |

## 目录结构

```
vfox-desktop/
├── src/                       # 前端源码
│   ├── views/                 # 页面（SDK / Plugins / Config / Settings / NotInstalled）
│   ├── components/            # 通用组件（AppLayout、AppSidebar、InstallProgressDialog）
│   ├── api/                   # Tauri invoke 封装
│   ├── stores/                # Pinia store
│   ├── router/                # 路由
│   └── types/                 # 共享类型
└── src-tauri/                 # Rust 后端
    └── src/
        ├── commands/          # Tauri 命令处理（system / sdk / plugin / config）
        ├── vfox/              # vfox CLI 封装（进程调用 + 输出解析）
        ├── config/            # config.yaml 读写
        ├── paths.rs           # 跨平台 user home 定位
        └── error.rs           # 统一错误类型
```

## 前置依赖

- 已安装 [vfox](https://github.com/version-fox/vfox) 并加入 `PATH`
- 开发环境：Node.js 18+、pnpm、Rust（含 cargo）

## 本地开发

```bash
pnpm install
pnpm tauri dev
```

## 打包

只生成主程序 exe（推荐，约 12MB）：

```bash
pnpm tauri build --no-bundle
```

产物位置：`src-tauri/target/release/vfox-desktop.exe`

同时生成 MSI / NSIS 安装包：

```bash
pnpm tauri build
```

产物位置：`src-tauri/target/release/bundle/{msi,nsis}/`

## 设计要点

- **vfox CLI 进程封装**：所有 CLI 调用集中在 `src-tauri/src/vfox/runner.rs`，统一注入 `NO_COLOR=1` 拿干净文本输出；在 Windows 下设置 `CREATE_NO_WINDOW` 防止弹出 cmd 窗口
- **两种调用模式**：
  - `run_collect`：一次性收集 stdout/stderr，用于 list/current/info 等读操作
  - `run_stream`：行级流式回调，用于 install/add/update 等长任务
- **错误模型**：`AppError` 区分 vfox 未安装、vfox 执行失败（非零退出）、IO、解析等错误，前端按类型给出差异化提示
