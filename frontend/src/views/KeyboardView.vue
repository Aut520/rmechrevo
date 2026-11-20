<!-- Author: Aut520 <aut@autkc.com> | https://github.com/Aut520/rmechrevo | GPL-3.0 -->
<script setup lang="ts">
import { computed } from 'vue'
import { Check } from 'lucide-vue-next'
import { colors, breathingColors } from '../constants'
import type { RgbState, ColorItem } from '../types'

const props = defineProps<{
  rgb: RgbState
  fnLock: boolean
}>()

defineEmits<{
  (e: 'set-rgb-power', enabled: boolean): void
  (e: 'set-rgb-brightness', percent: number): void
  (e: 'select-rgb-effect', modeIdx: number): void
  (e: 'select-rgb-color', color: ColorItem): void
  (e: 'toggle-fn-lock'): void
}>()

const isLit = computed(() => props.rgb.enabled && props.rgb.brightness > 0)
const isRainbow = computed(() => props.rgb.effect_mode === 1)
const isBreathing = computed(() => props.rgb.effect_mode === 2)
const showColorPicker = computed(() => isLit.value && !isRainbow.value)
const activePalette = computed(() => (isBreathing.value ? breathingColors : colors))

const effectiveRgb = computed<[number, number, number]>(() => {
  if (!isBreathing.value) {
    return [props.rgb.r, props.rgb.g, props.rgb.b]
  }
  let best = breathingColors[3]
  let minDist = Number.POSITIVE_INFINITY
  for (const item of breathingColors) {
    const dr = props.rgb.r - item.value[0]
    const dg = props.rgb.g - item.value[1]
    const db = props.rgb.b - item.value[2]
    const dist = dr * dr + dg * dg + db * db
    if (dist < minDist) {
      minDist = dist
      best = item
    }
  }
  return best.value
})

function isColorSelected(color: ColorItem) {
  const [r, g, b] = effectiveRgb.value
  return r === color.value[0] && g === color.value[1] && b === color.value[2]
}
</script>

<template>
  <div class="bento-page">
    <section class="glass-card">
      <div class="card-head">
        <strong>键盘背光</strong>
        <button
          role="switch"
          aria-label="键盘背光总开关"
          class="ios-toggle"
          :class="{ on: isLit }"
          @click="$emit('set-rgb-power', !isLit)"
        >
          <i />
        </button>
      </div>

      <div
        class="keyboard-visual"
        :class="{ rainbow: isLit && isRainbow, breathing: isLit && isBreathing, off: !isLit }"
        :style="{
          '--key-color': `rgb(${effectiveRgb[0]}, ${effectiveRgb[1]}, ${effectiveRgb[2]})`,
          '--key-glow': `rgba(${effectiveRgb[0]}, ${effectiveRgb[1]}, ${effectiveRgb[2]}, ${isLit ? (rgb.brightness / 100) * 0.55 : 0})`,
          '--key-opacity': isLit ? Math.max(0.45, rgb.brightness / 100) : 0.24
        }"
      >
        <div class="key-row" v-for="row in [15, 14, 13, 12]" :key="row">
          <i v-for="key in row" :key="key" />
        </div>
        <div class="key-row bottom">
          <i /><i /><i /><i class="space" /><i /><i /><i />
        </div>
      </div>
    </section>

    <div v-if="isLit" class="kbd-controls-grid" :class="{ single: !showColorPicker }">
      <section class="glass-card controls-card">
        <div class="ctrl-block">
          <div class="ctrl-head">
            <strong>灯效</strong>
          </div>
          <div class="capsule-group">
            <button
              v-for="(name, modeIdx) in ['常亮', '彩虹', '呼吸']"
              :key="name"
              class="capsule-btn"
              :class="{ active: rgb.effect_mode === modeIdx }"
              @click="$emit('select-rgb-effect', modeIdx)"
            >
              {{ name }}
            </button>
          </div>
        </div>

        <div class="ctrl-block">
          <div class="ctrl-head">
            <strong>亮度</strong>
            <span class="card-head-sub tabular">{{ rgb.brightness }}%</span>
          </div>
          <div class="capsule-group">
            <button
              v-for="percent in [25, 50, 75, 100]"
              :key="percent"
              class="capsule-btn"
              :class="{ active: rgb.brightness === percent }"
              @click="$emit('set-rgb-brightness', percent)"
            >
              {{ percent }}%
            </button>
          </div>
        </div>
      </section>

      <section v-if="showColorPicker" class="glass-card">
        <div class="card-head">
          <strong>色彩</strong>
        </div>
        <div class="swatches">
          <button
            v-for="color in activePalette"
            :key="color.name"
            class="swatch-btn"
            :class="{ selected: isColorSelected(color) }"
            :title="color.name"
            @click="$emit('select-rgb-color', color)"
          >
            <span class="swatch-circle" :style="{ background: color.hex }">
              <Check
                v-if="isColorSelected(color)"
                :size="13"
                :style="{ color: color.hex === '#ffffff' || color.hex === '#eab308' ? '#0f172a' : '#ffffff' }"
              />
            </span>
            <small>{{ color.name }}</small>
          </button>
        </div>
      </section>
    </div>

    <section class="glass-card fn-card">
      <strong>Fn 键翻转</strong>
      <button
        role="switch"
        aria-label="Fn 锁定开关"
        class="ios-toggle"
        :class="{ on: fnLock }"
        @click="$emit('toggle-fn-lock')"
      >
        <i />
      </button>
    </section>
  </div>
