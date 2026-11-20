<!-- Author: Aut520 <aut@autkc.com> | https://github.com/Aut520/rmechrevo | GPL-3.0 -->
<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

import type {
  Tab,
  Telemetry,
  CustomTuningConfig,
  BatteryInfo,
  SystemInfo,
  DriverStatus,
  AcpiPatchStatus,
  HibernateStatus,
  ConfirmationInfo,
  RgbState,
  ColorItem
} from './types'

import { defaultTuningConfig, tabs } from './constants'

import TitleBar from './layout/TitleBar.vue'
import Sidebar from './layout/Sidebar.vue'
import ToastNotice from './components/ToastNotice.vue'
import ConfirmModal from './components/ConfirmModal.vue'
import BackgroundSettings from './components/BackgroundSettings.vue'

import OverviewView from './views/OverviewView.vue'
import KeyboardView from './views/KeyboardView.vue'
import PowerView from './views/PowerView.vue'
import SystemView from './views/SystemView.vue'

const active = ref<Tab>('overview')
const connected = ref(false)
const busy = ref(false)
const errorMessage = ref<string | null>(null)
const confirmation = ref<ConfirmationInfo | null>(null)
const showAppearance = ref(false)

const systemInfo = reactive<SystemInfo>({
  product_name: 'Linux 硬件控制中心',
  bios_version: '',
  cpu_name: '',
  gpu_name: '',
  is_uniwill_loaded: false,
  has_hwmon_fans: false,
  has_acpi_call: false
})

const telemetry = reactive<Telemetry>({
  cpu_temp: null,
  gpu_temp: null,
  fan_cpu_rpm: null,
  fan_gpu_rpm: null
})

const currentMode = ref<number>(1)
const fanBoost = ref<boolean>(false)
const turboSubMode = ref<number>(1)
const fnLock = ref<boolean>(false)
const winLock = ref<boolean>(false)
const touchpadDisabled = ref<boolean>(false)
const batteryLimitMode = ref<number>(1)
const gpuMode = ref<string>('独显直连')
const cpuUnlocked = ref<boolean>(false)

const rgb = reactive<RgbState>({
  enabled: true,
  r: 255,
  g: 255,
  b: 255,
  brightness: 100,
  effect_mode: 0
})

const batteryInfo = reactive<BatteryInfo>({
  capacity: 100,
  status: '未充电 (适配器直通)',
  is_ac_online: true,
  voltage: 16.9,
  current_ma: 0,
  power_w: 0.0,
  health_percent: 100,
  cycle_count: 0,
  charge_now_mah: 5200,
  charge_full_mah: 5200,
  charge_design_mah: 5200,
  limit_mode: 1,
  limit_mode_reg: 2,
  limit_up: 80,
  limit_down: 75,
  hardware_status_msg: '当前处于日常平衡养护模式 (80%)。'
})

const driverStatus = reactive<DriverStatus>({
  ready: true,
  acpi_call_loaded: true,
  acpi_call_installed: true,
  uniwill_loaded: true,
  hotkey_ready: true,
  hotkey_device: null,
  hotkey_name: null,
  message: '硬件控制通道已就绪',
  hardware_info: {
    product_name: '--',
    board_name: '--',
    bios_version: '--',
    bios_date: '--',
    ec_version: '--',
    cpu_model: '--',
    gpu_model: '--',
    kernel_version: '--'
  }
})

const acpiPatchStatus = reactive<AcpiPatchStatus>({
  dsdt_applied: false,
  ssdt_sound_applied: false,
  ssdt_sound_needed: false,
  ssdt_d3cold_applied: false,
  ssdt_d3cold_needed: false,
  active_in_kernel: false,
  message: ''
})

