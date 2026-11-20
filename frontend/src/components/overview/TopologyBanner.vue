<!-- Author: Aut520 <aut@autkc.com> | https://github.com/Aut520/rmechrevo | GPL-3.0 -->
<script setup lang="ts">
import { computed } from 'vue'
import { Monitor, Cpu, Check } from 'lucide-vue-next'

const props = defineProps<{
  gpuMode: string
  busy: boolean
}>()

defineEmits<{
  (e: 'confirm-action', title: string, detail: string, action: string, payload?: Record<string, any>): void
}>()

const isDirectMode = computed(() => props.gpuMode.includes('独显') || props.gpuMode.includes('直连'))
</script>

<template>
  <section class="glass-card gpu-card">
    <div class="card-head">
      <strong>显卡输出 (MUX)</strong>
      <span class="gpu-tag" :class="isDirectMode ? 'direct' : 'hybrid'">
        <i />
        {{ isDirectMode ? '独显直连' : '混合输出' }}
      </span>
    </div>

    <div class="mux-grid">
      <button
        class="mux-option"
        :class="{ active: isDirectMode }"
        :disabled="busy || isDirectMode"
        @click="
          $emit(
            'confirm-action',
            '切换为独显直连 (Direct)',
            '写入 MUX NVRAM，重启后生效。确定切换？',
            'SwitchGpuMode',
            { to_direct: true }
          )
        "
      >
        <div class="mux-top">
          <div class="mux-icon direct">
            <Monitor :size="18" />
          </div>
          <Check v-if="isDirectMode" :size="15" class="mux-check" />
        </div>
        <strong>独显直连</strong>
        <small>dGPU Only</small>
      </button>

      <button
        class="mux-option"
        :class="{ active: !isDirectMode }"
        :disabled="busy || !isDirectMode"
        @click="
          $emit(
            'confirm-action',
            '切换为混合输出 (Hybrid)',
            '写入 MUX NVRAM，重启后生效。确定切换？',
            'SwitchGpuMode',
            { to_direct: false }
          )
        "
      >
        <div class="mux-top">
          <div class="mux-icon hybrid">
            <Cpu :size="18" />
          </div>
          <Check v-if="!isDirectMode" :size="15" class="mux-check" />
        </div>
        <strong>混合输出</strong>
        <small>Hybrid Optimus</small>
      </button>
    </div>
  </section>
</template>

<style scoped>
.gpu-card {
  display: flex;
  flex-direction: column;
}

.gpu-tag {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 2px 9px;
  border-radius: var(--radius-pill);
  font-size: 11px;
}

.gpu-tag i {
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.gpu-tag.direct {
  background: rgba(56, 189, 248, 0.18);
  border: 1px solid rgba(56, 189, 248, 0.42);
  color: #bae6fd;
}

.gpu-tag.direct i {
  background: #38bdf8;
}

.gpu-tag.hybrid {
  background: rgba(52, 211, 153, 0.18);
  border: 1px solid rgba(52, 211, 153, 0.42);
  color: #a7f3d0;
}

.gpu-tag.hybrid i {
  background: #34d399;
}

.mux-grid {
  flex: 1;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.mux-option {
  text-align: left;
  padding: 13px 14px;
  border-radius: var(--radius-md);
  background: var(--glass-sub-bg);
  border: 1px solid var(--glass-border);
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 6px;
}

.mux-option:disabled {
  opacity: 1;
  cursor: default;
}

.mux-option:not(:disabled):hover {
  background: var(--pill-hover);
  border-color: var(--glass-border-hover);
  transform: translateY(-1px);
}

.mux-option.active {
  background: var(--accent-soft);
  border-color: var(--accent-border);
  box-shadow: 0 4px 14px var(--accent-glow);
}

.mux-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 4px;
}

.mux-icon {
  width: 34px;
  height: 34px;
  border-radius: 9px;
  display: grid;
  place-items: center;
}

.mux-icon.direct {
  background: rgba(56, 189, 248, 0.2);
  color: #38bdf8;
}

.mux-icon.hybrid {
  background: rgba(52, 211, 153, 0.2);
  color: #34d399;
}

.mux-check {
  color: var(--accent);
}

.mux-option strong {
  font-size: 13.5px;
  font-weight: 700;
}

.mux-option small {
  font-size: 11px;
  color: var(--text-muted);
}
</style>
