<script setup lang="ts">
import { computed, h, onMounted, ref, watch } from 'vue'
import {
  NButton,
  NSpace,
  NDataTable,
  NTag,
  NEmpty,
  NList,
  NListItem,
  NText,
  NPopconfirm,
  NSpin,
  NInput,
  NSelect,
  NAlert,
  NModal,
  useMessage,
  type DataTableColumns,
  type SelectOption,
} from 'naive-ui'
import { useVfoxStore } from '@/stores/vfox'
import * as api from '@/api'
import type { SearchVersion } from '@/types'
import InstallProgressDialog from '@/components/InstallProgressDialog.vue'

interface VersionRow {
  v: string
}

const store = useVfoxStore()
const msg = useMessage()

const selectedSdk = ref<string | null>(null)
const installModalOpen = ref(false)
const pendingInstall = ref<null | { sdk: string; version: string }>(null)

// 远端版本选择相关
const remoteVersions = ref<SearchVersion[]>([])
const remoteLoading = ref(false)
const remoteError = ref<string | null>(null)
const selectedRemote = ref<string | null>(null)
const manualInput = ref('')
const filterInput = ref('')

/// 当前 SDK 支持的筛选词候选（从 vfox info notes 动态抽取）
/// 空数组表示该插件无分类概念，UI 不显示筛选区
const distributions = ref<string[]>([])

const sdkList = computed(() => store.sdks)
const versions = computed<VersionRow[]>(() => {
  const sel = selectedSdk.value
  if (!sel) return []
  return (store.sdks.find(s => s.sdk === sel)?.versions ?? []).map(v => ({ v }))
})
const currentForSelected = computed(() =>
  selectedSdk.value ? store.currentMap[selectedSdk.value] : undefined,
)

onMounted(async () => {
  await store.refresh()
  if (!selectedSdk.value && sdkList.value.length > 0) {
    selectedSdk.value = sdkList.value[0].sdk
  }
})

watch(sdkList, list => {
  if (selectedSdk.value && !list.some(s => s.sdk === selectedSdk.value)) {
    selectedSdk.value = list[0]?.sdk ?? null
  } else if (!selectedSdk.value && list.length > 0) {
    selectedSdk.value = list[0].sdk
  }
})

async function onUse(version: string) {
  if (!selectedSdk.value) return
  try {
    await store.useVersion(selectedSdk.value, version)
    msg.success(`已切换 ${selectedSdk.value} → ${version}`)
  } catch (e) {
    msg.error(String(e))
  }
}

async function onUninstall(version: string) {
  if (!selectedSdk.value) return
  try {
    await store.uninstall(selectedSdk.value, version)
    msg.success(`已卸载 ${selectedSdk.value}@${version}`)
  } catch (e) {
    msg.error(String(e))
  }
}

async function reloadVersions() {
  if (!selectedSdk.value) return
  remoteLoading.value = true
  remoteError.value = null
  selectedRemote.value = null
  remoteVersions.value = []
  try {
    remoteVersions.value = await api.searchVersions(
      selectedSdk.value,
      filterInput.value.trim() || undefined,
    )
  } catch (e) {
    remoteError.value = String(e)
  } finally {
    remoteLoading.value = false
  }
}

async function openInstallModal() {
  if (!selectedSdk.value) return
  installModalOpen.value = true
  selectedRemote.value = null
  manualInput.value = ''
  filterInput.value = ''
  distributions.value = []

  // 并发拉版本列表 + 插件元信息（只为拿 distributions）
  const sdk = selectedSdk.value
  remoteLoading.value = true
  remoteError.value = null

  const versionsPromise = api
    .searchVersions(sdk)
    .then(vs => {
      remoteVersions.value = vs
    })
    .catch(e => {
      remoteError.value = String(e)
    })

  const infoPromise = api
    .pluginInfo(sdk)
    .then(info => {
      distributions.value = info.distributions ?? []
    })
    .catch(() => {
      // info 拿不到不影响主流程，distributions 留空即可
    })

  await Promise.all([versionsPromise, infoPromise])
  remoteLoading.value = false
}

/// 点击某个 distribution 快捷按钮：填入并重新拉版本
async function pickDistribution(d: string) {
  filterInput.value = filterInput.value === d ? '' : d
  await reloadVersions()
}

