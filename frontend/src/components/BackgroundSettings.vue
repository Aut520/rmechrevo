<!-- Author: Aut520 <aut@autkc.com> | https://github.com/Aut520/rmechrevo | GPL-3.0 -->
<script setup lang="ts">
import { ref } from 'vue'
import {
  X,
  Sparkles,
  Link,
  Upload,
  Palette,
  RefreshCw,
  RotateCcw,
  Sun,
  Moon
} from 'lucide-vue-next'
import {
  useAppearance,
  DEFAULT_API_URL,
  PRESET_GRADIENTS,
  type BgMode
} from '../composables/useAppearance'

defineEmits<{
  (e: 'close'): void
}>()

const { appearance, refreshRandomWallpaper, resetAppearance, setLocalWallpaperFile } = useAppearance()
const fileInput = ref<HTMLInputElement | null>(null)
const uploadError = ref('')

const bgModes: { id: BgMode; label: string; icon: any }[] = [
  { id: 'api', label: '随机壁纸', icon: Sparkles },
  { id: 'url', label: '图片 URL', icon: Link },
  { id: 'local', label: '本地图片', icon: Upload },
  { id: 'gradient', label: '氛围渐变', icon: Palette }
]

async function onFileChange(e: Event) {
  uploadError.value = ''
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  try {
    await setLocalWallpaperFile(file)
  } catch (err: any) {
    uploadError.value = err?.message || '加载本地图片失败'
  } finally {
    input.value = ''
  }
}
</script>

<template>
  <div class="drawer-overlay" @click="$emit('close')">
    <aside class="drawer-panel" @click.stop>
      <div class="drawer-header">
        <h3>外观与壁纸</h3>
        <button class="drawer-close" title="关闭" @click="$emit('close')">
          <X :size="16" />
        </button>
      </div>

      <div class="drawer-scroll">
        <section class="drawer-section">
          <div class="section-label">背景来源</div>
          <div class="mode-grid">
            <button
              v-for="item in bgModes"
              :key="item.id"
              class="mode-btn"
              :class="{ active: appearance.bgMode === item.id }"
              @click="appearance.bgMode = item.id"
            >
              <component :is="item.icon" :size="14" />
              <span>{{ item.label }}</span>
            </button>
          </div>
        </section>

        <section v-if="appearance.bgMode === 'api'" class="drawer-section">
          <div class="section-label">
            <span>随机壁纸 API</span>
            <button class="inline-action" @click="refreshRandomWallpaper">
              <RefreshCw :size="12" />
              <span>换一张</span>
            </button>
          </div>
          <input
            v-model="appearance.apiUrl"
            type="text"
            class="glass-input"
            :placeholder="DEFAULT_API_URL"
          />
        </section>

        <section v-else-if="appearance.bgMode === 'url'" class="drawer-section">
          <div class="section-label">图片链接</div>
          <input
            v-model="appearance.customUrl"
            type="text"
            class="glass-input"
            placeholder="https://example.com/wallpaper.jpg"
          />
        </section>

        <section v-else-if="appearance.bgMode === 'local'" class="drawer-section">
          <div class="section-label">本地图片</div>
          <input
            ref="fileInput"
            type="file"
            accept="image/*"
            class="hidden-input"
            @change="onFileChange"
          />
          <button class="upload-box" @click="fileInput?.click()">
            <img
              v-if="appearance.localDataUrl"
              :src="appearance.localDataUrl"
              alt="本地壁纸"
              class="upload-preview"
            />
            <div class="upload-caption">
              <Upload :size="16" />
              <span>{{ appearance.localDataUrl ? '更换图片' : '选择图片' }}</span>
            </div>
          </button>
          <p v-if="uploadError" class="hint status-err">{{ uploadError }}</p>
        </section>

        <section v-else class="drawer-section">
          <div class="section-label">预设渐变</div>
          <div class="gradient-presets">
            <button
              v-for="g in PRESET_GRADIENTS"
              :key="g.name"
              class="gradient-chip"
              :style="{ background: `linear-gradient(135deg, ${g.start}, ${g.end})` }"
              @click="appearance.gradientStart = g.start; appearance.gradientEnd = g.end"
            >
              <span>{{ g.name }}</span>
            </button>
          </div>
          <div class="color-pair">
            <label>
              <span>起始色</span>
              <input v-model="appearance.gradientStart" type="color" />
            </label>
            <label>
              <span>结束色</span>
              <input v-model="appearance.gradientEnd" type="color" />
            </label>
          </div>
        </section>

        <section class="drawer-section">
          <div class="section-label">玻璃质感</div>
          <div class="two-col">
            <button
              class="mode-btn"
              :class="{ active: appearance.theme === 'dark' }"
              @click="appearance.theme = 'dark'"
            >
              <Moon :size="14" />
              <span>暗色烟熏</span>
            </button>
            <button
              class="mode-btn"
              :class="{ active: appearance.theme === 'light' }"
              @click="appearance.theme = 'light'"
            >
              <Sun :size="14" />
              <span>浅色磨砂</span>
            </button>
          </div>
        </section>

        <section class="drawer-section">
          <div class="section-label">主题强调色</div>
          <div class="two-col">
            <button
              class="mode-btn"
              :class="{ active: appearance.accent === 'sakura' }"
              @click="appearance.accent = 'sakura'"
            >
              <i class="swatch-dot" style="background: #f472b6" />
              <span>樱花粉</span>
            </button>
            <button
              class="mode-btn"
              :class="{ active: appearance.accent === 'cyan' }"
              @click="appearance.accent = 'cyan'"
            >
              <i class="swatch-dot" style="background: #38bdf8" />
              <span>晴空青</span>
            </button>
          </div>
        </section>

        <section class="drawer-section">
          <div class="section-label">界面字体</div>
          <div class="two-col">
            <button
              class="mode-btn"
              :class="{ active: appearance.font === 'wenkai' }"
              @click="appearance.font = 'wenkai'"
            >
              <span>霞鹜文楷</span>
            </button>
            <button
              class="mode-btn"
              :class="{ active: appearance.font === 'sans' }"
              @click="appearance.font = 'sans'"
            >
              <span>系统无衬线</span>
            </button>
          </div>
        </section>

        <section class="drawer-section">
          <div class="slider-head">
            <span>毛玻璃模糊度</span>
            <strong class="tabular">{{ appearance.blur }}px</strong>
          </div>
          <input v-model.number="appearance.blur" type="range" min="0" max="24" class="glass-range" />

          <div class="slider-head">
            <span>卡片不透明度</span>
            <strong class="tabular">{{ appearance.cardOpacity }}%</strong>
          </div>
          <input v-model.number="appearance.cardOpacity" type="range" min="15" max="65" class="glass-range" />

          <div class="slider-head">
            <span>壁纸压暗遮罩</span>
            <strong class="tabular">{{ appearance.overlayOpacity }}%</strong>
          </div>
          <input v-model.number="appearance.overlayOpacity" type="range" min="0" max="60" class="glass-range" />
        </section>
      </div>

      <div class="drawer-footer">
        <button class="btn btn-secondary" @click="resetAppearance">
          <RotateCcw :size="13" />
          <span>恢复默认</span>
        </button>
        <button class="btn btn-primary" @click="$emit('close')">完成</button>
      </div>
    </aside>
  </div>
