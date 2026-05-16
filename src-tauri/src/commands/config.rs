use crate::config::{model::VfoxConfig, read_config, write_config};
use crate::error::AppResult;

/// 读取 vfox 配置
#[tauri::command]
pub async fn get_config() -> AppResult<VfoxConfig> {
    read_config().await
}

/// 保存 vfox 配置
#[tauri::command]
pub async fn save_config(config: VfoxConfig) -> AppResult<()> {
    write_config(&config).await
}
