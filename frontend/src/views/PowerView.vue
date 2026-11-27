<!-- Author: Aut520 <aut@autkc.com> | https://github.com/Aut520/rmechrevo | GPL-3.0 -->
<script setup lang="ts">
import { computed } from 'vue'
import {
  BatteryCharging,
  Zap,
  ZapOff,
  Monitor,
  RefreshCw,
  Maximize2,
  Check
} from 'lucide-vue-next'
import { batteryModes } from '../constants'
import type { BatteryInfo } from '../types'

const props = defineProps<{
  batteryInfo: BatteryInfo
  batteryLimitMode: number
  gpuMode: string
  busy: boolean
}>()

defineEmits<{
  (e: 'refresh-status'): void
  (e: 'select-battery-limit', mode: number): void
  (e: 'confirm-action', title: string, detail: string, action: string, payload?: Record<string, any>): void
}>()

const isDirectGpu = computed(() => props.gpuMode.includes('直连') || props.gpuMode.includes('独显'))
</script>

<template>
  <div class="bento-page">
    <div class="power-grid">
      <section class="glass-card bat-hero">
        <div class="card-head">
          <strong>电池状态</strong>
          <button class="btn btn-secondary btn-sm" @click="$emit('refresh-status')">
            <RefreshCw :size="12" :class="{ spinning: busy }" />
            <span>刷新</span>
          </button>
        </div>

        <div class="bat-main">
          <div class="bat-pct tabular">{{ batteryInfo.capacity }}<small>%</small></div>
          <span class="bat-badge" :class="batteryInfo.is_ac_online ? 'ac' : 'dc'">
            <BatteryCharging :size="13" />
            {{ batteryInfo.status }}
          </span>
        </div>

        <div class="bat-metrics">
          <div class="mini-cell">
            <span>电压</span>
            <strong class="tabular">{{ batteryInfo.voltage.toFixed(2) }} V</strong>
          </div>
          <div class="mini-cell">
            <span>功率</span>
            <strong class="tabular">{{ batteryInfo.power_w.toFixed(1) }} W</strong>
          </div>
          <div class="mini-cell">
            <span>健康度</span>
            <strong class="tabular status-ok">{{ batteryInfo.health_percent }}% · {{ batteryInfo.cycle_count }}次</strong>
          </div>
          <div class="mini-cell">
            <span>容量</span>
            <strong class="tabular">{{ (batteryInfo.charge_full_mah / 1000).toFixed(1) }} / {{ (batteryInfo.charge_design_mah / 1000).toFixed(1) }} Ah</strong>
          </div>
        </div>
      </section>

      <div class="right-col">
        <section class="glass-card">
          <div class="card-head">
            <strong>充电上限</strong>
            <span class="card-head-sub tabular">{{ batteryInfo.limit_down }}% ~ {{ batteryInfo.limit_up }}%</span>
          </div>

          <div class="limit-grid">
            <button
              v-for="b in batteryModes"
              :key="b.mode"
              class="limit-card"
              :class="{ selected: batteryLimitMode === b.mode }"
              @click="$emit('select-battery-limit', b.mode)"
            >
              <div class="limit-head">
                <strong class="tabular">{{ b.percent }}<small>%</small></strong>
                <Check v-if="batteryLimitMode === b.mode" :size="14" class="limit-check" />
              </div>
              <div class="limit-label">{{ b.label }}</div>
            </button>
          </div>
        </section>

        <section class="glass-card gpu-controls">
          <div class="ctrl-row">
            <div class="ctrl-info">
              <strong>显卡输出 (MUX)</strong>
              <span class="status-accent">{{ gpuMode }}</span>
            </div>
            <div class="ctrl-btns">
              <button
                class="btn btn-secondary btn-sm"
                @click="$emit('confirm-action', '切换为混合输出 (Optimus)', '写入 MUX NVRAM，重启后生效。', 'SwitchGpuMode', { to_direct: false })"
              >
                <Monitor :size="13" />
                <span>混合输出</span>
              </button>
              <button
                class="btn btn-primary btn-sm"
                @click="$emit('confirm-action', '切换为独显直连', '写入 MUX NVRAM，重启后生效。', 'SwitchGpuMode', { to_direct: true })"
              >
                <Maximize2 :size="13" />
                <span>独显直连</span>
              </button>
            </div>
          </div>

          <div class="ctrl-row">
            <div class="ctrl-info">
              <strong>运行时独显供电</strong>
              <span :class="isDirectGpu ? 'status-warn' : 'status-ok'">
                {{ isDirectGpu ? '直连锁定' : '混合可用' }}
              </span>
            </div>
            <div class="ctrl-btns">
              <button
                class="btn btn-primary btn-sm"
                @click="$emit('confirm-action', '恢复独显供电', '重新接通 RTX 独立显卡 ACPI 供电。', 'SetDgpuRuntimePower', { power_on: true })"
              >
                <Zap :size="13" />
                <span>恢复供电</span>
              </button>
              <button
                class="btn btn-secondary btn-sm"
                :disabled="isDirectGpu"
                @click="$emit('confirm-action', '切断独显供电', '断开 RTX 独显供电，仅以核显低功耗运行。', 'SetDgpuRuntimePower', { power_on: false })"
              >
                <ZapOff :size="13" />
                <span>切断供电</span>
              </button>
            </div>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>

