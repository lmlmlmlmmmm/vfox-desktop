//! 应用自更新
//!
//! 当前发布形态是 portable 单文件 exe（`vfox-desktop.exe`），不走 Tauri 官方
//! plugin-updater（它默认绑定 NSIS/MSI bundle 和签名密钥）。这里走自实现：
//!
//! 1. 通过 GitHub Release API 查 latest 版本，与 CARGO_PKG_VERSION 比较
//! 2. 下载新 exe 到临时目录（流式，实时推送进度事件）
//! 3. 把当前 exe 重命名为 `*.old`（Windows 允许 rename 运行中的 exe）
//! 4. 把新 exe move 到原路径
//! 5. spawn 新 exe，当前进程 exit
//!
//! `.old` 文件由下次启动时 [`cleanup_stale_old_exe`] 清理。
//!
//! 仓库坐标硬编码 —— 自更新本身就强耦合 release 来源，没必要做成配置。
const REPO_OWNER: &str = "lmlmlmlmmmm";
const REPO_NAME: &str = "vfox-desktop";

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;

/// 应用版本对比结果
#[derive(Debug, Clone, Serialize)]
pub struct AppUpdateInfo {
    pub current: String,
    pub latest: String,
    pub has_update: bool,
    /// release 页面 URL，给"查看变更日志"按钮跳转用
    pub release_url: String,
    /// 新 exe 的直链下载 URL；无 .exe asset 时为 None
    pub download_url: Option<String>,
    /// GitHub 上的 release notes（markdown 原文）
    pub release_notes: String,
    /// asset 大小（字节），用于进度条总量预估；无则为 None
    pub asset_size: Option<u64>,
}

/// 下载进度事件 payload
#[derive(Debug, Clone, Serialize)]
pub struct UpdateProgress {
    pub downloaded: u64,
    /// 服务器返回的 Content-Length；流式响应里可能缺失
    pub total: Option<u64>,
}

/// GitHub Release API 响应（只取需要的字段）
#[derive(Debug, Deserialize)]
struct GhRelease {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    body: String,
    assets: Vec<GhAsset>,
}

#[derive(Debug, Clone, Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

/// 返回 vfox-desktop 自身版本（编译期注入）
#[tauri::command]
pub fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 查询 GitHub Release，与当前版本比较
#[tauri::command]
pub async fn check_app_update() -> AppResult<AppUpdateInfo> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        REPO_OWNER, REPO_NAME
    );

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        // GitHub API 强制要求 UA
        .user_agent(concat!("vfox-desktop/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| AppError::Other(format!("构造 HTTP 客户端失败: {e}")))?;

    let resp = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| AppError::Other(format!("访问 GitHub 失败: {e}")))?;

    if !resp.status().is_success() {
        return Err(AppError::Other(format!(
            "GitHub API 返回 {}",
            resp.status()
        )));
    }

    let release: GhRelease = resp
        .json()
        .await
        .map_err(|e| AppError::Other(format!("解析 release JSON 失败: {e}")))?;

    let current = env!("CARGO_PKG_VERSION").to_string();
    // tag 形如 v0.1.0，对比时去掉前缀
    let latest = release.tag_name.trim_start_matches('v').to_string();

    // 找出 .exe asset（按 workflow 配置，名字就是 vfox-desktop.exe）
    let exe_asset = release
        .assets
        .iter()
        .find(|a| a.name.ends_with(".exe"))
        .cloned()
        .or_else(|| release.assets.first().cloned());

    Ok(AppUpdateInfo {
        has_update: is_newer(&latest, &current),
        current,
        latest,
        release_url: release.html_url,
        download_url: exe_asset.as_ref().map(|a| a.browser_download_url.clone()),
        release_notes: release.body,
        asset_size: exe_asset.as_ref().map(|a| a.size),
    })
}

