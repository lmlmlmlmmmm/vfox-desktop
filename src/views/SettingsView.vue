<script setup lang="ts">
import { NCard, NForm, NFormItem, NRadioGroup, NRadioButton, NSpace, NText } from 'naive-ui'
import { useAppStore } from '@/stores/app'

const app = useAppStore()
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
