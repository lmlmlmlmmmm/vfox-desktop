pub mod model;

use crate::error::AppResult;
use crate::paths::vfox_config_path;
use model::VfoxConfig;
use tokio::fs;

/// 读取 `~/.vfox/config.yaml`。文件不存在时返回默认值。
pub async fn read_config() -> AppResult<VfoxConfig> {
    let p = vfox_config_path()?;
    if !p.exists() {
        return Ok(VfoxConfig::default());
    }
    let raw = fs::read_to_string(&p).await?;
    let cfg: VfoxConfig = serde_yaml::from_str(&raw).unwrap_or_default();
    Ok(cfg)
}

/// 写入 `~/.vfox/config.yaml`。写前自动备份为 `config.yaml.bak`。
pub async fn write_config(cfg: &VfoxConfig) -> AppResult<()> {
    let p = vfox_config_path()?;
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).await?;
    }
    // 写前备份（如有）
    if p.exists() {
        let bak = p.with_extension("yaml.bak");
        // 忽略备份失败（例如只读盘），不阻塞主写入
        let _ = fs::copy(&p, &bak).await;
    }
    let yaml = serde_yaml::to_string(cfg)?;
    fs::write(&p, yaml).await?;
    Ok(())
}