</template>

<style scoped>
.drawer-overlay {
  position: fixed;
  inset: 0;
  z-index: 200;
  background: rgba(0, 0, 0, 0.28);
  backdrop-filter: blur(3px);
  -webkit-backdrop-filter: blur(3px);
  display: flex;
  justify-content: flex-end;
  animation: fade-in 0.18s ease;
}

.drawer-panel {
  width: 356px;
  max-width: 92vw;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--glass-drawer-bg);
  backdrop-filter: blur(16px) saturate(145%);
  -webkit-backdrop-filter: blur(16px) saturate(145%);
  border-left: 1px solid var(--glass-border);
  box-shadow: -8px 0 28px rgba(0, 0, 0, 0.28);
  animation: slide-in 0.22s cubic-bezier(0.22, 1, 0.36, 1);
}

@keyframes slide-in {
  from {
    transform: translateX(28px);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}

.drawer-header {
  padding: 18px 20px 14px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--glass-divider);
}

.drawer-header h3 {
  font-size: 15px;
  font-weight: 700;
}

.drawer-header p {
  margin-top: 2px;
  font-size: 11.5px;
  color: var(--text-secondary);
}

.drawer-close {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  display: grid;
  place-items: center;
  background: var(--pill-bg);
  color: var(--text-secondary);
}

.drawer-close:hover {
  background: var(--pill-hover);
  color: var(--text-primary);
}

.drawer-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.drawer-section {
  display: flex;
  flex-direction: column;
  gap: 9px;
}

.section-label {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  font-weight: 700;
  color: var(--text-secondary);
}

.inline-action {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 9px;
  border-radius: var(--radius-pill);
  background: var(--accent-soft);
  border: 1px solid var(--accent-border);
  color: var(--text-primary);
  font-size: 11px;
}

.inline-action:hover {
  filter: brightness(1.1);
}

.mode-grid,
.two-col {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.mode-btn {
  height: 36px;
  padding: 0 10px;
  border-radius: var(--radius-sm);
  background: var(--glass-sub-bg);
  border: 1px solid var(--glass-border);
  color: var(--text-secondary);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  font-size: 12px;
}

.mode-btn:hover {
  background: var(--pill-hover);
  color: var(--text-primary);
}

.mode-btn.active {
  background: var(--accent-soft);
  border-color: var(--accent-border);
  color: var(--text-primary);
}

.swatch-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.hint {
  font-size: 11px;
  line-height: 1.5;
  color: var(--text-muted);
}

.hidden-input {
  display: none;
}

.upload-box {
  position: relative;
  width: 100%;
  height: 110px;
  border-radius: var(--radius-md);
  border: 1px dashed var(--glass-border-hover);
  background: var(--glass-sub-bg);
  overflow: hidden;
  display: grid;
  place-items: center;
}

.upload-box:hover {
  border-color: var(--accent-border);
}

.upload-preview {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  opacity: 0.6;
}

.upload-caption {
  position: relative;
  z-index: 1;
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 6px 12px;
  border-radius: var(--radius-pill);
  background: rgba(0, 0, 0, 0.45);
  color: #ffffff;
  font-size: 12px;
}

.gradient-presets {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 7px;
}

.gradient-chip {
  height: 36px;
  border-radius: var(--radius-sm);
  border: 1px solid rgba(255, 255, 255, 0.16);
  color: #ffffff;
  font-size: 11px;
  font-weight: 600;
}

.color-pair {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.color-pair label {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
  border-radius: var(--radius-sm);
  background: var(--glass-sub-bg);
  border: 1px solid var(--glass-border);
  font-size: 11.5px;
  color: var(--text-secondary);
}

.color-pair input[type="color"] {
  width: 28px;
  height: 22px;
  border: none;
  background: transparent;
  cursor: pointer;
}

.slider-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 4px;
}

.slider-head strong {
  color: var(--accent);
  font-size: 12px;
}

.drawer-footer {
  padding: 14px 20px;
  border-top: 1px solid var(--glass-divider);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
</style>
