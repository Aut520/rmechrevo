<!-- Author: Aut520 <aut@autkc.com> | https://github.com/Aut520/rmechrevo | GPL-3.0 -->
<script setup lang="ts">
import { watch } from 'vue'
import { AlertTriangle, X } from 'lucide-vue-next'

const props = defineProps<{
  errorMessage: string | null
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

let timer: ReturnType<typeof setTimeout> | null = null

watch(() => props.errorMessage, (val) => {
  if (timer) {
    clearTimeout(timer)
    timer = null
  }
  if (val) {
    timer = setTimeout(() => emit('close'), 5000)
  }
})
</script>

<template>
  <div v-if="errorMessage" role="alert" class="error-toast">
    <AlertTriangle :size="15" class="toast-icon" />
    <span class="toast-msg">{{ errorMessage }}</span>
    <button class="toast-close" title="关闭" @click="$emit('close')">
      <X :size="13" />
    </button>
  </div>
</template>

<style scoped>
.error-toast {
  position: fixed;
  right: 20px;
  bottom: 20px;
  z-index: 300;
  max-width: 400px;
  padding: 10px 14px;
  border-radius: var(--radius-md);
  background: rgba(30, 12, 16, 0.78);
  backdrop-filter: blur(14px);
  -webkit-backdrop-filter: blur(14px);
  border: 1px solid rgba(248, 113, 113, 0.48);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.32);
  color: #fee2e2;
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 12.5px;
  animation: toast-up 0.2s cubic-bezier(0.22, 1, 0.36, 1);
}

@keyframes toast-up {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.toast-icon {
  color: #f87171;
  flex-shrink: 0;
}

.toast-msg {
  flex: 1;
  line-height: 1.45;
}

.toast-close {
  width: 24px;
  height: 24px;
  border-radius: 6px;
  display: grid;
  place-items: center;
  color: rgba(254, 226, 226, 0.75);
}

.toast-close:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #ffffff;
}
</style>
