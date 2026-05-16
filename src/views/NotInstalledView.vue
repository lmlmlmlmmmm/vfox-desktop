<script setup lang="ts">
import { NCard, NButton, NSpace, NAlert } from 'naive-ui'
import { useAppStore } from '@/stores/app'
import { openUrl } from '@tauri-apps/plugin-opener'

const app = useAppStore()

async function recheck() {
  await app.checkVfox()
}

async function openSite() {
  await openUrl('https://vfox.lhan.me')
}
</script>

<template>
  <div class="wrap">
    <NCard class="card" title="未检测到 vfox" size="large">
      <NSpace vertical :size="16">
        <NAlert type="info" :show-icon="true">
          vfox 不在系统 PATH 中，桌面端无法工作。请先安装 vfox 后回到本应用。
        </NAlert>

        <div class="hint">
          <p><strong>Windows 推荐通过 Scoop 安装：</strong></p>
          <pre>scoop bucket add version-fox https://github.com/version-fox/scoop-version-fox
scoop install vfox</pre>
          <p>或访问官网下载安装包：</p>
        </div>

        <NSpace>
          <NButton type="primary" @click="openSite">访问 vfox 官网</NButton>
          <NButton @click="recheck">已安装，重新检测</NButton>
        </NSpace>
      </NSpace>
    </NCard>
  </div>
</template>

<style scoped>
.wrap {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  width: 100%;
  padding: 40px;
  box-sizing: border-box;
}
.card {
  max-width: 640px;
  width: 100%;
}
.hint pre {
  background: rgba(128, 128, 128, 0.1);
  padding: 12px 16px;
  border-radius: 6px;
  font-size: 12px;
  overflow: auto;
}
</style>
