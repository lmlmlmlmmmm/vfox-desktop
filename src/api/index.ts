import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type {
  AppUpdateInfo,
  AvailablePlugin,
  CurrentVersion,
  PluginInfo,
  SdkVersions,
  SearchVersion,
  StreamDone,
  StreamLine,
  UpdateProgress,
  VfoxConfig,
  VfoxStatus,
} from '@/types'

// ---------- system ----------
export const checkVfoxInstalled = () =>
  invoke<VfoxStatus>('check_vfox_installed')

export const openVfoxHome = () => invoke<void>('open_vfox_home')

// ---------- sdk ----------
export const listSdks = () => invoke<SdkVersions[]>('list_sdks')
export const currentVersions = () => invoke<CurrentVersion[]>('current_versions')
export const searchVersions = (sdk: string, filter?: string) =>
  invoke<SearchVersion[]>('search_versions', { sdk, filter: filter || null })

export const useSdk = (sdk: string, version: string) =>
  invoke<void>('use_sdk', { sdk, version })

export const uninstallSdk = (sdk: string, version: string) =>
  invoke<void>('uninstall_sdk', { sdk, version })

export const installSdk = (jobId: string, sdk: string, version: string) =>
  invoke<void>('install_sdk', { jobId, sdk, version })

// ---------- plugin ----------
export const listAvailablePlugins = () =>
  invoke<AvailablePlugin[]>('list_available_plugins')

export const pluginInfo = (name: string) =>
  invoke<PluginInfo>('plugin_info', { name })

export const addPlugin = (jobId: string, name: string) =>
  invoke<void>('add_plugin', { jobId, name })

export const removePlugin = (jobId: string, name: string) =>
  invoke<void>('remove_plugin', { jobId, name })

export const updatePlugin = (jobId: string, name: string) =>
  invoke<void>('update_plugin', { jobId, name })

export const updateAllPlugins = (jobId: string) =>
  invoke<void>('update_all_plugins', { jobId })

// ---------- config ----------
export const getConfig = () => invoke<VfoxConfig>('get_config')
export const saveConfig = (config: VfoxConfig) =>
  invoke<void>('save_config', { config })

// ---------- app self-update ----------
/// 读 vfox-desktop 自身版本（编译期注入）
export const appVersion = () => invoke<string>('app_version')

/// 查 GitHub Release，与当前版本比较
export const checkAppUpdate = () => invoke<AppUpdateInfo>('check_app_update')

/// 下载新 exe 并原地替换 + 重启。下载进度通过 onUpdateProgress 订阅
export const downloadAndApplyAppUpdate = (url: string) =>
  invoke<void>('download_and_apply_app_update', { url })

/// 订阅自更新下载进度事件
export async function onUpdateProgress(
  cb: (p: UpdateProgress) => void,
): Promise<UnlistenFn> {
  return listen<UpdateProgress>('app://update-progress', e => cb(e.payload))
}

// ---------- 流式事件订阅 ----------
/// 订阅一次性流式任务的输出和结束事件
///
/// 调用方传入唯一 jobId，组件挂载时调用本函数获取 unlisten，卸载时清理
export async function subscribeJob(
  jobId: string,
  onLine: (l: StreamLine) => void,
  onDone: (d: StreamDone) => void,
): Promise<UnlistenFn> {
  const unlistenLine = await listen<StreamLine>('vfox://stream-line', e => {
    if (e.payload.job_id === jobId) onLine(e.payload)
  })
  const unlistenDone = await listen<StreamDone>('vfox://stream-done', e => {
    if (e.payload.job_id === jobId) onDone(e.payload)
  })
  return () => {
    unlistenLine()
    unlistenDone()
  }
}

/// 生成简单 job_id（时间戳 + 随机数即可，单进程内唯一就行）
export function genJobId(): string {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
}
