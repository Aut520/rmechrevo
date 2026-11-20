<!-- Author: Aut520 <aut@autkc.com> | https://github.com/Aut520/rmechrevo | GPL-3.0 -->
<script setup lang="ts">
import { Lock, Unlock, Undo2, Check } from 'lucide-vue-next'
import type { CustomTuningConfig } from '../../types'

defineProps<{
  tuning: CustomTuningConfig
  cpuUnlocked: boolean
  tuningDirty: boolean
  cpuMaxPl1: number
  cpuMaxPl2: number
  validation: string | null
  busy: boolean
}>()

defineEmits<{
  (e: 'toggle-cpu-unlock'): void
  (e: 'edit-tuning'): void
  (e: 'restore-tuning'): void
  (e: 'apply-tuning'): void
}>()
</script>

<template>
  <section class="glass-card custom-panel">
    <div class="card-head">
      <div class="title-wrap">
        <span class="white-dot" />
        <strong>自定义参数</strong>
      </div>

      <div class="head-actions">
        <div class="pill-switch">
          <button :class="{ active: !cpuUnlocked }" @click="cpuUnlocked && $emit('toggle-cpu-unlock')">
            <Lock :size="11" />
            <span>120W</span>
          </button>
          <button class="danger" :class="{ active: cpuUnlocked }" @click="$emit('toggle-cpu-unlock')">
            <Unlock :size="11" />
            <span>140W</span>
          </button>
        </div>
        <button class="btn btn-secondary btn-sm" @click="$emit('restore-tuning')">
          <Undo2 :size="12" />
          <span>默认</span>
        </button>
        <button class="btn btn-primary btn-sm" :disabled="busy || !!validation" @click="$emit('apply-tuning')">
          <Check :size="12" />
          <span>{{ tuningDirty ? '应用 *' : '应用' }}</span>
        </button>
      </div>
    </div>

    <p v-if="validation" class="val-msg status-err">{{ validation }}</p>

    <div class="tuning-grid">
      <div class="tuning-col">
        <div class="col-title">CPU 功耗</div>
        <label class="slider-item" :class="{ alert: cpuUnlocked }">
          <div class="slider-top">
            <span>SPL 持续</span>
            <div class="num-wrap">
              <input type="number" min="35" :max="cpuMaxPl1" v-model.number="tuning.cpu_pl1" @input="$emit('edit-tuning')" />
              <em>W</em>
            </div>
          </div>
          <input type="range" min="35" :max="cpuMaxPl1" v-model.number="tuning.cpu_pl1" class="glass-range" @input="$emit('edit-tuning')" />
        </label>

        <label class="slider-item" :class="{ alert: cpuUnlocked }">
          <div class="slider-top">
            <span>sPPT 短时</span>
            <div class="num-wrap">
              <input type="number" min="35" :max="cpuMaxPl2" v-model.number="tuning.cpu_pl2" @input="$emit('edit-tuning')" />
              <em>W</em>
            </div>
          </div>
          <input type="range" min="35" :max="cpuMaxPl2" v-model.number="tuning.cpu_pl2" class="glass-range" @input="$emit('edit-tuning')" />
        </label>

        <label class="slider-item">
          <div class="slider-top">
            <span>fPPT 峰值</span>
            <div class="num-wrap">
              <input type="number" min="35" max="215" v-model.number="tuning.cpu_pl4" @input="$emit('edit-tuning')" />
              <em>W</em>
            </div>
          </div>
          <input type="range" min="35" max="215" v-model.number="tuning.cpu_pl4" class="glass-range" @input="$emit('edit-tuning')" />
        </label>
      </div>

      <div class="tuning-col">
        <div class="col-title">GPU 偏移</div>
        <label class="slider-item">
          <div class="slider-top">
            <span>核心频率</span>
            <div class="num-wrap">
              <input type="number" min="-100" max="250" v-model.number="tuning.gpu_core_offset" @input="$emit('edit-tuning')" />
              <em>MHz</em>
            </div>
          </div>
          <input type="range" min="-100" max="250" step="5" v-model.number="tuning.gpu_core_offset" class="glass-range" @input="$emit('edit-tuning')" />
        </label>

        <label class="slider-item">
          <div class="slider-top">
            <span>显存频率</span>
            <div class="num-wrap">
              <input type="number" min="-200" max="1000" v-model.number="tuning.gpu_mem_offset" @input="$emit('edit-tuning')" />
              <em>MHz</em>
            </div>
          </div>
          <input type="range" min="-200" max="1000" step="25" v-model.number="tuning.gpu_mem_offset" class="glass-range" @input="$emit('edit-tuning')" />
        </label>

        <label class="slider-item">
          <div class="slider-top">
            <span>Dynamic Boost</span>
            <div class="num-wrap">
              <input type="number" min="0" max="25" v-model.number="tuning.gpu_dynamic_boost" @input="$emit('edit-tuning')" />
              <em>W</em>
            </div>
          </div>
          <input type="range" min="0" max="25" step="1" v-model.number="tuning.gpu_dynamic_boost" class="glass-range" @input="$emit('edit-tuning')" />
        </label>
      </div>
    </div>
  </section>
</template>

<style scoped>
.custom-panel {
  border-color: rgba(255, 255, 255, 0.35);
  box-shadow: 0 6px 22px rgba(255, 255, 255, 0.08);
  animation: fade-in 0.2s ease;
}

.title-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
}

.white-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #ffffff;
  box-shadow: 0 0 10px rgba(255, 255, 255, 0.9);
}

.head-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.val-msg {
  margin-bottom: 10px;
  font-size: 12px;
}

.tuning-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}

.tuning-col {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.col-title {
  font-size: 11.5px;
  color: var(--text-muted);
  padding-left: 2px;
}

.slider-item {
  padding: 9px 12px;
  border-radius: var(--radius-sm);
  background: var(--glass-sub-bg);
  border: 1px solid var(--glass-divider);
  display: flex;
  flex-direction: column;
  gap: 7px;
}

.slider-item.alert {
  border-color: rgba(248, 113, 113, 0.4);
}

.slider-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  color: var(--text-secondary);
}

.num-wrap {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 1px 7px;
  border-radius: 5px;
  background: var(--glass-input-bg);
  border: 1px solid var(--glass-border);
}

.num-wrap input {
  width: 46px;
  border: 0;
  background: transparent;
  outline: none;
  text-align: right;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  font-family: var(--font-sans);
}

.num-wrap em {
  font-style: normal;
  font-size: 10.5px;
  color: var(--text-muted);
}

@media (max-width: 880px) {
  .tuning-grid {
    grid-template-columns: 1fr;
  }
}
</style>
