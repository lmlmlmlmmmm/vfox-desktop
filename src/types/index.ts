/// 与 Rust 端 DTO 一一对应
export interface VfoxStatus {
  installed: boolean
  version: string | null
  home: string | null
}

export interface CurrentVersion {
  sdk: string
  version: string
}

export interface SdkVersions {
  sdk: string
  versions: string[]
}

export interface AvailablePlugin {
  name: string
  /// vfox 官方维护（version-fox 组织）vs 第三方
  official: boolean
  /// 本机已 add（来自 ~/.vfox/plugin/ 文件系统）
  installed: boolean
  homepage: string
}

export interface SearchVersion {
  version: string
  is_lts: boolean
  is_installed: boolean
}

export interface PluginInfo {
  name: string
  version: string
  homepage: string
  description: string
  notes: string
  /// 从 notes 启发式抽取的"可用筛选词"（如 java 的 oracle/tem/zulu）
  distributions: string[]
}

export interface VfoxConfig {
  proxy: { url: string; enable: boolean }
  storage: { sdkPath: string }
  registry: { address: string }
  legacyVersionFile: { enable: boolean; strategy: string }
  cache: { availableHookDuration: string }
}

/// 流式任务事件
export interface StreamLine {
  job_id: string
  stream: 'stdout' | 'stderr'
  line: string
}

export interface StreamDone {
  job_id: string
  success: boolean
  code: number | null
}

/// 应用自身的更新检查结果
export interface AppUpdateInfo {
  current: string
  latest: string
  has_update: boolean
  release_url: string
  /// 新 exe 直链；release 中无 .exe asset 时为 null
  download_url: string | null
  release_notes: string
  asset_size: number | null
}

/// 应用更新下载进度事件
export interface UpdateProgress {
  downloaded: number
  /// HTTP 响应可能不带 Content-Length
  total: number | null
}
