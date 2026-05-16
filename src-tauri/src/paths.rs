use crate::error::{AppError, AppResult};
use std::path::PathBuf;

/// 定位 vfox 数据目录 `~/.vfox`
///
/// vfox 1.0.x 在 Windows 下默认使用 `%USERPROFILE%\.vfox`。
/// 后续若 vfox 支持 `VFOX_HOME` 环境变量，可在此优先读取。
pub fn vfox_home() -> AppResult<PathBuf> {
    if let Ok(p) = std::env::var("VFOX_HOME") {
        return Ok(PathBuf::from(p));
    }
    let home = dirs::home_dir().ok_or_else(|| AppError::Other("无法定位用户主目录".into()))?;
    Ok(home.join(".vfox"))
}

/// vfox 的 `config.yaml` 路径
pub fn vfox_config_path() -> AppResult<PathBuf> {
    Ok(vfox_home()?.join("config.yaml"))
}

/// 已安装的插件目录 `~/.vfox/plugin`
pub fn vfox_plugin_dir() -> AppResult<PathBuf> {
    Ok(vfox_home()?.join("plugin"))
}

/// 已下载的 SDK 目录 `~/.vfox/sdks`
pub fn vfox_sdks_dir() -> AppResult<PathBuf> {
    Ok(vfox_home()?.join("sdks"))
}

/// SDK 版本物理存储目录 `~/.vfox/cache`
///
/// 真实安装位置：`cache/<sdk>/v-<version>/`。
/// `sdks/<sdk>` 只是指向当前激活版本的 symlink。
pub fn vfox_cache_dir() -> AppResult<PathBuf> {
    Ok(vfox_home()?.join("cache"))
}
