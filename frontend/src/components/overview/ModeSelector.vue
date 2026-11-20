<!-- Author: Aut520 <aut@autkc.com> | https://github.com/Aut520/rmechrevo | GPL-3.0 -->
<script setup lang="ts">
import { VolumeX, Zap } from 'lucide-vue-next'
import { modes } from '../../constants'

defineProps<{
  currentMode: number
  silentTurbo: boolean
  busy: boolean
}>()

defineEmits<{
  (e: 'select-mode', modeId: number): void
  (e: 'toggle-silent-turbo'): void
}>()
</script>

<template>
  <section class="glass-card mode-hero">
    <div class="card-head">
      <strong>性能模式</strong>
      <button
        v-if="currentMode === 0"
        class="turbo-pill"
        :class="{ silent: silentTurbo }"
        :disabled="busy"
        @click="$emit('toggle-silent-turbo')"
      >
        <VolumeX v-if="silentTurbo" :size="12" />
        <Zap v-else :size="12" />
        <span>{{ silentTurbo ? '静音 80%' : '满血 100%' }}</span>
      </button>
    </div>

    <div class="modes-grid">
      <button
        v-for="mode in modes"
        :key="mode.id"
        class="mode-tile"
        :class="[mode.color, { selected: currentMode === mode.id }]"
        :disabled="busy"
        @click="$emit('select-mode', mode.id)"
      >
        <div class="mode-icon" :class="mode.color">
          <component :is="mode.icon" :size="18" />
        </div>
        <div class="tile-text">
          <strong>{{ mode.name }}</strong>
          <small>{{ mode.english }}</small>
        </div>
        <span v-if="currentMode === mode.id" class="active-dot" :class="mode.color" />
      </button>
    </div>
  </section>
</template>

<style scoped>
.mode-hero {
  display: flex;
  flex-direction: column;
}

.turbo-pill {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 3px 10px;
  border-radius: var(--radius-pill);
  font-size: 11.5px;
  background: rgba(251, 146, 60, 0.22);
  border: 1px solid rgba(251, 146, 60, 0.5);
  color: #fed7aa;
}

.turbo-pill.silent {
  background: rgba(52, 211, 153, 0.22);
  border-color: rgba(52, 211, 153, 0.5);
  color: #a7f3d0;
}

.modes-grid {
  flex: 1;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.mode-tile {
  position: relative;
  text-align: left;
  padding: 15px 14px;
  border-radius: var(--radius-md);
  background: var(--glass-sub-bg);
  border: 1px solid var(--glass-border);
  display: flex;
  align-items: center;
  gap: 12px;
}

.mode-tile:hover:not(:disabled) {
  background: var(--pill-hover);
  border-color: var(--glass-border-hover);
  transform: translateY(-1px);
}

.mode-tile.red.selected {
  background: rgba(244, 63, 94, 0.22);
  border-color: rgba(244, 63, 94, 0.58);
  box-shadow: 0 4px 16px rgba(244, 63, 94, 0.2);
}

.mode-tile.cyan.selected {
  background: rgba(56, 189, 248, 0.22);
  border-color: rgba(56, 189, 248, 0.58);
  box-shadow: 0 4px 16px rgba(56, 189, 248, 0.2);
}

.mode-tile.green.selected {
  background: rgba(52, 211, 153, 0.22);
  border-color: rgba(52, 211, 153, 0.58);
  box-shadow: 0 4px 16px rgba(52, 211, 153, 0.2);
}

.mode-tile.white.selected {
  background: rgba(255, 255, 255, 0.2);
  border-color: rgba(255, 255, 255, 0.62);
  box-shadow: 0 4px 18px rgba(255, 255, 255, 0.16);
}

.mode-icon {
  width: 38px;
  height: 38px;
  border-radius: 10px;
  display: grid;
  place-items: center;
  flex-shrink: 0;
}

.mode-icon.red {
  background: rgba(244, 63, 94, 0.2);
  color: #fb7185;
}

.mode-icon.cyan {
  background: rgba(56, 189, 248, 0.2);
  color: #38bdf8;
}

.mode-icon.green {
  background: rgba(52, 211, 153, 0.2);
  color: #34d399;
}

.mode-icon.white {
  background: rgba(255, 255, 255, 0.18);
  color: #ffffff;
}

.tile-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.tile-text strong {
  font-size: 14px;
  font-weight: 700;
}

.tile-text small {
  font-size: 10px;
  letter-spacing: 0.08em;
  color: var(--text-muted);
}

.active-dot {
  position: absolute;
  top: 11px;
  right: 11px;
  width: 7px;
  height: 7px;
  border-radius: 50%;
}

.active-dot.red {
  background: #fb7185;
  box-shadow: 0 0 8px #fb7185;
}

.active-dot.cyan {
  background: #38bdf8;
  box-shadow: 0 0 8px #38bdf8;
}

.active-dot.green {
  background: #34d399;
  box-shadow: 0 0 8px #34d399;
}

.active-dot.white {
  background: #ffffff;
  box-shadow: 0 0 8px rgba(255, 255, 255, 0.9);
}
</style>
