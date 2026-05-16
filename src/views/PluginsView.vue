<script setup lang="ts">
import { computed, h, onMounted, ref } from 'vue'
import {
  NButton,
  NSpace,
  NDataTable,
  NTag,
  NSpin,
  NInput,
  NPopconfirm,
  NDrawer,
  NDrawerContent,
  NCard,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import * as api from '@/api'
import type { AvailablePlugin, PluginInfo } from '@/types'
import { useVfoxStore } from '@/stores/vfox'
import InstallProgressDialog from '@/components/InstallProgressDialog.vue'

const msg = useMessage()
const vfoxStore = useVfoxStore()
const list = ref<AvailablePlugin[]>([])
const loading = ref(false)
const filter = ref('')

const infoOpen = ref(false)
const infoLoading = ref(false)
const info = ref<PluginInfo | null>(null)

type Job =
  | { op: 'add-plugin'; pluginName: string; title: string }
  | { op: 'remove-plugin'; pluginName: string; title: string }
  | { op: 'update-plugin'; pluginName: string; title: string }
  | { op: 'update-all'; title: string }
const job = ref<Job | null>(null)

const filtered = computed(() => {
  const f = filter.value.trim().toLowerCase()
  if (!f) return list.value
  return list.value.filter(p => p.name.toLowerCase().includes(f))
})

async function refresh() {
  loading.value = true
  try {
    list.value = await api.listAvailablePlugins()
  } catch (e) {
    msg.error(`加载插件市场失败：${e}`)
  } finally {
    loading.value = false
  }
}

async function viewInfo(name: string) {
  infoOpen.value = true
  info.value = null
  infoLoading.value = true
  try {
    info.value = await api.pluginInfo(name)
  } catch (e) {
    msg.error(`获取详情失败：${e}`)
    infoOpen.value = false
  } finally {
    infoLoading.value = false
  }
}

function add(name: string) {
  job.value = { op: 'add-plugin', pluginName: name, title: `安装插件 ${name}` }
}
function remove(name: string) {
  job.value = { op: 'remove-plugin', pluginName: name, title: `移除插件 ${name}` }
}
function update(name: string) {
  job.value = { op: 'update-plugin', pluginName: name, title: `更新插件 ${name}` }
}
function updateAll() {
  job.value = { op: 'update-all', title: '更新全部插件' }
}

async function onDone(success: boolean) {
  if (success) msg.success('操作完成')
  else msg.error('操作失败，请查看日志')
  job.value = null
  // 并发刷新两个 store：本页插件列表 + 跨页 SDK 列表
  await Promise.all([refresh(), vfoxStore.refresh()])
}

const columns = computed<DataTableColumns<AvailablePlugin>>(() => [
  {
    title: '插件',
    key: 'name',
    width: 180,
    render(row) {
      return h(
        'a',
        {
          style: 'color:inherit; text-decoration:none; cursor:pointer; font-weight:500',
          onClick: () => viewInfo(row.name),
        },
        row.name,
      )
    },
  },
  {
    title: '状态',
    key: 'installed',
    width: 90,
    render(row) {
      return h(
        NTag,
        { size: 'small', type: row.installed ? 'success' : 'default' },
        { default: () => (row.installed ? '已安装' : '未安装') },
      )
    },
  },
  {
    title: '仓库',
    key: 'homepage',
    minWidth: 280,
    render(row) {
      return h(
        'span',
        {
          style:
            'font-size:12px; opacity:0.7; white-space:nowrap; overflow:hidden; text-overflow:ellipsis; display:block;',
          title: row.homepage,
        },
        row.homepage,
      )
    },
  },
  {
    title: '操作',
    key: 'actions',
    width: 140,
    render(row) {
      const buttons = [] as any[]
      if (row.installed) {
        buttons.push(
          h(
            NButton,
            { size: 'small', secondary: true, onClick: () => update(row.name) },
            { default: () => '更新' },
          ),
        )
        buttons.push(
          h(
            NPopconfirm,
            {
              onPositiveClick: () => remove(row.name),
              positiveText: '移除',
              negativeText: '取消',
            },
            {
              trigger: () =>
                h(
                  NButton,
                  { size: 'small', type: 'error', tertiary: true },
                  { default: () => '移除' },
                ),
              default: () =>
                `移除插件 ${row.name}？相关 SDK 将不再可用，但已安装的版本目录保留。`,
            },
          ),
        )
      } else {
        buttons.push(
          h(
            NButton,
            {
              size: 'small',
              type: 'primary',
              onClick: () => add(row.name),
            },
            { default: () => '安装' },
          ),
        )
      }
      return h('div', { style: 'display:flex; gap:8px;' }, buttons)
    },
  },
])

onMounted(refresh)
</script>

<template>
  <div class="page">
    <header class="page-header">
      <div class="page-title-row">
        <h2 class="page-title">插件市场</h2>
        <span class="page-title-hint">仅展示 vfox 官方插件源的内置插件</span>
      </div>
      <NSpace>
        <NInput v-model:value="filter" placeholder="过滤插件名" clearable style="width:220px" />
        <NButton :loading="loading" @click="refresh">刷新</NButton>
        <NButton type="primary" secondary @click="updateAll">更新全部</NButton>
      </NSpace>
    </header>

    <div v-if="loading && list.length === 0" class="center">
      <NSpin />
    </div>

    <NDataTable
      v-else
      :columns="columns"
      :data="filtered"
      :row-key="(r: AvailablePlugin) => r.name"
      size="small"
      flex-height
      style="height: calc(100vh - 150px)"
    />

    <!-- 详情抽屉 -->
    <NDrawer v-model:show="infoOpen" :width="520" placement="right">
      <NDrawerContent :title="info?.name ?? '插件详情'" :native-scrollbar="false">
        <div v-if="infoLoading" class="center" style="padding:40px"><NSpin /></div>
        <NCard v-else-if="info" :bordered="false">
          <p><strong>版本：</strong>{{ info.version }}</p>
          <p>
            <strong>主页：</strong>
            <a :href="info.homepage" target="_blank">{{ info.homepage }}</a>
          </p>
          <p v-if="info.description"><strong>简介：</strong>{{ info.description }}</p>
          <div v-if="info.notes">
            <strong>备注：</strong>
            <pre class="notes">{{ info.notes }}</pre>
          </div>
        </NCard>
      </NDrawerContent>
    </NDrawer>

    <!-- 流式对话框 -->
    <InstallProgressDialog
      v-if="job"
      :title="job.title"
      :op="job.op"
      :plugin-name="(job as any).pluginName"
      @done="onDone"
    />
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
.page-title-row {
  display: flex;
  align-items: baseline;
  gap: 10px;
  min-width: 0;
}
.page-title-hint {
  font-size: 12px;
  opacity: 0.55;
  white-space: nowrap;
}
.center {
  display: flex;
  justify-content: center;
  align-items: center;
}
.notes {
  white-space: pre-wrap;
  font-family: "JetBrains Mono", Consolas, monospace;
  font-size: 12px;
  background: rgba(128, 128, 128, 0.08);
  padding: 10px 12px;
  border-radius: 4px;
  margin-top: 6px;
}
</style>
