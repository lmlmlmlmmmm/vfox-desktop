<script setup lang="ts">
import { computed, onMounted } from 'vue'
import {
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
  NLoadingBarProvider,
  darkTheme,
  zhCN,
  dateZhCN,
  useOsTheme,
  type GlobalTheme,
} from 'naive-ui'
import { useRoute, useRouter } from 'vue-router'
import { useAppStore } from '@/stores/app'
import AppLayout from '@/components/AppLayout.vue'

const app = useAppStore()
const router = useRouter()
const route = useRoute()

const osTheme = useOsTheme()
const theme = computed<GlobalTheme | null>(() => {
  const pref = app.themePref
  if (pref === 'dark') return darkTheme
  if (pref === 'light') return null
  return osTheme.value === 'dark' ? darkTheme : null
})

onMounted(async () => {
  await app.checkVfox()
  if (!app.vfoxStatus.installed) {
    router.replace({ name: 'not-installed' })
  }
})

// 已装后，如果还停在 not-installed 页，回到主页
import { watch } from 'vue'
watch(
  () => app.vfoxStatus.installed,
  installed => {
    if (installed && route.name === 'not-installed') {
      router.replace({ name: 'sdk' })
    }
  },
)
</script>

<template>
  <NConfigProvider :theme="theme" :locale="zhCN" :date-locale="dateZhCN">
    <NLoadingBarProvider>
      <NMessageProvider>
        <NDialogProvider>
          <AppLayout />
        </NDialogProvider>
      </NMessageProvider>
    </NLoadingBarProvider>
  </NConfigProvider>
</template>

<style>
html,
body,
#app {
  margin: 0;
  padding: 0;
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "PingFang SC",
    "Hiragino Sans GB", "Microsoft YaHei", sans-serif;
  /* 兜底色，AppLayout 会在挂载后用 CSS 变量覆盖 */
  background: #fff;
  color: #1f1f1f;
}
</style>
