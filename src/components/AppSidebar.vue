<script setup lang="ts">
import { computed, h, type VNodeChild } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NIcon, NMenu, NButton, type MenuOption } from 'naive-ui'
import { useAppStore } from '@/stores/app'
import { openVfoxHome } from '@/api'

const route = useRoute()
const router = useRouter()
const app = useAppStore()

const renderIcon = (emoji: string) => (): VNodeChild =>
  h(NIcon, { size: 18 }, { default: () => emoji })

const menuOptions = computed<MenuOption[]>(() => [
  { label: 'SDK 管理', key: 'sdk', icon: renderIcon('📦') },
  { label: '插件市场', key: 'plugins', icon: renderIcon('🧩') },
  { label: 'vfox 配置', key: 'config', icon: renderIcon('⚙️') },
  { label: '应用设置', key: 'settings', icon: renderIcon('🎨') },
])

const activeKey = computed(() => (route.name as string) ?? 'sdk')

function onSelect(key: string) {
  router.push({ name: key })
}
</script>

<template>
  <aside class="sidebar">
    <div class="brand">
      <span class="brand-mark">▲</span>
      <span class="brand-name">vfox Desktop</span>
    </div>
    <div class="vfox-version" v-if="app.vfoxStatus.version">
      vfox v{{ app.vfoxStatus.version }}
    </div>

    <NMenu
      class="menu"
      :value="activeKey"
      :options="menuOptions"
      :indent="18"
      @update:value="onSelect"
    />

    <div class="footer">
      <NButton block secondary size="small" @click="openVfoxHome">
        打开数据目录
      </NButton>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 220px;
  border-right: 1px solid var(--app-border);
  display: flex;
  flex-direction: column;
  padding: 16px 0;
  box-sizing: border-box;
  background: var(--app-surface);
  color: var(--app-fg);
}

.brand {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 16px 4px;
  font-weight: 600;
  font-size: 16px;
}
.brand-mark {
  color: #18a058;
}

.vfox-version {
  padding: 0 16px 16px;
  font-size: 12px;
  opacity: 0.55;
}

.menu {
  flex: 1;
}

.footer {
  padding: 12px 16px 4px;
}
</style>
