use serde::{Deserialize, Serialize};

/// SDK 当前生效版本（来自 `vfox current`）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentVersion {
    pub sdk: String,
    pub version: String,
}

/// 已安装 SDK 的全部版本（来自 `vfox list`）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkVersions {
    pub sdk: String,
    pub versions: Vec<String>,
}

/// 插件市场条目（来自 `vfox available` + 文件系统交叉验证）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailablePlugin {
    pub name: String,
    /// 是否由 vfox 官方维护（来自 vfox available 输出的 ✓/✗ 标记，
    /// 实际语义是 "version-fox 组织维护" vs "第三方插件"）
    pub official: bool,
    /// 是否在本机已 add（来自扫描 `~/.vfox/plugin/<name>/` 真实存在）
    pub installed: bool,
    pub homepage: String,
}

/// `vfox search <sdk>` 输出的一行
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchVersion {
    pub version: String,
    pub is_lts: bool,
    pub is_installed: bool,
}

/// 插件详细信息（来自 `vfox info <plugin>`）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub homepage: String,
    pub description: String,
    /// vfox info 输出里 Notes 字段后的多行原文
    pub notes: String,
    /// 从 notes 里启发式抽取的"可用筛选词"
    ///
    /// 例：java 的 notes 形如 `Oracle: x.y.z-oracle`，抽出 oracle/open/graal/tem/zulu。
    /// 用于安装弹窗里的"分类筛选"按钮组。空 Vec 表示该插件无分类概念。
    pub distributions: Vec<String>,
}

/// vfox 安装状态（启动探测用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VfoxStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub home: Option<String>,
}

/// 流式输出事件 payload
#[derive(Debug, Clone, Serialize)]
pub struct StreamLine {
    /// 业务关联 ID，由调用方生成，前端用此匹配自己的对话框
    pub job_id: String,
    /// stdout / stderr
    pub stream: &'static str,
    pub line: String,
}

/// 流式任务结束事件
#[derive(Debug, Clone, Serialize)]
pub struct StreamDone {
    pub job_id: String,
    pub success: bool,
    pub code: Option<i32>,
}