const hibernateStatus = reactive<HibernateStatus>({
  kernel_resume_configured: false,
  kernel_resume_param: '',
  swap_uuid: '',
  swap_device: '',
  swap_major_minor: '',
  sys_power_resume: '',
  nvidia_preserve_vram: false,
  nvidia_tmp_on_disk: false,
  nvidia_services_ready: false,
  swap_size_gb: 0,
  ram_size_gb: 0,
  swap_sufficient: false,
  zram_priority_conflict: false,
  dracut_resume_module: false,
  all_healthy: false,
  summary: ''
})

const tuning = reactive<CustomTuningConfig>(defaultTuningConfig())
const tuningDirty = ref<boolean>(false)

const silentTurbo = computed(() => turboSubMode.value === 0)
const cpuMaxPl1 = computed(() => cpuUnlocked.value ? 140 : 120)
const cpuMaxPl2 = computed(() => 140)
const activeTitle = computed(() => tabs.find(t => t.id === active.value)?.name || '概览大厅')

const validation = computed(() => {
  if (![tuning.cpu_pl1, tuning.cpu_pl2, tuning.cpu_pl4, tuning.gpu_dynamic_boost].every(Number.isInteger)) {
    return '请输入整数功耗参数'
  }
  const maxPl1 = cpuUnlocked.value ? 140 : 120
  if (tuning.cpu_pl1 < 35 || tuning.cpu_pl1 > maxPl1) {
    return `SPL 持续功耗范围为 35 ~ ${maxPl1} W`
  }
  if (tuning.cpu_pl2 < tuning.cpu_pl1 || tuning.cpu_pl2 > 140) {
    return 'sPPT 短时功耗须满足 SPL ≤ sPPT ≤ 140 W'
  }
  if (tuning.cpu_pl4 < tuning.cpu_pl2 || tuning.cpu_pl4 > 215) {
    return 'fPPT 峰值功耗须满足 sPPT ≤ fPPT ≤ 215 W'
  }
  if (tuning.gpu_dynamic_boost < 0 || tuning.gpu_dynamic_boost > 25) {
    return 'Dynamic Boost 范围为 0 ~ 25 W'
  }
  if (tuning.gpu_core_offset < -100 || tuning.gpu_core_offset > 250) {
    return 'GPU 核心偏移范围为 -100 ~ +250 MHz'
  }
  if (tuning.gpu_mem_offset < -200 || tuning.gpu_mem_offset > 1000) {
    return 'GPU 显存偏移范围为 -200 ~ +1000 MHz'
  }
  return null
})

function reportError(msg: string) {
  errorMessage.value = msg
}

async function command(action: string, payload: Record<string, any> = {}) {
  busy.value = true
  try {
    await invoke('send_command', { action, payload })
  } catch (err) {
    reportError(`操作失败 (${action}): ${err}`)
  } finally {
    busy.value = false
  }
}

async function selectMode(modeId: number) {
  if (currentMode.value === modeId) return
  currentMode.value = modeId
  if (modeId !== 0 && turboSubMode.value === 0) {
    turboSubMode.value = 1
  }
  fanBoost.value = false
  await command('SetMode', { mode: modeId })
}

async function toggleSilentTurbo() {
  const nextSubMode = turboSubMode.value === 0 ? 1 : 0
  turboSubMode.value = nextSubMode
  await command('SetTurboSubMode', { sub_mode: nextSubMode })
}

async function toggleFanBoost() {
  const next = !fanBoost.value
  fanBoost.value = next
  await command('SetFanBoost', { enabled: next })
}

async function toggleWinLock() {
  const next = !winLock.value
  winLock.value = next
  await command('SetWinLock', { locked: next })
}

async function toggleTouchpad() {
  const next = !touchpadDisabled.value
  touchpadDisabled.value = next
  await command('SetTouchpadDisable', { disabled: next })
}

async function toggleFnLock() {
  const next = !fnLock.value
  fnLock.value = next
  await command('SetFnLock', { enabled: next })
}

