<!-- Author: Aut520 <aut@autkc.com> | https://github.com/Aut520/rmechrevo | GPL-3.0 -->
<script setup lang="ts">
import { computed } from 'vue'
import type { Component } from 'vue'

const props = withDefaults(defineProps<{
  label: string
  value: number | null
  unit: string
  icon: Component
  variant?: 'warm' | 'cool' | 'fan'
}>(), {
  variant: 'warm'
})

const displayValue = computed(() => {
  if (props.value === null) return '--'
  if (props.unit === 'RPM') return Math.round(props.value).toString()
  if (props.unit === '°C') return props.value.toFixed(1)
  return props.value.toString()
})

const barWidth = computed(() => {
  if (props.value === null) return 0
  if (props.variant === 'fan') return Math.min((props.value / 6000) * 100, 100)
  return Math.min(Math.max(props.value, 0), 100)
})
</script>

<template>
  <div class="metric-cell">
    <div class="top">
      <span>{{ label }}</span>
      <component :is="icon" :size="14" class="icon" :class="variant" />
    </div>
    <div class="val tabular">
      <strong>{{ displayValue }}</strong>
      <small>{{ unit }}</small>
    </div>
    <div class="bar" :class="variant">
      <i :style="{ width: `${barWidth}%` }" />
    </div>
  </div>
</template>

<style scoped>
.metric-cell {
  padding: 13px 14px;
  border-radius: var(--radius-md);
  background: var(--glass-sub-bg);
  border: 1px solid var(--glass-divider);
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: 8px;
}

.top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 11.5px;
  color: var(--text-secondary);
}

.icon.warm {
  color: #fb7185;
}

.icon.cool {
  color: var(--color-success);
}

.icon.fan {
  color: var(--accent-secondary);
}

.val {
  display: flex;
  align-items: baseline;
  gap: 3px;
}

.val strong {
  font-size: 22px;
  font-weight: 700;
  line-height: 1.1;
}

.val small {
  font-size: 11px;
  color: var(--text-muted);
}

.bar {
  height: 4px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.12);
  overflow: hidden;
}

.bar i {
  display: block;
  height: 100%;
  border-radius: inherit;
  transition: width 0.35s ease;
}

.bar.warm i {
  background: linear-gradient(90deg, #fb923c, #f43f5e);
}

.bar.cool i {
  background: linear-gradient(90deg, #34d399, #10b981);
}

.bar.fan i {
  background: linear-gradient(90deg, #38bdf8, #818cf8);
}
</style>