</template>

<style scoped>
.keyboard-visual {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 14px;
  border-radius: var(--radius-md);
  background: rgba(0, 0, 0, 0.34);
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: inset 0 0 22px var(--key-glow);
  opacity: var(--key-opacity);
  transition: opacity 0.25s ease, box-shadow 0.25s ease;
}

.keyboard-visual.off {
  box-shadow: none;
}

.keyboard-visual.off .key-row i {
  border-color: rgba(255, 255, 255, 0.24);
  box-shadow: none;
}

.keyboard-visual.rainbow {
  background: linear-gradient(
    120deg,
    rgba(244, 63, 94, 0.28),
    rgba(251, 191, 36, 0.25),
    rgba(52, 211, 153, 0.26),
    rgba(56, 189, 248, 0.28),
    rgba(168, 85, 247, 0.28)
  );
  background-size: 220% 220%;
  box-shadow: inset 0 0 24px rgba(168, 85, 247, 0.3);
  animation: rainbow-flow 5s ease infinite;
}

.keyboard-visual.rainbow .key-row i {
  border-color: rgba(255, 255, 255, 0.45);
  box-shadow: 0 0 8px rgba(255, 255, 255, 0.18);
}

.keyboard-visual.breathing {
  animation: breathe-glow 2.6s ease-in-out infinite;
}

@keyframes rainbow-flow {
  0% {
    background-position: 0% 50%;
  }
  50% {
    background-position: 100% 50%;
  }
  100% {
    background-position: 0% 50%;
  }
}

@keyframes breathe-glow {
  0%,
  100% {
    opacity: var(--key-opacity);
  }
  50% {
    opacity: 0.32;
  }
}

.key-row {
  display: flex;
  gap: 5px;
}

.key-row i {
  flex: 1;
  height: 22px;
  border-radius: 5px;
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid var(--key-color);
  box-shadow: 0 0 7px var(--key-glow);
  transition: border-color 0.25s ease, box-shadow 0.25s ease;
}

.key-row.bottom i.space {
  flex: 5;
}

.kbd-controls-grid {
  display: grid;
  grid-template-columns: 1.05fr 1fr;
  gap: 14px;
  align-items: start;
}

.kbd-controls-grid.single {
  grid-template-columns: 1fr;
}

.controls-card {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: 14px;
}

.ctrl-block {
  display: flex;
  flex-direction: column;
  gap: 9px;
}

.ctrl-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.ctrl-head strong {
  font-size: 13.5px;
}

.swatches {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 9px;
}

.swatch-btn {
  padding: 11px 6px;
  border-radius: var(--radius-md);
  background: var(--glass-sub-bg);
  border: 1px solid var(--glass-divider);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}

.swatch-btn:hover {
  background: var(--pill-hover);
  border-color: var(--glass-border-hover);
}

.swatch-btn.selected {
  background: var(--accent-soft);
  border-color: var(--accent-border);
}

.swatch-circle {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.28);
}

.swatch-btn small {
  font-size: 11.5px;
  color: var(--text-secondary);
}

.fn-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 17px;
}

.fn-card strong {
  font-size: 13.5px;
}

@media (max-width: 880px) {
  .kbd-controls-grid {
    grid-template-columns: 1fr;
  }
}
</style>