function toggleCpuUnlock() {
  cpuUnlocked.value = !cpuUnlocked.value
  if (!cpuUnlocked.value && tuning.cpu_pl1 > 120) {
    tuning.cpu_pl1 = 120
  }
}

async function setRgbPower(enabled: boolean) {
  rgb.enabled = enabled
  if (enabled && rgb.brightness === 0) rgb.brightness = 100
  await command('SetRgbPower', { enabled })
}

async function setRgbBrightness(percent: number) {
  rgb.brightness = percent
  rgb.enabled = percent > 0
  await command('SetRgbBrightness', { percent })
}

async function selectRgbEffect(effectMode: number) {
  rgb.effect_mode = effectMode
  rgb.enabled = true
  if (rgb.brightness === 0) rgb.brightness = 100
  await command('SetRgbEffect', { effect_mode: effectMode })
}

async function selectRgbColor(color: ColorItem) {
  rgb.r = color.value[0]
  rgb.g = color.value[1]
  rgb.b = color.value[2]
  rgb.enabled = true
  if (rgb.brightness === 0) rgb.brightness = 100
  if (rgb.effect_mode === 1) rgb.effect_mode = 0
  await command('SetRgbColor', {
    r: color.value[0],
    g: color.value[1],
    b: color.value[2]
  })
}

function applyBatteryLimitPreview(mode: number) {
  batteryLimitMode.value = mode
  batteryInfo.limit_mode = mode
  if (mode === 2) {
    batteryInfo.limit_up = 60
    batteryInfo.limit_down = 50
    batteryInfo.hardware_status_msg = '长效养护 (60%)：电量达到 60% 时 EC 物理切断充电。'
  } else if (mode === 1) {
    batteryInfo.limit_up = 80
    batteryInfo.limit_down = 75
    batteryInfo.hardware_status_msg = '日常平衡养护 (80%)。'
  } else {
    batteryInfo.limit_up = 100
    batteryInfo.limit_down = 95
    batteryInfo.hardware_status_msg = '常规充满 (100%)。'
  }
}

async function selectBatteryLimit(mode: number) {
  applyBatteryLimitPreview(mode)
  await command('SetBatteryLimit', { mode })
}

function editTuning() {
  tuningDirty.value = true
}

function restoreTuning() {
  Object.assign(tuning, defaultTuningConfig())
  cpuUnlocked.value = false
  tuningDirty.value = true
}

function confirmAction(title: string, detail: string, action: string, payload: Record<string, any> = {}) {
  confirmation.value = { title, detail, action, payload }
}

async function confirmed() {
  const item = confirmation.value
  confirmation.value = null
  if (!item) return
  if (item.action === 'ExitAndRestoreHardware') {
    await quit()
  } else {
    await command(item.action, item.payload)
  }
}

async function windowAction(action: 'minimize_window' | 'toggle_maximize' | 'hide_window') {
  try {
    await invoke(action)
  } catch (err) {
    reportError(`窗口操作失败: ${err}`)
  }
}

async function quit() {
  try {
    await invoke('quit_app')
  } catch (err) {
    reportError(`退出应用失败: ${err}`)
  }
}

