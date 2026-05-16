import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import * as api from '@/api'
import type { CurrentVersion, SdkVersions } from '@/types'

/// SDK 状态：已安装版本列表 + 当前生效版本
export const useVfoxStore = defineStore('vfox', () => {
  const sdks = ref<SdkVersions[]>([])
  const current = ref<CurrentVersion[]>([])
  const loading = ref(false)

  /// 以 sdk 名为键的"当前版本"映射
  const currentMap = computed<Record<string, string>>(() => {
    const m: Record<string, string> = {}
    for (const c of current.value) m[c.sdk] = c.version
    return m
  })

  async function refresh() {
    loading.value = true
    try {
      // 并发：list 和 current 是独立读操作
      const [list, cur] = await Promise.all([api.listSdks(), api.currentVersions()])
      sdks.value = list
      current.value = cur
    } finally {
      loading.value = false
    }
  }

  async function useVersion(sdk: string, version: string) {
    await api.useSdk(sdk, version)
    await refresh()
  }

  async function uninstall(sdk: string, version: string) {
    await api.uninstallSdk(sdk, version)
    await refresh()
  }

  return { sdks, current, currentMap, loading, refresh, useVersion, uninstall }
})
