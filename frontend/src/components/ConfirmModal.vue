<!-- Author: Aut520 <aut@autkc.com> | https://github.com/Aut520/rmechrevo | GPL-3.0 -->
<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { AlertTriangle } from 'lucide-vue-next'
import type { ConfirmationInfo } from '../types'

defineProps<{
  confirmation: ConfirmationInfo | null
}>()

const emit = defineEmits<{
  (e: 'cancel'): void
  (e: 'confirm'): void
}>()

function onKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('cancel')
}

onMounted(() => window.addEventListener('keydown', onKeyDown))
onUnmounted(() => window.removeEventListener('keydown', onKeyDown))
</script>

<template>
  <div v-if="confirmation" class="modal-overlay" @click.self="$emit('cancel')">
    <div class="confirm-dialog" role="dialog" aria-modal="true">
      <div class="dialog-head">
        <div class="dialog-icon">
          <AlertTriangle :size="20" />
        </div>
        <h3>{{ confirmation.title }}</h3>
      </div>
      <p class="dialog-detail">{{ confirmation.detail }}</p>
      <div class="dialog-actions">
        <button class="btn btn-secondary" @click="$emit('cancel')">取消</button>
        <button class="btn btn-primary" @click="$emit('confirm')">确定执行</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  z-index: 250;
  background: rgba(0, 0, 0, 0.36);
  backdrop-filter: blur(5px);
  -webkit-backdrop-filter: blur(5px);
  display: grid;
  place-items: center;
  padding: 20px;
  animation: fade-in 0.16s ease;
}

.confirm-dialog {
  width: 100%;
  max-width: 440px;
  padding: 22px 24px;
  border-radius: var(--radius-xl);
  background: var(--glass-drawer-bg);
  backdrop-filter: blur(18px) saturate(145%);
  -webkit-backdrop-filter: blur(18px) saturate(145%);
  border: 1px solid var(--glass-border-hover);
  box-shadow: 0 20px 44px rgba(0, 0, 0, 0.35);
}

.dialog-head {
  display: flex;
  align-items: center;
  gap: 12px;
}

.dialog-icon {
  width: 38px;
  height: 38px;
  border-radius: 10px;
  background: rgba(251, 191, 36, 0.18);
  border: 1px solid rgba(251, 191, 36, 0.38);
  color: var(--color-warning);
  display: grid;
  place-items: center;
  flex-shrink: 0;
}

.dialog-head h3 {
  font-size: 15px;
  font-weight: 700;
}

.dialog-detail {
  margin-top: 12px;
  font-size: 12.5px;
  line-height: 1.65;
  color: var(--text-secondary);
}

.dialog-actions {
  margin-top: 20px;
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>