function confirmInstall() {
  if (!selectedSdk.value) return
  const v = (selectedRemote.value || manualInput.value).trim()
  if (!v) {
    msg.warning('请选择或输入要安装的版本号')
    return
  }
  installModalOpen.value = false
  pendingInstall.value = { sdk: selectedSdk.value, version: v }
}

// NSelect 选项：已安装版本灰显禁选，LTS 加标签
const remoteOptions = computed<SelectOption[]>(() =>
  remoteVersions.value.map(v => ({
    label: v.version + (v.is_lts ? '  (LTS)' : '') + (v.is_installed ? '  · 已安装' : ''),
    value: v.version,
    disabled: v.is_installed,
  })),
)

async function onDialogDone(success: boolean) {
  if (success) msg.success('安装完成')
  else msg.error('安装失败，请查看日志')
  pendingInstall.value = null
  await store.refresh()
}

// 版本表格列定义
const versionColumns = computed<DataTableColumns<VersionRow>>(() => [
  {
    title: '版本',
    key: 'v',
    render(row) {
      const isCurrent = row.v === currentForSelected.value
      return h('span', { style: 'display:inline-flex; align-items:center; gap:8px;' }, [
        row.v,
        isCurrent
          ? h(NTag, { type: 'success', size: 'small', round: true }, { default: () => '当前' })
          : null,
      ])
    },
  },
  {
    title: '操作',
    key: 'actions',
    width: 220,
    render(row) {
      const isCurrent = row.v === currentForSelected.value
      const switchBtn = !isCurrent
        ? h(
            NButton,
            {
              size: 'small',
              type: 'primary',
              secondary: true,
              onClick: () => onUse(row.v),
            },
            { default: () => '切换为当前' },
          )
        : null

      const uninstallBtn = h(
        NPopconfirm,
        {
          onPositiveClick: () => onUninstall(row.v),
          positiveText: '卸载',
          negativeText: '取消',
        },
        {
          trigger: () =>
            h(
              NButton,
              { size: 'small', type: 'error', tertiary: true },
              { default: () => '卸载' },
            ),
          default: () => `确定卸载 ${selectedSdk.value}@${row.v}？`,
        },
      )

      return h('div', { style: 'display:flex; gap:8px;' }, [switchBtn, uninstallBtn])
    },
  },
])
</script>

