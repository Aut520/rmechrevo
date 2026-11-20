<!-- Author: Aut520 <aut@autkc.com> | https://github.com/Aut520/rmechrevo | GPL-3.0 -->
<script setup lang="ts">
import { Cpu, Monitor, Fan } from 'lucide-vue-next'
import TelemetryCard from './TelemetryCard.vue'
import type { Telemetry } from '../../types'

defineProps<{
  telemetry: Telemetry
}>()
</script>

<template>
  <section class="glass-card telemetry-panel">
    <div class="card-head">
      <strong>实时遥测</strong>
    </div>
    <div class="telemetry-matrix">
      <TelemetryCard label="CPU 温度" :value="telemetry.cpu_temp" unit="°C" :icon="Cpu" variant="warm" />
      <TelemetryCard label="GPU 温度" :value="telemetry.gpu_temp" unit="°C" :icon="Monitor" variant="cool" />
      <TelemetryCard label="左风扇" :value="telemetry.fan_cpu_rpm" unit="RPM" :icon="Fan" variant="fan" />
      <TelemetryCard label="右风扇" :value="telemetry.fan_gpu_rpm" unit="RPM" :icon="Fan" variant="fan" />
    </div>
  </section>
</template>

<style scoped>
.telemetry-panel {
  display: flex;
  flex-direction: column;
}

.telemetry-matrix {
  flex: 1;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}
</style>
