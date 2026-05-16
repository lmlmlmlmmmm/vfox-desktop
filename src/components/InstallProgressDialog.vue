<script setup lang="ts">
/// 通用流式进度对话框
///
/// - 通过 props 传入 title 以及"要执行的操作"，组件内部生成 jobId 并订阅事件
/// - 支持三种触发模式：SDK 安装 / 插件 add / 插件 remove / 插件 update / 全量更新
/// - 完成时 emit("done", success)，由父组件决定后续 UX

import { computed, onBeforeUnmount, onMounted, ref, watch, nextTick } from 'vue'
import { NModal, NButton, NSpace, NTag } from 'naive-ui'
import type { UnlistenFn } from '@tauri-apps/api/event'
import * as api from '@/api'
import { genJobId, subscribeJob } from '@/api'

type OpKind = 'install-sdk' | 'add-plugin' | 'remove-plugin' | 'update-plugin' | 'update-all'

const props = withDefaults(
  defineProps<{
    title: string
    /// SDK 安装时使用
    sdk?: string
    version?: string
    /// 插件操作时使用
    pluginName?: string
    /// 默认为 install-sdk
    op?: OpKind
  }>(),
  { op: 'install-sdk' },
)

const emit = defineEmits<{
  done: [success: boolean]
}>()

const show = ref(true)
const lines = ref<{ stream: string; text: string }[]>([])
const done = ref(false)
const success = ref(false)
const jobId = genJobId()

let unlisten: UnlistenFn | null = null
const logBox = ref<HTMLElement | null>(null)

async function start() {
  unlisten = await subscribeJob(
    jobId,
    l => {
      lines.value.push({ stream: l.stream, text: l.line })
      // 自动滚到底
      nextTick(() => {
        if (logBox.value) logBox.value.scrollTop = logBox.value.scrollHeight
      })
    },
    d => {
      done.value = true
      success.value = d.success
      emit('done', d.success)
    },
  )

  try {
    switch (props.op) {
      case 'install-sdk':
        await api.installSdk(jobId, props.sdk!, props.version!)
        break
      case 'add-plugin':
        await api.addPlugin(jobId, props.pluginName!)
        break
      case 'remove-plugin':
        await api.removePlugin(jobId, props.pluginName!)
        break
      case 'update-plugin':
        await api.updatePlugin(jobId, props.pluginName!)
        break
      case 'update-all':
        await api.updateAllPlugins(jobId)
        break
    }
  } catch (e) {
    // 进程级错误（比如根本启动失败），追加到日志
    lines.value.push({ stream: 'stderr', text: `[error] ${String(e)}` })
    if (!done.value) {
      done.value = true
      success.value = false
      emit('done', false)
    }
  }
}

onMounted(start)
onBeforeUnmount(() => {
  unlisten?.()
})

// 防止 done 后用户关闭对话框时漏触发清理
watch(show, v => {
  if (!v) unlisten?.()
})

const statusLabel = computed(() => {
  if (!done.value) return '执行中'
  return success.value ? '成功' : '失败'
})
const statusType = computed<'info' | 'success' | 'error'>(() => {
  if (!done.value) return 'info'
  return success.value ? 'success' : 'error'
})
</script>

<template>
  <NModal
    v-model:show="show"
    preset="card"
    :title="title"
    style="width:720px"
    :mask-closable="false"
    :close-on-esc="done"
    :closable="done"
    @after-leave="$emit('done', success)"
  >
    <NSpace vertical :size="12">
      <div>
        <NTag :type="statusType">{{ statusLabel }}</NTag>
      </div>

      <div ref="logBox" class="log-box">
        <div
          v-for="(l, i) in lines"
          :key="i"
          :class="['log-line', l.stream === 'stderr' ? 'err' : '']"
        >
          {{ l.text }}
        </div>
        <div v-if="lines.length === 0" class="log-empty">等待输出…</div>
      </div>

      <NSpace justify="end">
        <NButton :disabled="!done" @click="show = false">关闭</NButton>
      </NSpace>
    </NSpace>
  </NModal>
</template>

<style scoped>
.log-box {
  height: 360px;
  overflow: auto;
  background: rgba(0, 0, 0, 0.78);
  color: #d4d4d4;
  border-radius: 6px;
  padding: 12px 14px;
  font-family: "JetBrains Mono", Consolas, "Cascadia Code", monospace;
  font-size: 12px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-all;
}
.log-line.err {
  color: #ff8b8b;
}
.log-empty {
  opacity: 0.5;
  font-style: italic;
}
</style>