<style scoped>
.power-grid {
  display: grid;
  grid-template-columns: 1fr 1.25fr;
  gap: 14px;
  align-items: start;
}

.right-col {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.bat-hero {
  display: flex;
  flex-direction: column;
}

.bat-main {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-bottom: 14px;
}

.bat-pct {
  font-size: 38px;
  font-weight: 800;
  line-height: 1;
}

.bat-pct small {
  margin-left: 2px;
  font-size: 15px;
  color: var(--text-secondary);
}

.bat-badge {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  border-radius: var(--radius-pill);
  font-size: 11.5px;
}

.bat-badge.ac {
  background: rgba(52, 211, 153, 0.2);
  color: #a7f3d0;
}

.bat-badge.dc {
  background: rgba(251, 191, 36, 0.2);
  color: #fde68a;
}

:global(:root[data-theme="light"]) .bat-badge.ac {
  background: rgba(5, 150, 105, 0.16);
  color: #047857;
}

:global(:root[data-theme="light"]) .bat-badge.dc {
  background: rgba(217, 119, 6, 0.16);
  color: #b45309;
}

.bat-metrics {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.mini-cell {
  padding: 9px 11px;
  border-radius: var(--radius-sm);
  background: var(--glass-sub-bg);
  border: 1px solid var(--glass-divider);
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.mini-cell span {
  font-size: 11px;
  color: var(--text-muted);
}

.mini-cell strong {
  font-size: 12.5px;
}

.limit-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
}

.limit-card {
  text-align: left;
  padding: 12px 13px;
  border-radius: var(--radius-md);
  background: var(--glass-sub-bg);
  border: 1px solid var(--glass-border);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.limit-card:hover {
  background: var(--pill-hover);
  border-color: var(--glass-border-hover);
}

.limit-card.selected {
  background: var(--accent-soft);
  border-color: var(--accent-border);
  box-shadow: 0 4px 14px var(--accent-glow);
}

.limit-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.limit-head strong {
  font-size: 20px;
  font-weight: 700;
  line-height: 1.1;
}

.limit-head small {
  font-size: 11px;
  margin-left: 2px;
  color: var(--text-secondary);
}

.limit-check {
  color: var(--accent);
}

.limit-label {
  font-size: 12px;
  color: var(--text-secondary);
}

.gpu-controls {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.ctrl-row {
  padding: 10px 12px;
  border-radius: var(--radius-md);
  background: var(--glass-sub-bg);
  border: 1px solid var(--glass-divider);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.ctrl-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.ctrl-info strong {
  font-size: 13px;
  font-weight: 700;
}

.ctrl-info span {
  font-size: 11.5px;
}

.ctrl-btns {
  display: flex;
  align-items: center;
  gap: 7px;
  flex-shrink: 0;
}

@media (max-width: 920px) {
  .power-grid {
    grid-template-columns: 1fr;
  }
}
</style>
