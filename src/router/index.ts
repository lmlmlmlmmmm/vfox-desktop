import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  { path: '/', redirect: '/sdk' },
  {
    path: '/sdk',
    name: 'sdk',
    component: () => import('@/views/SdkView.vue'),
    meta: { title: 'SDK 管理' },
  },
  {
    path: '/plugins',
    name: 'plugins',
    component: () => import('@/views/PluginsView.vue'),
    meta: { title: '插件市场' },
  },
  {
    path: '/config',
    name: 'config',
    component: () => import('@/views/ConfigView.vue'),
    meta: { title: 'vfox 配置' },
  },
  {
    path: '/settings',
    name: 'settings',
    component: () => import('@/views/SettingsView.vue'),
    meta: { title: '应用设置' },
  },
  {
    path: '/not-installed',
    name: 'not-installed',
    component: () => import('@/views/NotInstalledView.vue'),
    meta: { title: '未检测到 vfox' },
  },
]

export default createRouter({
  history: createWebHashHistory(),
  routes,
})
