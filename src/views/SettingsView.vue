<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import {
  NButton,
  NCard,
  NCode,
  NCollapse,
  NCollapseItem,
  NForm,
  NFormItem,
  NProgress,
  NRadioButton,
  NRadioGroup,
  NSpace,
  NTag,
  NText,
  useMessage,
} from 'naive-ui'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { openUrl } from '@tauri-apps/plugin-opener'
import * as api from '@/api'
import { useAppStore } from '@/stores/app'
import type { AppUpdateInfo, UpdateProgress } from '@/types'

const app = useAppStore()
const msg = useMessage()

// 自身版本（启动后从后端拉一次，整个进程生命周期内不变）
const appVer = ref<string>('')

// 更新检查状态机：idle → checking → result(latest/has_update)
type CheckPhase = 'idle' | 'checking' | 'checked'
const phase = ref<CheckPhase>('idle')
const info = ref<AppUpdateInfo | null>(null)

// 下载状态
const downloading = ref(false)
const progress = ref<UpdateProgress | null>(null)
let unlistenProgress: UnlistenFn | null = null

onMounted(async () => {
  try {
    appVer.value = await api.appVersion()
  } catch (e) {
    appVer.value = '未知'
  }
  unlistenProgress = await api.onUpdateProgress(p => {
    progress.value = p
  })
})

onUnmounted(() => {
  unlistenProgress?.()
})

const percent = computed(() => {
  const p = progress.value
  if (!p || !p.total) return 0
  return Math.min(100, Math.round((p.downloaded / p.total) * 100))
})

function formatMB(bytes: number | null | undefined): string {
  if (!bytes) return '-'
  return (bytes / 1024 / 1024).toFixed(1) + ' MB'
}

async function onCheck() {
  phase.value = 'checking'
  info.value = null
  try {
    info.value = await api.checkAppUpdate()
    phase.value = 'checked'
  } catch (e) {
    phase.value = 'idle'
    msg.error('检查更新失败：' + String(e))
  }
}

async function onDownload() {
  if (!info.value?.download_url) {
    msg.error('该 release 缺少 exe 资源，无法自动更新')
    return
  }
  downloading.value = true
  progress.value = null
  try {
    // 调用成功意味着新进程已起、当前进程即将退出 —— 走不到这里也正常
    await api.downloadAndApplyAppUpdate(info.value.download_url)
  } catch (e) {
    downloading.value = false
    msg.error('更新失败：' + String(e))
  }
}

async function onOpenReleases() {
  const url =
    info.value?.release_url ??
    'https://github.com/lmlmlmlmmmm/vfox-desktop/releases'
  await openUrl(url)
}
</script>

<template>
  <div class="page">
    <h2 class="page-title">应用设置</h2>

    <NSpace vertical :size="16">
      <NCard title="外观">
        <NForm label-placement="left" :label-width="100">
          <NFormItem label="主题">
            <NRadioGroup v-model:value="app.themePref">
              <NRadioButton value="system">跟随系统</NRadioButton>
              <NRadioButton value="light">浅色</NRadioButton>
              <NRadioButton value="dark">深色</NRadioButton>
            </NRadioGroup>
          </NFormItem>
        </NForm>
      </NCard>

      <NCard title="应用更新">
        <NSpace vertical :size="12">
          <!-- 头部：当前版本 + 检查按钮 + Releases 链接 -->
          <NSpace align="center" :size="12">
            <NText>当前版本：</NText>
            <NTag :bordered="false" size="small">v{{ appVer || '...' }}</NTag>
            <NButton
              size="small"
              type="primary"
              :loading="phase === 'checking'"
              :disabled="downloading"
              @click="onCheck"
            >
              检查更新
            </NButton>
            <NButton size="small" tertiary @click="onOpenReleases">
              打开 Releases
            </NButton>
          </NSpace>

          <!-- 检查结果 -->
          <template v-if="phase === 'checked' && info">
            <!-- 已是最新 -->
            <NText v-if="!info.has_update" depth="3" style="font-size:13px">
              ✓ 当前已经是最新版本（v{{ info.latest }}）
            </NText>

            <!-- 有新版本 -->
            <template v-else>
              <NSpace align="center" :size="8">
                <NTag type="success" size="small" :bordered="false">新版本</NTag>
                <NText>v{{ info.current }} → v{{ info.latest }}</NText>
                <NText depth="3" style="font-size:12px">
                  ({{ formatMB(info.asset_size) }})
                </NText>
              </NSpace>

              <NCollapse v-if="info.release_notes">
                <NCollapseItem title="发布说明" name="notes">
                  <NCode :code="info.release_notes" language="markdown" word-wrap />
                </NCollapseItem>
              </NCollapse>

              <NSpace align="center" :size="12">
                <NButton
                  type="primary"
                  :loading="downloading"
                  :disabled="!info.download_url"
                  @click="onDownload"
                >
                  {{ downloading ? '下载中…' : '下载并重启' }}
                </NButton>
                <NText v-if="downloading && progress" depth="3" style="font-size:12px">
                  {{ formatMB(progress.downloaded) }}
                  <template v-if="progress.total">
                    / {{ formatMB(progress.total) }}
                  </template>
                </NText>
              </NSpace>

              <NProgress
                v-if="downloading"
                type="line"
                :percentage="percent"
                :indicator-placement="'inside'"
                :height="14"
              />
            </template>
          </template>
        </NSpace>
      </NCard>

      <NCard title="关于">
        <NSpace vertical :size="8">
          <NText>vfox Desktop — 一个 vfox 的 Windows 桌面 GUI 壳</NText>
          <NText depth="3" style="font-size:12px">
            vfox 版本：{{ app.vfoxStatus.version ?? '未检测到' }}
          </NText>
          <NText depth="3" style="font-size:12px">
            数据目录：{{ app.vfoxStatus.home ?? '-' }}
          </NText>
        </NSpace>
      </NCard>
    </NSpace>
  </div>
</template>

<style scoped>
.page {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.page-title {
  margin: 0 0 16px;
  font-size: 20px;
  font-weight: 600;
}
</style>
