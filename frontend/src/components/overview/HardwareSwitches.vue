<!-- Author: Aut520 <aut@autkc.com> | https://github.com/Aut520/rmechrevo | GPL-3.0 -->
<script setup lang="ts">
import { computed } from 'vue'
import { Fan, LockKeyhole, MousePointer2 } from 'lucide-vue-next'

const props = defineProps<{
  fanBoost: boolean
  winLock: boolean
  touchpadDisabled: boolean
  busy: boolean
}>()

const emit = defineEmits<{
  (e: 'toggle-fan-boost'): void
  (e: 'toggle-win-lock'): void
  (e: 'toggle-touchpad'): void
}>()

const switches = computed(() => [
  { id: 'fanBoost', icon: Fan, label: '暴风强冷', state: props.fanBoost, disabled: props.busy },
  { id: 'winLock', icon: LockKeyhole, label: 'Win 键锁定', state: props.winLock, disabled: props.busy },
  { id: 'touchpad', icon: MousePointer2, label: '禁用触控板', state: props.touchpadDisabled, disabled: props.busy }
])

function handleToggle(id: string) {
  if (id === 'fanBoost') emit('toggle-fan-boost')
  else if (id === 'winLock') emit('toggle-win-lock')
  else if (id === 'touchpad') emit('toggle-touchpad')
}
</script>

<template>
  <section class="glass-card switches-panel">
    <div class="card-head">
      <strong>快捷开关</strong>
    </div>

    <div class="switch-list">
      <div
        v-for="item in switches"
        :key="item.id"
        class="switch-row"
        :class="{ active: item.state }"
      >
        <div class="switch-icon" :class="{ active: item.state }">
          <component
            :is="item.icon"
            :size="15"
            :class="{ spinning: item.id === 'fanBoost' && item.state }"
          />
        </div>
        <strong class="switch-label">{{ item.label }}</strong>
        <button
          role="switch"
          :aria-label="`${item.label}开关`"
          class="ios-toggle"
          :class="{ on: item.state }"
          :disabled="item.disabled"
          @click="handleToggle(item.id)"
        >
          <i />
        </button>
      </div>
    </div>
  </section>
</template>

<style scoped>
.switches-panel {
  display: flex;
  flex-direction: column;
}

.switch-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.switch-row {
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 8px 12px;
  border-radius: var(--radius-md);
  background: var(--glass-sub-bg);
  border: 1px solid var(--glass-divider);
  transition: border-color 0.2s ease;
}

.switch-row.active {
  border-color: var(--accent-border);
}

.switch-icon {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  background: var(--pill-bg);
  color: var(--text-secondary);
  display: grid;
  place-items: center;
  flex-shrink: 0;
}

.switch-icon.active {
  background: var(--accent-soft);
  color: var(--accent);
}

.switch-label {
  flex: 1;
  font-size: 13px;
  font-weight: 700;
}
</style>