function applyFullStatus(p: any) {
  if (p.mode !== undefined) currentMode.value = p.mode
  if (p.turbo_sub_mode !== undefined) turboSubMode.value = p.turbo_sub_mode
  if (p.fan_boost !== undefined) fanBoost.value = p.fan_boost
  if (p.win_lock !== undefined) winLock.value = p.win_lock
  if (p.touchpad_disabled !== undefined) touchpadDisabled.value = p.touchpad_disabled
  if (p.fn_lock !== undefined) fnLock.value = p.fn_lock

  if (p.rgb_power !== undefined) rgb.enabled = p.rgb_power
  if (p.rgb_r !== undefined) rgb.r = p.rgb_r
  if (p.rgb_g !== undefined) rgb.g = p.rgb_g
  if (p.rgb_b !== undefined) rgb.b = p.rgb_b
  if (p.rgb_brightness !== undefined) rgb.brightness = p.rgb_brightness
  if (p.rgb_effect_mode !== undefined) rgb.effect_mode = p.rgb_effect_mode

  if (p.battery_limit_mode !== undefined) {
    batteryLimitMode.value = p.battery_limit_mode
    batteryInfo.limit_mode = p.battery_limit_mode
  }
  if (p.battery_info) Object.assign(batteryInfo, p.battery_info)
  if (p.tuning) {
    Object.assign(tuning, p.tuning)
    if (tuning.cpu_pl1 > 120) cpuUnlocked.value = true
  }
  if (p.gpu_mode) gpuMode.value = p.gpu_mode
  if (p.driver_status) Object.assign(driverStatus, p.driver_status)
  if (p.acpi_patch_status) Object.assign(acpiPatchStatus, p.acpi_patch_status)
  if (p.hibernate_status) Object.assign(hibernateStatus, p.hibernate_status)
}

function handleIpcEvent(event: any) {
  if (!event) return
  const type = event.type || Object.keys(event)[0]
  const p = event.payload || event[type] || event

  switch (type) {
    case 'FullStatus':
      applyFullStatus(p)
      break
    case 'ModeChanged':
      if (p.mode !== undefined) {
        currentMode.value = p.mode
        if (p.mode !== 0) turboSubMode.value = 1
      }
      break
    case 'TurboSubModeChanged':
      if (p.sub_mode !== undefined) turboSubMode.value = p.sub_mode
      break
    case 'FanBoostChanged':
      if (p.enabled !== undefined) fanBoost.value = p.enabled
      break
    case 'RgbChanged':
      if (p.enabled !== undefined) rgb.enabled = p.enabled
      if (p.r !== undefined) rgb.r = p.r
      if (p.g !== undefined) rgb.g = p.g
      if (p.b !== undefined) rgb.b = p.b
      if (p.brightness !== undefined) rgb.brightness = p.brightness
      if (p.effect_mode !== undefined) rgb.effect_mode = p.effect_mode
      break
    case 'BatteryLimitChanged':
      if (p.mode !== undefined) batteryLimitMode.value = p.mode
      if (p.battery_info) Object.assign(batteryInfo, p.battery_info)
      break
    case 'TuningResult':
      tuningDirty.value = false
      break
    case 'SwitchChanged':
      if (p.switch_name === 'win_lock') winLock.value = p.state
      if (p.switch_name === 'touchpad_disabled' || p.switch_name === 'touchpad') touchpadDisabled.value = p.state
      if (p.switch_name === 'fn_lock') fnLock.value = p.state
      break
    case 'GpuModeResult':
      if (p.current_mode) gpuMode.value = p.current_mode
      break
    case 'DriverInstallResult':
      if (p.status) Object.assign(driverStatus, p.status)
      if (p.success === false && p.message) reportError(p.message)
      break
    case 'AcpiPatchResult':
      if (p.status) Object.assign(acpiPatchStatus, p.status)
      if (p.success === false && p.message) reportError(p.message)
      break
    case 'HibernateFixResult':
      if (p.status) Object.assign(hibernateStatus, p.status)
      if (p.success === false && p.message) reportError(p.message)
      break
  }
}

const unlisteners: UnlistenFn[] = []

async function requestFullStatus() {
  try {
    await invoke('send_command', { action: 'GetStatus', payload: {} })
  } catch {}
}

