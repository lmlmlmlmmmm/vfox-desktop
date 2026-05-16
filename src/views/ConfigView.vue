<script setup lang="ts">
import { onMounted, ref } from 'vue'
import {
  NCard,
  NForm,
  NFormItem,
  NInput,
  NSwitch,
  NButton,
  NSpace,
  NSelect,
  NAlert,
  NSpin,
  useMessage,
} from 'naive-ui'
import * as api from '@/api'
import type { VfoxConfig } from '@/types'

const msg = useMessage()
const loading = ref(false)
const saving = ref(false)
const cfg = ref<VfoxConfig | null>(null)

const strategyOptions = [
  { label: 'specified（按文件指定）', value: 'specified' },
  { label: 'latest（最新匹配）', value: 'latest' },
  { label: 'disable（禁用）', value: 'disable' },
]

async function load() {
  loading.value = true
  try {
    cfg.value = await api.getConfig()
  } catch (e) {
    msg.error(`读取配置失败：${e}`)
  } finally {
    loading.value = false
  }
}

async function save() {
  if (!cfg.value) return
  saving.value = true
  try {
    await api.saveConfig(cfg.value)
    msg.success('配置已保存（原文件已备份为 config.yaml.bak）')
  } catch (e) {
    msg.error(`保存失败：${e}`)
  } finally {
    saving.value = false
  }
}

onMounted(load)
</script>

<template>
  <div class="page">
    <header class="page-header">
      <h2 class="page-title">vfox 配置</h2>
      <NSpace>
        <NButton :loading="loading" @click="load">重新加载</NButton>
        <NButton type="primary" :loading="saving" :disabled="!cfg" @click="save">保存</NButton>
      </NSpace>
    </header>

    <NAlert type="info" style="margin-bottom:16px" :show-icon="true">
      此处直接读写 <code>~/.vfox/config.yaml</code>。保存前会自动备份为
      <code>config.yaml.bak</code>。注释不会被保留。
    </NAlert>

    <div v-if="loading" class="center"><NSpin /></div>

    <NSpace v-else-if="cfg" vertical :size="16">
      <NCard title="代理">
        <NForm label-placement="left" :label-width="120">
          <NFormItem label="启用代理">
            <NSwitch v-model:value="cfg.proxy.enable" />
          </NFormItem>
          <NFormItem label="代理 URL">
            <NInput
              v-model:value="cfg.proxy.url"
              placeholder="如 http://127.0.0.1:7890"
              :disabled="!cfg.proxy.enable"
            />
          </NFormItem>
        </NForm>
      </NCard>

      <NCard title="存储">
        <NForm label-placement="left" :label-width="120">
          <NFormItem label="SDK 安装路径">
            <NInput
              v-model:value="cfg.storage.sdkPath"
              placeholder="留空使用默认 ~/.vfox/sdks"
            />
          </NFormItem>
        </NForm>
      </NCard>

      <NCard title="插件注册表">
        <NForm label-placement="left" :label-width="120">
          <NFormItem label="Registry 地址">
            <NInput
              v-model:value="cfg.registry.address"
              placeholder="留空使用默认官方源"
            />
          </NFormItem>
        </NForm>
      </NCard>

      <NCard title="Legacy 版本文件">
        <NForm label-placement="left" :label-width="120">
          <NFormItem label="启用">
            <NSwitch v-model:value="cfg.legacyVersionFile.enable" />
          </NFormItem>
          <NFormItem label="策略">
            <NSelect
              v-model:value="cfg.legacyVersionFile.strategy"
              :options="strategyOptions"
              :disabled="!cfg.legacyVersionFile.enable"
            />
          </NFormItem>
        </NForm>
      </NCard>

      <NCard title="缓存">
        <NForm label-placement="left" :label-width="150">
          <NFormItem label="available 缓存时长">
            <NInput
              v-model:value="cfg.cache.availableHookDuration"
              placeholder="如 12h / 30m"
            />
          </NFormItem>
        </NForm>
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
.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}
.page-title {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
}
.center {
  display: flex;
  justify-content: center;
  padding: 40px;
}
</style>
