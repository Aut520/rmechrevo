// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

import {
  LayoutGrid,
  SlidersHorizontal,
  Keyboard,
  BatteryCharging,
  ShieldCheck,
  Flame,
  Scale,
  Leaf
} from 'lucide-vue-next'
import type { TabItem, ModeItem, ColorItem, BatteryModeItem, CustomTuningConfig } from './types'

export const tabs: TabItem[] = [
  { id: 'overview', name: '概览大厅', icon: LayoutGrid },
  { id: 'keyboard', name: 'RGB 键盘', icon: Keyboard },
  { id: 'power', name: '电池与显卡', icon: BatteryCharging },
  { id: 'system', name: '系统与补丁', icon: ShieldCheck }
]

export const modes: ModeItem[] = [
  {
    id: 0,
    name: '狂暴模式',
    english: 'TURBO',
    icon: Flame,
    color: 'red',
    desc: '满血功耗释放与激进散热'
  },
  {
    id: 1,
    name: '均衡模式',
    english: 'BALANCED',
    icon: Scale,
    color: 'cyan',
    desc: '日常游戏性能与噪音平衡'
  },
  {
    id: 2,
    name: '办公模式',
    english: 'OFFICE',
    icon: Leaf,
    color: 'green',
    desc: '低功耗限制与静音续航'
  },
  {
    id: 3,
    name: '自定义模式',
    english: 'CUSTOM',
    icon: SlidersHorizontal,
    color: 'white',
    desc: '机身白光 · 自由调节功耗与频率'
  }
]

export const colors: ColorItem[] = [
  { name: '纯白', value: [255, 255, 255], hex: '#ffffff' },
  { name: '烈红', value: [255, 0, 0], hex: '#ef4444' },
  { name: '亮绿', value: [0, 255, 0], hex: '#10b981' },
  { name: '纯蓝', value: [0, 0, 255], hex: '#3b82f6' },
  { name: '青蓝', value: [0, 255, 255], hex: '#06b6d4' },
  { name: '魅紫', value: [180, 0, 255], hex: '#a855f7' },
  { name: '暖黄', value: [255, 204, 0], hex: '#eab308' },
  { name: '明橙', value: [255, 102, 0], hex: '#f97316' }
]

export const breathingColors: ColorItem[] = [
  { name: '烈红', value: [255, 0, 0], hex: '#ef4444' },
  { name: '亮绿', value: [0, 255, 0], hex: '#10b981' },
  { name: '纯蓝', value: [0, 0, 255], hex: '#3b82f6' },
  { name: '纯白', value: [255, 255, 255], hex: '#ffffff' }
]

export const batteryModes: BatteryModeItem[] = [
  { mode: 2, percent: 60, label: '长效养护', desc: '长期插电，保护电芯寿命' },
  { mode: 1, percent: 80, label: '日常平衡', desc: '日常插电平衡，延缓电池衰减' },
  { mode: 0, percent: 100, label: '常规充满', desc: '出门移动办公前常规充满' }
]

export const defaultTuningConfig = (): CustomTuningConfig => ({
  cpu_pl1: 80,
  cpu_pl2: 115,
  cpu_pl4: 180,
  gpu_core_offset: 0,
  gpu_mem_offset: 0,
  gpu_dynamic_boost: 25,
  fan_left_duty: 80,
  fan_right_duty: 80
})
