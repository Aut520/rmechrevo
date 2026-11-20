// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

import { reactive, watch } from 'vue'

export type BgMode = 'api' | 'url' | 'local' | 'gradient'
export type GlassTheme = 'dark' | 'light'
export type AccentTheme = 'sakura' | 'cyan'
export type FontTheme = 'wenkai' | 'sans'

export interface AppearanceConfig {
  bgMode: BgMode
  apiUrl: string
  customUrl: string
  localDataUrl: string
  gradientStart: string
  gradientEnd: string
  theme: GlassTheme
  accent: AccentTheme
  font: FontTheme
  blur: number
  cardOpacity: number
  overlayOpacity: number
}

export const DEFAULT_API_URL = 'https://t.alcy.cc/moez'

export const PRESET_GRADIENTS = [
  { name: '夜樱暮色', start: '#2b1b3d', end: '#0f172a' },
  { name: '深海青岚', start: '#0e3a4d', end: '#0b1120' },
  { name: '雾紫星尘', start: '#3b1d5e', end: '#13182b' },
  { name: '霞光暖暮', start: '#4c1d3d', end: '#181828' },
  { name: '极简深空', start: '#1e293b', end: '#090d16' }
]

const STORAGE_KEY = 'rmechrevo_appearance_v2'

const defaultConfig = (): AppearanceConfig => ({
  bgMode: 'api',
  apiUrl: DEFAULT_API_URL,
  customUrl: '',
  localDataUrl: '',
  gradientStart: '#2b1b3d',
  gradientEnd: '#0f172a',
  theme: 'dark',
  accent: 'sakura',
  font: 'wenkai',
  blur: 10,
  cardOpacity: 34,
  overlayOpacity: 16
})

const state = reactive<AppearanceConfig>(defaultConfig())
let apiSeed = Date.now()
let initialized = false

function buildApiImageUrl(base: string): string {
  const trimmed = (base || DEFAULT_API_URL).trim()
  const sep = trimmed.includes('?') ? '&' : '?'
  return `${trimmed}${sep}_t=${apiSeed}`
}

export function applyAppearance() {
  const root = document.documentElement
  root.dataset.theme = state.theme
  root.dataset.accent = state.accent
  root.dataset.font = state.font

  let bgValue = `linear-gradient(135deg, ${state.gradientStart}, ${state.gradientEnd})`
  if (state.bgMode === 'api') {
    bgValue = `url("${buildApiImageUrl(state.apiUrl)}")`
  } else if (state.bgMode === 'url' && state.customUrl.trim()) {
    bgValue = `url("${state.customUrl.trim()}")`
  } else if (state.bgMode === 'local' && state.localDataUrl) {
    bgValue = `url("${state.localDataUrl}")`
  }

  root.style.setProperty('--bg-image', bgValue)
  root.style.setProperty('--bg-fallback', `linear-gradient(135deg, ${state.gradientStart}, ${state.gradientEnd})`)
  root.style.setProperty('--glass-blur', `${state.blur}px`)
  root.style.setProperty('--glass-alpha', (state.cardOpacity / 100).toFixed(2))
  root.style.setProperty('--bg-overlay-alpha', (state.overlayOpacity / 100).toFixed(2))
}

function saveAppearance() {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state))
  } catch {
    // 本地图片超出配额时静默跳过持久化
  }
}

export function initAppearance() {
  if (initialized) return
  initialized = true
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) {
      Object.assign(state, defaultConfig(), JSON.parse(raw))
    }
  } catch {
    Object.assign(state, defaultConfig())
  }
  applyAppearance()
  watch(state, () => {
    applyAppearance()
    saveAppearance()
  }, { deep: true })
}

export function refreshRandomWallpaper() {
  apiSeed = Date.now()
  if (state.bgMode !== 'api') {
    state.bgMode = 'api'
  } else {
    applyAppearance()
  }
}

export function resetAppearance() {
  apiSeed = Date.now()
  Object.assign(state, defaultConfig())
}

export async function setLocalWallpaperFile(file: File): Promise<void> {
  const dataUrl = await compressImageToDataUrl(file, 1920, 0.86)
  state.localDataUrl = dataUrl
  state.bgMode = 'local'
}

function compressImageToDataUrl(file: File, maxWidth: number, quality: number): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onerror = () => reject(new Error('读取本地图片失败'))
    reader.onload = () => {
      const img = new Image()
      img.onerror = () => reject(new Error('图片格式不支持'))
      img.onload = () => {
        const scale = img.width > maxWidth ? maxWidth / img.width : 1
        const w = Math.round(img.width * scale)
        const h = Math.round(img.height * scale)
        const canvas = document.createElement('canvas')
        canvas.width = w
        canvas.height = h
        const ctx = canvas.getContext('2d')
        if (!ctx) {
          resolve(String(reader.result))
          return
        }
        ctx.drawImage(img, 0, 0, w, h)
        resolve(canvas.toDataURL('image/jpeg', quality))
      }
      img.src = String(reader.result)
    }
    reader.readAsDataURL(file)
  })
}

export function useAppearance() {
  initAppearance()
  return {
    appearance: state,
    refreshRandomWallpaper,
    resetAppearance,
    setLocalWallpaperFile
  }
}
