use serde::{Deserialize, Serialize};

/// 对应 `~/.vfox/config.yaml` 的根结构
///
/// 来自 vfox 1.0.8 默认配置：
/// ```yaml
/// proxy: { url: "", enable: false }
/// storage: { sdkPath: "" }
/// registry: { address: "" }
/// legacyVersionFile: { enable: true, strategy: specified }
/// cache: { availableHookDuration: 12h }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct VfoxConfig {
    pub proxy: ProxyConfig,
    pub storage: StorageConfig,
    pub registry: RegistryConfig,
    #[serde(rename = "legacyVersionFile")]
    pub legacy_version_file: LegacyVersionFileConfig,
    pub cache: CacheConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ProxyConfig {
    pub url: String,
    pub enable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct StorageConfig {
    #[serde(rename = "sdkPath")]
    pub sdk_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RegistryConfig {
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LegacyVersionFileConfig {
    pub enable: bool,
    pub strategy: String,
}

impl Default for LegacyVersionFileConfig {
    fn default() -> Self {
        Self {
            enable: true,
            strategy: "specified".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    /// 字符串形如 "12h" / "30m"；vfox 自行解析，桌面端不强制语义
    #[serde(rename = "availableHookDuration")]
    pub available_hook_duration: String,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            available_hook_duration: "12h".to_string(),
        }
    }
}
