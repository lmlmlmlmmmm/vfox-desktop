use crate::error::AppResult;
use crate::vfox::parser;
use crate::vfox::runner;
use crate::vfox::state;
use crate::vfox::types::{CurrentVersion, SdkVersions, SearchVersion, StreamDone, StreamLine};
use tauri::{AppHandle, Emitter};

/// 列出所有已加插件 + 各自已装版本
///
/// 直接扫 `~/.vfox/plugin/` 和 `~/.vfox/cache/`，**不调** `vfox list`。
/// 原因：vfox list 只列出"至少装了一个版本"的 SDK，会漏掉已 add 但未 install
/// 任何版本的插件 —— 那种情况下用户在 GUI 上看不到 SDK，触发不了首次安装。
#[tauri::command]
pub async fn list_sdks() -> AppResult<Vec<SdkVersions>> {
    state::read_installed_sdks().await
}

/// 当前生效版本
///
/// 直接读 `~/.vfox/sdks/<sdk>` symlink 目标，**不调** `vfox current`。
/// 原因详见 [`crate::vfox::state`] 模块注释（env 快照问题）。
#[tauri::command]
pub async fn current_versions() -> AppResult<Vec<CurrentVersion>> {
    state::read_current_versions().await
}

/// 查询某个 SDK 的所有可用远端版本
///
/// 走 `vfox search <sdk> [filter]`；non-TTY 下自动输出 plain text。
///
/// `filter` 是可选的插件特定筛选关键字：
/// - java 插件：oracle / tem / zulu / graal / open（不传默认 OpenJDK）
/// - 其他插件可能是 stable / lts / nightly 等，语义由插件定义
#[tauri::command]
pub async fn search_versions(
    sdk: String,
    filter: Option<String>,
) -> AppResult<Vec<SearchVersion>> {
    let mut args = vec!["search", sdk.as_str()];
    let f = filter.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty());
    if let Some(ff) = f {
        args.push(ff);
    }
    let out = runner::run_collect(&args).await?;
    Ok(parser::parse_search(&out.stdout))
}

/// 切换 SDK 到指定版本（全局生效）
///
/// 等价 CLI: `vfox use -g <sdk>@<version>`
///
/// 关键：vfox use 默认作用域是 `--session`（只影响当前 shell 进程），
/// 桌面 GUI 每次调用都是独立的临时进程，session 作用域等于没切换。
/// 桌面端的语义只能是"全局切换"，因此强制 `--global`。
#[tauri::command]
pub async fn use_sdk(sdk: String, version: String) -> AppResult<()> {
    let target = format!("{sdk}@{version}");
    runner::run_collect(&["use", "--global", &target]).await?;
    Ok(())
}

/// 卸载 SDK 的某个版本
///
/// 等价 CLI: `vfox uninstall <sdk>@<version>`
#[tauri::command]
pub async fn uninstall_sdk(sdk: String, version: String) -> AppResult<()> {
    let target = format!("{sdk}@{version}");
    runner::run_collect(&["uninstall", &target]).await?;
    Ok(())
}

/// 流式安装某个 SDK 版本
///
/// 前端通过监听 `vfox://stream-line` 和 `vfox://stream-done` 事件，
/// 用 `job_id` 匹配自己的进度对话框。
#[tauri::command]
pub async fn install_sdk(
    app: AppHandle,
    job_id: String,
    sdk: String,
    version: String,
) -> AppResult<()> {
    let target = format!("{sdk}@{version}");
    let app_handle = app.clone();
    let jid = job_id.clone();

    let code = runner::run_stream(&["install", &target], move |stream, line| {
        let _ = app_handle.emit(
            "vfox://stream-line",
            StreamLine {
                job_id: jid.clone(),
                stream,
                line,
            },
        );
    })
    .await?;

    let _ = app.emit(
        "vfox://stream-done",
        StreamDone {
            job_id,
            success: code == 0,
            code: Some(code),
        },
    );
    Ok(())
}
