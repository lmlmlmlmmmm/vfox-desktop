use crate::error::AppResult;
use crate::paths;
use crate::vfox::runner;
use crate::vfox::types::VfoxStatus;

/// 启动时探测 vfox 是否安装可用
#[tauri::command]
pub async fn check_vfox_installed() -> VfoxStatus {
    match runner::detect_version().await {
        Ok(version) => VfoxStatus {
            installed: true,
            version: Some(version),
            home: paths::vfox_home().ok().map(|p| p.display().to_string()),
        },
        Err(_) => VfoxStatus {
            installed: false,
            version: None,
            home: None,
        },
    }
}

/// 在系统资源管理器中打开 vfox 数据目录
#[tauri::command]
pub async fn open_vfox_home() -> AppResult<()> {
    let p = paths::vfox_home()?;
    if !p.exists() {
        tokio::fs::create_dir_all(&p).await?;
    }
    // Windows 用 explorer.exe；其他平台让前端走 plugin-opener
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer.exe")
            .arg(p.as_os_str())
            .spawn()?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = p;
    }
    Ok(())
}