onMounted(async () => {
  try {
    unlisteners.push(
      await listen('ipc-event', (event: any) => handleIpcEvent(event.payload))
    )

    unlisteners.push(
      await listen('connection-status', (event: any) => {
        const payload = event.payload
        const nextConn = typeof payload === 'boolean'
          ? payload
          : Boolean(payload && payload.connected)
        const becameOnline = !connected.value && nextConn
        connected.value = nextConn
        if (becameOnline) {
          requestFullStatus()
        }
      })
    )

    unlisteners.push(
      await listen('telemetry', (event: any) => {
        const p = event.payload
        if (!p) return
        if (p.cpu_temp !== undefined) telemetry.cpu_temp = p.cpu_temp
        if (p.gpu_temp !== undefined) telemetry.gpu_temp = p.gpu_temp
        if (p.fan_cpu_rpm !== undefined) telemetry.fan_cpu_rpm = p.fan_cpu_rpm
        if (p.fan_gpu_rpm !== undefined) telemetry.fan_gpu_rpm = p.fan_gpu_rpm
        if (p.battery) {
          if (p.battery.capacity !== undefined) batteryInfo.capacity = p.battery.capacity
          if (p.battery.voltage !== undefined) batteryInfo.voltage = p.battery.voltage
          if (p.battery.current_ma !== undefined) batteryInfo.current_ma = p.battery.current_ma
          if (p.battery.power_w !== undefined) batteryInfo.power_w = p.battery.power_w
          if (p.battery.is_ac_online !== undefined) batteryInfo.is_ac_online = p.battery.is_ac_online
          if (p.battery.limit_mode !== undefined && p.battery.limit_mode >= 0) {
            batteryInfo.limit_mode = p.battery.limit_mode
            batteryLimitMode.value = p.battery.limit_mode
            if (p.battery.limit_up !== undefined) batteryInfo.limit_up = p.battery.limit_up
            if (p.battery.limit_down !== undefined) batteryInfo.limit_down = p.battery.limit_down
            if (p.battery.status) batteryInfo.status = p.battery.status
            if (p.battery.hardware_status_msg) batteryInfo.hardware_status_msg = p.battery.hardware_status_msg
          }
        }
      })
    )

    try {
      const cfg = await invoke<any>('get_initial_status')
      if (cfg) {
        if (cfg.battery_limit_mode !== undefined) applyBatteryLimitPreview(cfg.battery_limit_mode)
        if (cfg.performance_mode !== undefined) currentMode.value = cfg.performance_mode
        if (cfg.fan_boost !== undefined) fanBoost.value = cfg.fan_boost
        if (cfg.turbo_sub_mode !== undefined) turboSubMode.value = cfg.turbo_sub_mode
        if (cfg.fn_lock !== undefined) fnLock.value = cfg.fn_lock
        if (cfg.win_lock !== undefined) winLock.value = cfg.win_lock
        if (cfg.touchpad_disabled !== undefined) touchpadDisabled.value = cfg.touchpad_disabled
        if (cfg.rgb_power !== undefined) rgb.enabled = cfg.rgb_power
        if (cfg.rgb_r !== undefined) rgb.r = cfg.rgb_r
        if (cfg.rgb_g !== undefined) rgb.g = cfg.rgb_g
        if (cfg.rgb_b !== undefined) rgb.b = cfg.rgb_b
        if (cfg.rgb_brightness !== undefined) rgb.brightness = cfg.rgb_brightness
        if (cfg.rgb_effect_mode !== undefined) rgb.effect_mode = cfg.rgb_effect_mode
        if (cfg.custom_tuning) {
          Object.assign(tuning, cfg.custom_tuning)
          if (tuning.cpu_pl1 > 120) cpuUnlocked.value = true
        }
      }
    } catch {}

    try {
      const info = await invoke<SystemInfo>('get_system_info')
      if (info) {
        Object.assign(systemInfo, info)
        if (info.gpu_mode) gpuMode.value = info.gpu_mode
      }
    } catch {}

    try {
      const hStatus = await invoke<HibernateStatus>('get_hibernate_status')
      if (hStatus) Object.assign(hibernateStatus, hStatus)
    } catch {}

    try {
      const aStatus = await invoke<AcpiPatchStatus>('get_acpi_patch_status')
      if (aStatus) Object.assign(acpiPatchStatus, aStatus)
    } catch {}

    try {
      connected.value = await invoke<boolean>('check_ipc_connected')
    } catch {}

    await requestFullStatus()
    const retryTimer = setTimeout(() => {
      if (driverStatus.hardware_info.board_name === '--') {
        requestFullStatus()
      }
    }, 800)
    unlisteners.push(() => clearTimeout(retryTimer))

    const pollTimer = setInterval(async () => {
      try {
        const wasOffline = !connected.value
        connected.value = await invoke<boolean>('check_ipc_connected')
        if ((wasOffline && connected.value) || (connected.value && driverStatus.hardware_info.board_name === '--')) {
          requestFullStatus()
        }
      } catch {}
    }, 5000)
    unlisteners.push(() => clearInterval(pollTimer))
  } catch (err) {
    reportError(`初始化失败: ${err}`)
  }
})

