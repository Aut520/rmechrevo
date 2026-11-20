// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

import type { Component } from 'vue'

export type Tab = 'overview' | 'keyboard' | 'power' | 'system'

export interface Telemetry {
  cpu_temp: number | null
  gpu_temp: number | null
  fan_cpu_rpm: number | null
  fan_gpu_rpm: number | null
}

export interface CustomTuningConfig {
  cpu_pl1: number
  cpu_pl2: number
  cpu_pl4: number
  gpu_core_offset: number
  gpu_mem_offset: number
  gpu_dynamic_boost: number
  fan_left_duty: number
  fan_right_duty: number
}

export interface BatteryInfo {
  capacity: number
  status: string
  is_ac_online: boolean
  voltage: number
  current_ma: number
  power_w: number
  health_percent: number
  cycle_count: number
  charge_now_mah: number
  charge_full_mah: number
  charge_design_mah: number
  limit_mode: number
  limit_mode_reg?: number
  limit_up: number
  limit_down: number
  hardware_status_msg: string
}

export interface SystemHardwareInfo {
  product_name: string
  board_name: string
  bios_version: string
  bios_date: string
  ec_version: string
  cpu_model: string
  gpu_model: string
  kernel_version: string
}

export interface DriverStatus {
  ready: boolean
  acpi_call_loaded: boolean
  acpi_call_installed: boolean
  uniwill_loaded: boolean
  hotkey_ready: boolean
  hotkey_device: string | null
  hotkey_name: string | null
  message: string
  hardware_info: SystemHardwareInfo
}

export interface AcpiPatchStatus {
  dsdt_applied: boolean
  ssdt_sound_applied: boolean
  ssdt_sound_needed: boolean
  ssdt_d3cold_applied: boolean
  ssdt_d3cold_needed: boolean
  active_in_kernel: boolean
  message: string
}

export interface HibernateStatus {
  kernel_resume_configured: boolean
  kernel_resume_param: string
  swap_uuid: string
  swap_device: string
  swap_major_minor: string
  sys_power_resume: string
  nvidia_preserve_vram: boolean
  nvidia_tmp_on_disk: boolean
  nvidia_services_ready: boolean
  swap_size_gb: number
  ram_size_gb: number
  swap_sufficient: boolean
  zram_priority_conflict: boolean
  dracut_resume_module: boolean
  all_healthy: boolean
  summary: string
}

export interface SystemInfo {
  product_name: string
  bios_version: string
  cpu_name: string
  gpu_name: string
  gpu_mode?: string
  is_uniwill_loaded: boolean
  has_hwmon_fans: boolean
  has_acpi_call: boolean
}

export interface NoticeInfo {
  message: string
  error: boolean
}

export interface ConfirmationInfo {
  title: string
  detail: string
  action: string
  payload: Record<string, any>
}

export interface HistoryItem {
  time: string
  message: string
  success: boolean
}

export interface RgbState {
  enabled: boolean
  r: number
  g: number
  b: number
  brightness: number
  effect_mode: number
}

export interface TabItem {
  id: Tab
  name: string
  icon: Component
}

export interface ModeItem {
  id: number
  name: string
  english: string
  icon: Component
  color: string
  desc: string
}

export interface ColorItem {
  name: string
  value: [number, number, number]
  hex: string
}

export interface BatteryModeItem {
  mode: number
  percent: number
  label: string
  desc: string
}