<template>
  <div class="page">
    <header class="page-header">
      <h2 class="page-title">SDK 管理</h2>
      <NSpace>
        <NButton @click="store.refresh()" :loading="store.loading">刷新</NButton>
        <NButton type="primary" :disabled="!selectedSdk" @click="openInstallModal">
          安装新版本
        </NButton>
      </NSpace>
    </header>

    <div class="layout" v-if="store.loading && sdkList.length === 0">
      <NSpin />
    </div>

    <div class="layout" v-else-if="sdkList.length === 0">
      <NEmpty description="还没有安装任何 SDK 插件。请先到「插件市场」添加一个，例如 nodejs / java / golang。" />
    </div>

    <div class="layout" v-else>
      <div class="sdk-col">
        <NList hoverable clickable>
          <NListItem
            v-for="sdk in sdkList"
            :key="sdk.sdk"
            :class="{ active: sdk.sdk === selectedSdk }"
            @click="selectedSdk = sdk.sdk"
          >
            <NSpace align="center" :wrap="false" justify="space-between" style="width:100%">
              <div>
                <NText>{{ sdk.sdk }}</NText>
                <NText depth="3" style="margin-left:8px; font-size:12px">
                  {{ sdk.versions.length }} 个版本
                </NText>
              </div>
              <NTag v-if="store.currentMap[sdk.sdk]" type="success" size="small" round>
                {{ store.currentMap[sdk.sdk] }}
              </NTag>
            </NSpace>
          </NListItem>
        </NList>
      </div>

      <div class="version-col">
        <h3 class="sub-title" v-if="selectedSdk">{{ selectedSdk }} 的版本</h3>
        <NEmpty v-if="versions.length === 0" description="还没有任何版本，点右上「安装新版本」开始。" />
        <NDataTable
          v-else
          :columns="versionColumns"
          :data="versions"
          :row-key="(r: VersionRow) => r.v"
          size="small"
          flex-height
          style="height: calc(100vh - 220px)"
        />
      </div>
    </div>

    <NModal
      v-model:show="installModalOpen"
      preset="card"
      style="width:560px"
      :title="`安装 ${selectedSdk} 的新版本`"
    >
      <NSpace vertical :size="16">
        <!-- 分类/供应商筛选：只在插件提供候选时显示 -->
        <div v-if="distributions.length > 0">
          <p class="filter-label">
            分类筛选
            <small class="filter-hint">从插件元信息自动识别 · 留空使用默认</small>
          </p>
          <NSpace :size="8" :wrap="true">
            <NButton
              v-for="d in distributions"
              :key="d"
              size="small"
              :type="filterInput.trim() === d ? 'primary' : 'default'"
              :secondary="filterInput.trim() !== d"
              :disabled="remoteLoading"
              @click="pickDistribution(d)"
            >
              {{ d }}
            </NButton>
          </NSpace>
        </div>

        <div v-if="remoteLoading" class="modal-loading">
          <NSpin />
          <span style="margin-left:12px; opacity:0.7">正在从远端获取可用版本…</span>
        </div>

        <template v-else-if="remoteError">
          <NAlert type="warning" :show-icon="true">
            无法从 vfox 拉取版本列表：{{ remoteError }}
          </NAlert>
          <p style="margin:0; opacity:0.7">你可以手动输入版本号继续：</p>
          <NInput
            v-model:value="manualInput"
            placeholder="如 21.0.2+13"
            @keydown.enter="confirmInstall"
          />
        </template>

        <template v-else>
          <p style="margin:0; opacity:0.7">
            为 <strong>{{ selectedSdk }}</strong>
            <span v-if="filterInput.trim()"> · 筛选词 "{{ filterInput.trim() }}"</span>
            找到 {{ remoteVersions.length }} 个远端版本：
          </p>
          <NSelect
            v-model:value="selectedRemote"
            :options="remoteOptions"
            filterable
            clearable
            placeholder="选择要安装的版本"
            size="medium"
          />
          <details class="manual-fallback">
            <summary>列表里没有？手动输入版本号</summary>
            <NInput
              v-model:value="manualInput"
              placeholder="如 21.0.2+13"
              style="margin-top:8px"
              @keydown.enter="confirmInstall"
            />
          </details>
        </template>

        <NSpace justify="end">
          <NButton @click="installModalOpen = false">取消</NButton>
          <NButton
            type="primary"
            :disabled="remoteLoading || (!selectedRemote && !manualInput.trim())"
            @click="confirmInstall"
          >
            开始安装
          </NButton>
        </NSpace>
      </NSpace>
    </NModal>

    <InstallProgressDialog
      v-if="pendingInstall"
      :title="`安装 ${pendingInstall.sdk}@${pendingInstall.version}`"
      :sdk="pendingInstall.sdk"
      :version="pendingInstall.version"
      @done="onDialogDone"
    />
  </div>
</template>

<style scoped>
.page {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
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
.layout {
  display: flex;
  gap: 16px;
  min-height: 0;
  flex: 1;
}
.sdk-col {
  width: 260px;
  flex-shrink: 0;
  border-radius: 6px;
  overflow: auto;
  border: 1px solid var(--app-border);
  background: var(--app-surface);
}
.version-col {
  flex: 1;
  min-width: 0;
}
.sub-title {
  margin: 0 0 12px;
  font-size: 16px;
  font-weight: 600;
}
:deep(.n-list-item.active) {
  background: var(--app-hover);
}
.modal-loading {
  display: flex;
  align-items: center;
  padding: 12px 0;
}
.manual-fallback summary {
  cursor: pointer;
  opacity: 0.6;
  font-size: 13px;
  user-select: none;
}
.manual-fallback summary:hover {
  opacity: 0.9;
}
.filter-label {
  margin: 0 0 6px;
  font-size: 13px;
  font-weight: 500;
}
.filter-hint {
  margin-left: 6px;
  font-weight: 400;
  opacity: 0.55;
  font-size: 12px;
}
</style>
