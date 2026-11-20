<!-- Author: Aut520 <aut@autkc.com> | https://github.com/Aut520/rmechrevo | GPL-3.0 -->
<script setup lang="ts">
import TelemetryDashboard from '../components/overview/TelemetryDashboard.vue'
import ModeSelector from '../components/overview/ModeSelector.vue'
import CustomTuningPanel from '../components/overview/CustomTuningPanel.vue'
import HardwareSwitches from '../components/overview/HardwareSwitches.vue'
import TopologyBanner from '../components/overview/TopologyBanner.vue'
import type { Telemetry, CustomTuningConfig } from '../types'

defineProps<{
  telemetry: Telemetry
  currentMode: number
  silentTurbo: boolean
  fanBoost: boolean
  winLock: boolean
  touchpadDisabled: boolean
  gpuMode: string
  tuning: CustomTuningConfig
  cpuUnlocked: boolean
  tuningDirty: boolean
  cpuMaxPl1: number
  cpuMaxPl2: number
  validation: string | null
  busy: boolean
}>()

defineEmits<{
  (e: 'select-mode', modeId: number): void
  (e: 'toggle-silent-turbo'): void
  (e: 'toggle-fan-boost'): void
  (e: 'toggle-win-lock'): void
  (e: 'toggle-touchpad'): void
  (e: 'toggle-cpu-unlock'): void
  (e: 'edit-tuning'): void
  (e: 'restore-tuning'): void
  (e: 'apply-tuning'): void
  (e: 'confirm-action', title: string, detail: string, action: string, payload?: Record<string, any>): void
}>()
</script>

<template>
  <div class="bento-page">
    <div class="bento-row-top">
      <ModeSelector
        :current-mode="currentMode"
        :silent-turbo="silentTurbo"
        :busy="busy"
        @select-mode="modeId => $emit('select-mode', modeId)"
        @toggle-silent-turbo="$emit('toggle-silent-turbo')"
      />
      <TelemetryDashboard :telemetry="telemetry" />
    </div>

    <CustomTuningPanel
      v-if="currentMode === 3"
      :tuning="tuning"
      :cpu-unlocked="cpuUnlocked"
      :tuning-dirty="tuningDirty"
      :cpu-max-pl1="cpuMaxPl1"
      :cpu-max-pl2="cpuMaxPl2"
      :validation="validation"
      :busy="busy"
      @toggle-cpu-unlock="$emit('toggle-cpu-unlock')"
      @edit-tuning="$emit('edit-tuning')"
      @restore-tuning="$emit('restore-tuning')"
      @apply-tuning="$emit('apply-tuning')"
    />

    <div class="bento-row-bottom">
      <HardwareSwitches
        :fan-boost="fanBoost"
        :win-lock="winLock"
        :touchpad-disabled="touchpadDisabled"
        :busy="busy"
        @toggle-fan-boost="$emit('toggle-fan-boost')"
        @toggle-win-lock="$emit('toggle-win-lock')"
        @toggle-touchpad="$emit('toggle-touchpad')"
      />
      <TopologyBanner
        :gpu-mode="gpuMode"
        :busy="busy"
        @confirm-action="(title, detail, action, payload) => $emit('confirm-action', title, detail, action, payload)"
      />
    </div>
  </div>
</template>

<style scoped>
.bento-row-top {
  display: grid;
  grid-template-columns: 1.38fr 1fr;
  gap: 14px;
  align-items: stretch;
}

.bento-row-bottom {
  display: grid;
  grid-template-columns: 1fr 1.12fr;
  gap: 14px;
  align-items: stretch;
}

@media (max-width: 920px) {
  .bento-row-top,
  .bento-row-bottom {
    grid-template-columns: 1fr;
  }
}
</style>
