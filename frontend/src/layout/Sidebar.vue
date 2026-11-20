<!-- Author: Aut520 <aut@autkc.com> | https://github.com/Aut520/rmechrevo | GPL-3.0 -->
<script setup lang="ts">
import { RefreshCw, Palette } from 'lucide-vue-next'
import { tabs } from '../constants'
import { useAppearance } from '../composables/useAppearance'
import type { Tab, SystemInfo } from '../types'

defineProps<{
  active: Tab
  connected: boolean
  systemInfo: SystemInfo
}>()

defineEmits<{
  (e: 'select', tab: Tab): void
  (e: 'open-settings'): void
}>()

const { appearance, refreshRandomWallpaper } = useAppearance()
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-brand" data-tauri-drag-region>
      <span class="brand-badge">RMECHREVO</span>
      <span class="brand-sub" :title="systemInfo.product_name" data-tauri-drag-region>
        {{ systemInfo.product_name || 'Linux 硬件控制中心' }}
      </span>
    </div>

    <nav class="nav-list" aria-label="主导航">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="nav-item"
        :class="{ active: active === tab.id }"
        @click="$emit('select', tab.id)"
      >
        <component :is="tab.icon" :size="16" class="nav-icon" />
        <span>{{ tab.name }}</span>
      </button>
    </nav>

    <div class="sidebar-foot">
      <div class="daemon-state" :class="{ online: connected }">
        <span class="state-dot" />
        <span>{{ connected ? '守护服务在线' : '连接守护中' }}</span>
      </div>

      <div class="foot-tools">
        <button
          v-if="appearance.bgMode === 'api'"
          class="tool-btn"
          title="换一张随机壁纸"
          aria-label="换一张随机壁纸"
          @click="refreshRandomWallpaper"
        >
          <RefreshCw :size="15" />
        </button>
        <button
          class="tool-btn"
          title="外观与壁纸设置"
          aria-label="外观与壁纸设置"
          @click="$emit('open-settings')"
        >
          <Palette :size="15" />
        </button>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: max-content;
  min-width: 136px;
  max-width: 162px;
  flex-shrink: 0;
  height: 100%;
  padding: 16px 10px 14px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  background: var(--glass-sidebar-bg);
  backdrop-filter: blur(var(--glass-blur)) saturate(135%);
  -webkit-backdrop-filter: blur(var(--glass-blur)) saturate(135%);
  border-right: 1px solid var(--glass-divider);
}

.sidebar-brand {
  padding: 2px 4px 4px;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 6px;
}

.brand-badge {
  background: var(--accent-gradient);
  color: #ffffff;
  font-size: 10.5px;
  font-weight: 800;
  letter-spacing: 0.1em;
  padding: 3px 9px;
  border-radius: 6px;
  box-shadow: 0 2px 10px var(--accent-glow);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.22);
}

.brand-sub {
  font-size: 11.5px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 122px;
}

.nav-list {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.nav-item {
  height: 38px;
  padding: 0 11px;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  gap: 9px;
  font-size: 13px;
  white-space: nowrap;
  color: var(--text-secondary);
  border: 1px solid transparent;
}

.nav-item:hover {
  background: var(--pill-hover);
  color: var(--text-primary);
}

.nav-item.active {
  background: var(--accent-soft);
  border-color: var(--accent-border);
  color: var(--text-primary);
  box-shadow: 0 4px 12px var(--accent-glow);
}

.nav-item.active .nav-icon {
  color: var(--accent);
}

.sidebar-foot {
  padding-top: 12px;
  border-top: 1px solid var(--glass-divider);
  display: flex;
  flex-direction: column;
  gap: 9px;
}

.daemon-state {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 0 4px;
  font-size: 11.5px;
  white-space: nowrap;
  color: var(--text-muted);
}

.state-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
  background: var(--color-warning);
}

.daemon-state.online {
  color: var(--text-secondary);
}

.daemon-state.online .state-dot {
  background: var(--color-success);
  box-shadow: 0 0 8px var(--color-success);
}

.foot-tools {
  display: flex;
  gap: 6px;
}

.tool-btn {
  flex: 1;
  height: 32px;
  padding: 0;
  border-radius: var(--radius-sm);
  background: var(--glass-sub-bg);
  border: 1px solid var(--glass-border);
  color: var(--text-secondary);
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.tool-btn:hover {
  background: var(--pill-hover);
  border-color: var(--glass-border-hover);
  color: var(--text-primary);
}
</style>