/// 比较语义化版本号 `latest > current`
///
/// 支持 `major.minor.patch`，多余段（如 -beta）按字典序回退。
/// 自己实现而不引入 semver 依赖：发布版本号始终由我们控制，格式可预期。
fn is_newer(latest: &str, current: &str) -> bool {
    let parse = |s: &str| -> Vec<u32> {
        s.split(|c: char| c == '.' || c == '-')
            .take(3)
            .map(|seg| seg.parse::<u32>().unwrap_or(0))
            .collect()
    };
    let a = parse(latest);
    let b = parse(current);
    for i in 0..a.len().max(b.len()) {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        if x != y {
            return x > y;
        }
    }
    false
}

/// 流式下载新 exe 到临时目录，下载完成后立即原地替换并重启
///
/// 进度通过 `app://update-progress` 事件推送，前端用同一事件名监听。
#[tauri::command]
pub async fn download_and_apply_app_update(
    app: AppHandle,
    url: String,
) -> AppResult<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60 * 10))
        .user_agent(concat!("vfox-desktop/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| AppError::Other(format!("构造 HTTP 客户端失败: {e}")))?;

    let mut resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Other(format!("发起下载请求失败: {e}")))?;

    if !resp.status().is_success() {
        return Err(AppError::Other(format!(
            "下载失败，HTTP {}",
            resp.status()
        )));
    }

    let total = resp.content_length();

    // 临时文件：同目录下加 .new 后缀，省去跨盘 move
    let exe_path = std::env::current_exe()
        .map_err(|e| AppError::Other(format!("无法定位当前 exe: {e}")))?;
    let new_path = exe_path.with_extension("exe.new");

    // 清掉历史残留（上次中途失败留下的）
    let _ = tokio::fs::remove_file(&new_path).await;

    let mut file = tokio::fs::File::create(&new_path)
        .await
        .map_err(|e| AppError::Other(format!("创建临时文件失败: {e}")))?;

    let mut downloaded: u64 = 0;
    // reqwest 自带 chunk() 流式 API，避免引入 futures-util
    while let Some(chunk) = resp
        .chunk()
        .await
        .map_err(|e| AppError::Other(format!("下载分片失败: {e}")))?
    {
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        let _ = app.emit(
            "app://update-progress",
            UpdateProgress {
                downloaded,
                total,
            },
        );
    }
    file.flush().await?;
    drop(file);

    // 自替换：rename 当前 exe → .old，move 新 exe → 原路径
    let old_path = exe_path.with_extension("exe.old");
    let _ = tokio::fs::remove_file(&old_path).await; // 可能已存在

    tokio::fs::rename(&exe_path, &old_path)
        .await
        .map_err(|e| AppError::Other(format!("重命名当前 exe 失败: {e}")))?;

    if let Err(e) = tokio::fs::rename(&new_path, &exe_path).await {
        // 移动失败，尽量把旧 exe 改回来，避免桌面端"失踪"
        let _ = tokio::fs::rename(&old_path, &exe_path).await;
        return Err(AppError::Other(format!("写入新 exe 失败: {e}")));
    }

    spawn_detached(&exe_path)?;

    // 给新进程一个起步窗口，避免它还没绑定主窗口当前进程就退了
    tokio::time::sleep(Duration::from_millis(300)).await;
    app.exit(0);
    Ok(())
}

/// 启动一个完全脱离当前进程的子进程
///
/// Windows 必须用 `DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP`，否则
/// 当前进程退出时子进程会被 console 句柄牵连。
fn spawn_detached(exe: &PathBuf) -> AppResult<()> {
    let mut cmd = std::process::Command::new(exe);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }
    cmd.spawn()
        .map_err(|e| AppError::Other(format!("启动新版本失败: {e}")))?;
    Ok(())
}

/// 启动时清理上次升级残留的 `*.exe.old`
///
/// Windows 上正在运行的 exe 无法被删，所以替换流程把旧 exe 重命名为 `.old`
/// 留到下次启动统一清理。失败不报错（可能用户双击 .old 又跑起来一个）。
pub fn cleanup_stale_old_exe() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let old = exe.with_extension("exe.old");
    let _ = std::fs::remove_file(old);
    // 顺手把可能遗留的 .new 也清掉
    let stale_new = exe.with_extension("exe.new");
    let _ = std::fs::remove_file(stale_new);
}
