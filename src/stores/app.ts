import { defineStore } from 'pinia'
import { ref } from 'vue'
import * as api from '@/api'
import type { VfoxStatus } from '@/types'

/// 应用全局状态：vfox 是否安装、主题等
export const useAppStore = defineStore('app', () => {
  const vfoxStatus = ref<VfoxStatus>({ installed: false, version: null, home: null })
  const checking = ref(true)

  /// 主题：light / dark / system
  const themePref = ref<'light' | 'dark' | 'system'>('system')

  async function checkVfox() {
    checking.value = true
    try {
      vfoxStatus.value = await api.checkVfoxInstalled()
    } finally {
      checking.value = false
    }
  }

  return { vfoxStatus, checking, themePref, checkVfox }
})
