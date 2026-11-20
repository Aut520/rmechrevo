<!-- Author: Aut520 <aut@autkc.com> | https://github.com/Aut520/rmechrevo | GPL-3.0 -->
<script setup lang="ts">
import { Minus, Square, X } from 'lucide-vue-next'

defineProps<{
  title: string
}>()

defineEmits<{
  (e: 'window-action', action: 'minimize_window' | 'toggle_maximize' | 'hide_window'): void
}>()
</script>

<template>
  <header class="titlebar" data-tauri-drag-region @dblclick="$emit('window-action', 'toggle_maximize')">
    <h1 class="page-title" data-tauri-drag-region>{{ title }}</h1>

    <div class="win-controls">
      <button class="win-btn" title="最小化" @click.stop="$emit('window-action', 'minimize_window')">
        <Minus :size="14" />
      </button>
      <button class="win-btn" title="最大化" @click.stop="$emit('window-action', 'toggle_maximize')">
        <Square :size="11" />
      </button>
      <button class="win-btn close" title="隐藏到托盘" @click.stop="$emit('window-action', 'hide_window')">
        <X :size="15" />
      </button>
    </div>
  </header>
</template>

<style scoped>
.titlebar {
  height: 46px;
  flex-shrink: 0;
  padding: 0 14px 0 22px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  z-index: 20;
}

.page-title {
  font-size: 15px;
  font-weight: 700;
  letter-spacing: 0.01em;
  color: var(--text-primary);
}

.win-controls {
  display: flex;
  align-items: center;
  gap: 4px;
}

.win-btn {
  width: 30px;
  height: 30px;
  display: grid;
  place-items: center;
  border-radius: 8px;
  color: var(--text-secondary);
}

.win-btn:hover {
  background: var(--pill-bg);
  color: var(--text-primary);
}

.win-btn.close:hover {
  background: rgba(248, 113, 113, 0.28);
  color: #fee2e2;
}
</style>