onUnmounted(() => {
  unlisteners.forEach(stop => stop())
})
</script>

<template>
  <div class="app-shell">
    <div class="app-wallpaper" />

    <div class="app-container">
      <Sidebar
        :active="active"
        :connected="connected"
        :system-info="systemInfo"
        @select="active = $event"
        @open-settings="showAppearance = true"
      />

      <div class="app-stage">
        <TitleBar
          :title="activeTitle"
          @window-action="windowAction"
        />

        <main class="app-main">
          <OverviewView
            v-if="active === 'overview'"
            :telemetry="telemetry"
            :current-mode="currentMode"
            :silent-turbo="silentTurbo"
            :fan-boost="fanBoost"
            :win-lock="winLock"
            :touchpad-disabled="touchpadDisabled"
            :gpu-mode="gpuMode"
            :tuning="tuning"
            :cpu-unlocked="cpuUnlocked"
            :tuning-dirty="tuningDirty"
            :cpu-max-pl1="cpuMaxPl1"
            :cpu-max-pl2="cpuMaxPl2"
            :validation="validation"
            :busy="busy"
            @select-mode="selectMode"
            @toggle-silent-turbo="toggleSilentTurbo"
            @toggle-fan-boost="toggleFanBoost"
            @toggle-win-lock="toggleWinLock"
            @toggle-touchpad="toggleTouchpad"
            @toggle-cpu-unlock="toggleCpuUnlock"
            @edit-tuning="editTuning"
            @restore-tuning="restoreTuning"
            @apply-tuning="command('SetCustomTuning', { config: { ...tuning } })"
            @confirm-action="confirmAction"
          />

          <KeyboardView
            v-else-if="active === 'keyboard'"
            :rgb="rgb"
            :fn-lock="fnLock"
            @set-rgb-power="setRgbPower"
            @set-rgb-brightness="setRgbBrightness"
            @select-rgb-effect="selectRgbEffect"
            @select-rgb-color="selectRgbColor"
            @toggle-fn-lock="toggleFnLock"
          />

          <PowerView
            v-else-if="active === 'power'"
            :battery-info="batteryInfo"
            :battery-limit-mode="batteryLimitMode"
            :gpu-mode="gpuMode"
            :busy="busy"
            @refresh-status="command('GetStatus')"
            @select-battery-limit="selectBatteryLimit"
            @confirm-action="confirmAction"
          />

          <SystemView
            v-else
            :driver-status="driverStatus"
            :hibernate-status="hibernateStatus"
            :acpi-patch-status="acpiPatchStatus"
            :busy="busy"
            @refresh-status="command('GetStatus')"
            @confirm-action="confirmAction"
          />
        </main>
      </div>
    </div>

    <ToastNotice :error-message="errorMessage" @close="errorMessage = null" />

    <BackgroundSettings v-if="showAppearance" @close="showAppearance = false" />

    <ConfirmModal :confirmation="confirmation" @cancel="confirmation = null" @confirm="confirmed" />
  </div>
</template>
