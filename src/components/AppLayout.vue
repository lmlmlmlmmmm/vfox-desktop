<script setup lang="ts">
/// 应用主布局
///
/// 必须放在 `NConfigProvider` 内部 —— 用 `useThemeVars()` 拿当前主题色，
/// 写到 `:root` 的全局 CSS 变量上，让侧边栏、主区域等"非 naive-ui 容器"
/// 也能跟着主题切换变色。
import { watchEffect } from 'vue'
import { NSpin, useThemeVars } from 'naive-ui'
import { useAppStore } from '@/stores/app'
import AppSidebar from '@/components/AppSidebar.vue'

const app = useAppStore()
const themeVars = useThemeVars()

watchEffect(() => {
  const v = themeVars.value
  const root = document.documentElement
  root.style.setProperty('--app-bg', v.bodyColor)
  root.style.setProperty('--app-surface', v.cardColor)
  root.style.setProperty('--app-fg', v.textColor1)
  root.style.setProperty('--app-border', v.dividerColor)
  root.style.setProperty('--app-hover', v.hoverColor)
})
</script>

<template>
  <div class="app-shell">
    <template v-if="app.checking">
      <div class="app-boot">
        <NSpin size="large" />
        <p>正在检测 vfox…</p>
      </div>
    </template>

    <template v-else-if="!app.vfoxStatus.installed">
      <router-view />
    </template>

    <template v-else>
      <AppSidebar />
      <main class="app-main">
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </main>
    </template>
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  background: var(--app-bg);
  color: var(--app-fg);
  transition: background 0.2s ease, color 0.2s ease;
}

.app-main {
  flex: 1;
  min-width: 0;
  overflow: auto;
  padding: 24px 32px;
  background: var(--app-bg);
  color: var(--app-fg);
}

.app-boot {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  opacity: 0.8;
  background: var(--app-bg);
  color: var(--app-fg);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.18s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
