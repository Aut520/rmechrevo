<!-- Author: Aut520 <aut@autkc.com> | https://github.com/Aut520/rmechrevo | GPL-3.0 -->
<script setup lang="ts">
import { RefreshCw, Download, ShieldCheck, Undo2 } from 'lucide-vue-next'
import type { DriverStatus, HibernateStatus, AcpiPatchStatus } from '../types'

defineProps<{
  driverStatus: DriverStatus
  hibernateStatus: HibernateStatus
  acpiPatchStatus: AcpiPatchStatus
  busy: boolean
}>()

defineEmits<{
  (e: 'refresh-status'): void
  (e: 'confirm-action', title: string, detail: string, action: string, payload?: Record<string, any>): void
}>()
</script>

<template>
  <div class="bento-page">
    <div class="sys-row-top">
      <section class="glass-card">
        <div class="card-head">
          <strong>设备指纹</strong>
          <button class="btn btn-secondary btn-sm" @click="$emit('refresh-status')">
            <RefreshCw :size="12" :class="{ spinning: busy }" />
            <span>刷新</span>
          </button>
        </div>
        <div class="spec-grid">
          <div class="spec-item">
            <span>机型</span>
            <strong>{{ driverStatus.hardware_info.product_name }}</strong>
          </div>
          <div class="spec-item">
            <span>主板</span>
            <strong>{{ driverStatus.hardware_info.board_name }}</strong>
          </div>
          <div class="spec-item">
            <span>BIOS / EC</span>
            <strong class="tabular">{{ driverStatus.hardware_info.bios_version }} · v{{ driverStatus.hardware_info.ec_version }}</strong>
          </div>
          <div class="spec-item">
            <span>处理器</span>
            <strong>{{ driverStatus.hardware_info.cpu_model }}</strong>
          </div>
          <div class="spec-item">
            <span>独立显卡</span>
            <strong>{{ driverStatus.hardware_info.gpu_model }}</strong>
          </div>
          <div class="spec-item">
            <span>内核</span>
            <strong class="tabular">{{ driverStatus.hardware_info.kernel_version }}</strong>
          </div>
        </div>
      </section>

      <section class="glass-card flex-card">
        <div>
          <div class="card-head">
            <div class="head-title-row">
              <strong>驱动通道</strong>
              <span :class="driverStatus.ready ? 'status-ok' : 'status-warn'" class="mini-status">
                {{ driverStatus.ready ? '就绪' : '异常' }}
              </span>
            </div>
            <button
              class="btn btn-secondary btn-sm"
              @click="$emit('confirm-action', '加载驱动', '安装并加载 acpi_call 模块。', 'InstallDrivers')"
            >
              <Download :size="12" />
              <span>加载驱动</span>
            </button>
          </div>
          <div class="info-list">
            <div class="info-row">
              <span>acpi_call</span>
              <strong :class="driverStatus.acpi_call_loaded ? 'status-ok' : 'status-warn'">
                {{ driverStatus.acpi_call_loaded ? '已挂载' : '未挂载' }}
              </strong>
            </div>
            <div class="info-row">
              <span>Uniwill</span>
              <strong :class="driverStatus.uniwill_loaded ? 'status-ok' : 'status-muted'">
                {{ driverStatus.uniwill_loaded ? '已加载' : '未加载' }}
              </strong>
            </div>
            <div class="info-row">
              <span>物理按键</span>
              <strong class="status-ok tabular">{{ driverStatus.hotkey_device || '就绪' }}</strong>
            </div>
          </div>
        </div>
      </section>
    </div>

    <div class="sys-row-bottom">
      <section class="glass-card flex-card">
        <div>
          <div class="card-head">
            <strong>S4 休眠防黑屏</strong>
            <span :class="hibernateStatus.all_healthy ? 'status-ok' : 'status-warn'">
              {{ hibernateStatus.all_healthy ? '就绪' : '待加固' }}
            </span>
          </div>
          <div class="info-list">
            <div class="info-row">
              <span>resume 参数</span>
              <strong :class="hibernateStatus.kernel_resume_configured ? 'status-ok' : 'status-err'">
                {{ hibernateStatus.kernel_resume_configured ? '已配置' : '未配置' }}
              </strong>
            </div>
            <div class="info-row">
              <span>SWAP</span>
              <strong class="status-ok tabular">{{ hibernateStatus.swap_size_gb }}G / {{ hibernateStatus.ram_size_gb }}G</strong>
            </div>
            <div class="info-row">
              <span>显存落盘</span>
              <strong :class="hibernateStatus.nvidia_tmp_on_disk ? 'status-ok' : 'status-warn'">
                {{ hibernateStatus.nvidia_tmp_on_disk ? '/var/tmp' : '未配置' }}
              </strong>
            </div>
            <div class="info-row">
              <span>休眠服务 · dracut</span>
              <strong :class="hibernateStatus.nvidia_services_ready && hibernateStatus.dracut_resume_module ? 'status-ok' : 'status-warn'">
                {{ hibernateStatus.nvidia_services_ready ? '已启用' : '未启用' }} · {{ hibernateStatus.dracut_resume_module ? '已注入' : '未注入' }}
              </strong>
            </div>
          </div>
        </div>

        <div class="btn-row">
          <button
            class="btn btn-secondary btn-sm"
            :disabled="busy"
            @click="$emit('confirm-action', '回滚休眠配置', '移除已部署的休眠引导配置与 dracut 模块。', 'RollbackHibernateFix')"
          >
            <Undo2 :size="12" />
            <span>回滚</span>
          </button>
          <button
            class="btn btn-primary btn-sm"
            :disabled="busy"
            @click="$emit('confirm-action', '加固 S4 休眠链路', '配置 GRUB resume、NVIDIA 显存落盘并重建引导映像。', 'InstallHibernateFix')"
          >
            <ShieldCheck :size="12" />
            <span>一键加固</span>
          </button>
        </div>
      </section>

      <section class="glass-card flex-card">
        <div>
          <div class="card-head">
            <strong>S3 睡眠防护</strong>
            <span :class="acpiPatchStatus.dsdt_applied ? 'status-ok' : 'status-muted'">
              {{ acpiPatchStatus.dsdt_applied ? '已部署' : '未部署' }}
            </span>
          </div>
          <div class="info-list">
            <div class="info-row">
              <span>DSDT (PEHP=0)</span>
              <strong :class="acpiPatchStatus.dsdt_applied ? 'status-ok' : 'status-muted'">
                {{ acpiPatchStatus.dsdt_applied ? (acpiPatchStatus.active_in_kernel ? '生效中' : '待重启') : '未部署' }}
              </strong>
            </div>
            <div class="info-row">
              <span>SSDT 声卡</span>
              <strong :class="acpiPatchStatus.ssdt_sound_needed ? (acpiPatchStatus.ssdt_sound_applied ? 'status-ok' : 'status-muted') : 'status-accent'">
                {{ acpiPatchStatus.ssdt_sound_needed ? (acpiPatchStatus.ssdt_sound_applied ? '已挂载' : '未挂载') : '无需部署' }}
              </strong>
            </div>
            <div class="info-row">
              <span>SSDT D3cold</span>
              <strong :class="acpiPatchStatus.ssdt_d3cold_needed ? (acpiPatchStatus.ssdt_d3cold_applied ? 'status-ok' : 'status-muted') : 'status-accent'">
                {{ acpiPatchStatus.ssdt_d3cold_needed ? (acpiPatchStatus.ssdt_d3cold_applied ? '已挂载' : '未挂载') : '无需部署' }}
              </strong>
            </div>
          </div>
        </div>

        <div class="btn-row">
          <button
            class="btn btn-secondary btn-sm"
            :disabled="busy || !acpiPatchStatus.dsdt_applied"
            @click="$emit('confirm-action', '回滚 ACPI 补丁', '移除已部署的 DSDT / SSDT 补丁，重启生效。', 'RollbackAcpiPatches')"
          >
            <Undo2 :size="12" />
            <span>回滚</span>
          </button>
          <button
            class="btn btn-primary btn-sm"
            :disabled="busy"
            @click="$emit('confirm-action', '部署 ACPI 补丁', '注入 DSDT PEHP=0 并重建内核引导映像，重启生效。', 'InstallAcpiPatches')"
          >
            <ShieldCheck :size="12" />
            <span>部署 / 更新</span>
          </button>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.sys-row-top {
  display: grid;
  grid-template-columns: 1.35fr 1fr;
  gap: 14px;
  align-items: start;
}

.sys-row-bottom {
  display: grid;
  grid-template-columns: 1.12fr 1fr;
  gap: 14px;
  align-items: stretch;
}

.flex-card {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
}

.head-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.mini-status {
  font-size: 11.5px;
}

.spec-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.spec-item {
  padding: 8px 11px;
  border-radius: var(--radius-sm);
  background: var(--glass-sub-bg);
  border: 1px solid var(--glass-divider);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.spec-item span {
  font-size: 11px;
  color: var(--text-muted);
}

.spec-item strong {
  font-size: 12px;
  word-break: break-word;
}

@media (max-width: 900px) {
  .sys-row-top,
  .sys-row-bottom {
    grid-template-columns: 1fr;
  }
}
</style>
