//! 直接读取 vfox 文件系统状态
//!
//! 为什么不全走 `vfox <cmd>` CLI？
//! 因为 vfox 在 Windows 上的"当前版本"信息部分来自**进程内环境变量快照**，
//! 而 Tauri 桌面主进程的 env 在启动那一刻就冻结了，
//! 之后从 GUI spawn 的 `vfox current` 子进程会继承这个旧 env，
//! 即使 `vfox use --global` 已经更新了注册表和 symlink，
//! GUI 内部看到的"current"始终是 GUI 启动那一刻的快照。
//!
//! 解决：直接读 `~/.vfox/sdks/<sdk>` symlink 的目标。
//! 该 symlink 是 vfox 在 Windows 上"当前激活版本"的唯一文件系统真相源。

use crate::error::AppResult;
use crate::paths::{vfox_cache_dir, vfox_plugin_dir, vfox_sdks_dir};
use crate::vfox::types::{CurrentVersion, SdkVersions};
use std::collections::HashSet;
use std::path::Path;

/// 扫描 `~/.vfox/sdks/`，对每个 symlink 解析当前激活版本
///
/// symlink 目标形如：`~/.vfox/cache/<sdk>/v-<version>/<sdk>-<version>`，
/// 版本号取倒数第二级目录名去掉 `v-` 前缀。
pub async fn read_current_versions() -> AppResult<Vec<CurrentVersion>> {
    let sdks_dir = vfox_sdks_dir()?;
    if !sdks_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = tokio::fs::read_dir(&sdks_dir).await?;
    let mut out = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let sdk_name = match path.file_name().and_then(|s| s.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        // 读 symlink；非 symlink（普通目录/文件）跳过
        let target = match tokio::fs::read_link(&path).await {
            Ok(t) => t,
            Err(_) => continue,
        };

        // 解析版本：父目录形如 `v-<version>`
        let parent_name = target
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str());

        let version = match parent_name.and_then(|n| n.strip_prefix("v-")) {
            Some(v) => v.to_string(),
            None => continue,
        };

        out.push(CurrentVersion {
            sdk: sdk_name,
            version,
        });
    }

    Ok(out)
}

/// 列出所有已加插件 + 各自已装版本
///
/// 为什么不走 `vfox list`？因为 vfox list 只会列出**至少装了一个版本**的 SDK，
/// 只 `vfox add` 了但还没 `vfox install` 的插件会被漏掉，导致用户
/// 在 SDK 页面看不到该 SDK 也就触发不了首次安装。
///
/// 实现：
/// - SDK 列表来源：`~/.vfox/plugin/` 下的目录名（已加的插件）
/// - 每个 SDK 的已装版本：`~/.vfox/cache/<sdk>/v-*` 子目录
/// - 未装任何版本的 SDK 返回空版本数组
pub async fn read_installed_sdks() -> AppResult<Vec<SdkVersions>> {
    let plugin_dir = vfox_plugin_dir()?;
    if !plugin_dir.exists() {
        return Ok(Vec::new());
    }
    let cache_dir = vfox_cache_dir()?;

    let mut entries = tokio::fs::read_dir(&plugin_dir).await?;
    let mut out = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let ftype = match entry.file_type().await {
            Ok(t) => t,
            Err(_) => continue,
        };
        if !ftype.is_dir() {
            continue;
        }
        let name = match entry.file_name().to_str().map(String::from) {
            Some(n) => n,
            None => continue,
        };

        // 该 SDK 的已装版本（可能为空 = 已加插件但没装任何版本）
        let sdk_cache = cache_dir.join(&name);
        let versions = if sdk_cache.exists() {
            read_versions_under(&sdk_cache).await.unwrap_or_default()
        } else {
            Vec::new()
        };

        out.push(SdkVersions { sdk: name, versions });
    }

    // 字典序，保证 UI 列表稳定
    out.sort_by(|a, b| a.sdk.cmp(&b.sdk));
    Ok(out)
}

/// 扫某个 SDK 的 cache 目录，列出 `v-<version>` 子目录里的 version 部分
async fn read_versions_under(dir: &Path) -> AppResult<Vec<String>> {
    let mut entries = tokio::fs::read_dir(dir).await?;
    let mut out = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let ftype = match entry.file_type().await {
            Ok(t) => t,
            Err(_) => continue,
        };
        if !ftype.is_dir() {
            continue;
        }
        let name = match entry.file_name().to_str().map(String::from) {
            Some(n) => n,
            None => continue,
        };
        if let Some(ver) = name.strip_prefix("v-") {
            out.push(ver.to_string());
        }
    }
    // 按字典序排序；不做版本号语义排序避免引依赖，
    // 用户体验上影响有限（UI 上已装版本不多）
    out.sort();
    Ok(out)
}

/// 列出本机已加（`vfox add` 过）的插件名集合
///
/// 即 `~/.vfox/plugin/<name>/` 下所有目录条目的名字。
pub async fn installed_plugin_names() -> AppResult<HashSet<String>> {
    let plugin_dir = vfox_plugin_dir()?;
    let mut out = HashSet::new();
    if !plugin_dir.exists() {
        return Ok(out);
    }
    let mut entries = tokio::fs::read_dir(&plugin_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let ftype = match entry.file_type().await {
            Ok(t) => t,
            Err(_) => continue,
        };
        if !ftype.is_dir() {
            continue;
        }
        if let Some(name) = entry.file_name().to_str() {
            out.insert(name.to_string());
        }
    }
    Ok(out)
}
