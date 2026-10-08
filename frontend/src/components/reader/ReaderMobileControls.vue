<template>
  <div
    class="mobile-controls"
    :style="{ '--popup-bg': theme.popup, '--font-color': theme.fontColor }"
    @click.stop
    @touchstart.stop
    @touchmove.stop
    @touchend.stop
  >
    <!-- 亮度遮罩层：压暗整个阅读界面 -->
    <Teleport to="body">
      <div
        v-if="brightness < 100"
        class="reader-brightness-mask"
        :style="{ background: `rgba(0, 0, 0, ${(100 - brightness) / 100 * 0.72})` }"
      ></div>
    </Teleport>

    <!-- 居中悬浮控制面板 -->
    <Transition name="panel-pop">
      <div v-show="show" class="m-panel" :class="{ 'is-dark': isDark }">
        <!-- 第一栏：章节进度调节 -->
        <div class="m-row m-row-progress">
          <button
            class="m-icon-btn"
            :class="{ disabled: !store.hasPrev }"
            title="上一章"
            aria-label="上一章"
            @click="$emit('prev')"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M19 20 9 12l10-8v16z" />
              <path d="M5 19V5" />
            </svg>
          </button>

          <div
            class="m-track"
            @mousedown="startProgressDrag"
            @touchstart.prevent="startProgressDrag"
          >
            <div class="m-track-bg">
              <div class="m-track-fill" :style="{ width: progressPercent + '%' }"></div>
              <div class="m-track-thumb" :style="{ left: progressPercent + '%' }"></div>
            </div>
          </div>

          <button
            class="m-icon-btn"
            :class="{ disabled: !store.hasNext }"
            title="下一章"
            aria-label="下一章"
            @click="$emit('next')"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M5 4l10 8-10 8V4z" />
              <path d="M19 5v14" />
            </svg>
          </button>
        </div>

        <!-- 第二栏：明亮度调节 -->
        <div class="m-row m-row-brightness">
          <button
            class="m-letter-btn"
            :class="{ active: autoBrightness }"
            title="跟随系统亮度"
            aria-label="跟随系统亮度"
            @click="toggleAutoBrightness"
          >
            A
          </button>

          <button class="m-icon-btn small" title="调暗" aria-label="调暗" @click="stepBrightness(-10)">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <path d="M5 12h14" />
            </svg>
          </button>

          <div
            class="m-track"
            @mousedown="startBrightnessDrag"
            @touchstart.prevent="startBrightnessDrag"
          >
            <div class="m-track-bg">
              <div class="m-track-fill" :style="{ width: brightness + '%' }"></div>
              <div class="m-track-thumb" :style="{ left: brightness + '%' }"></div>
            </div>
          </div>

          <button class="m-icon-btn small" title="调亮" aria-label="调亮" @click="stepBrightness(10)">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <path d="M12 5v14M5 12h14" />
            </svg>
          </button>
        </div>

        <!-- 功能网格：4 × 2 -->
        <div class="m-grid">
          <button class="m-grid-item" @click="$emit('search')">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="11" cy="11" r="7" />
              <path d="m20 20-3.5-3.5" />
            </svg>
            <span>搜索</span>
          </button>

          <button class="m-grid-item" :class="{ active: store.isAutoScrolling }" @click="store.toggleAutoReading()">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="4" width="18" height="13" rx="2" />
              <path d="M8 20h8M12 17v3" />
              <path d="m10 8.5 4 2.5-4 2.5v-5z" />
            </svg>
            <span>自动</span>
          </button>

          <button class="m-grid-item" :class="{ active: store.activePanel === 'rule' }" @click="store.togglePanel('rule')">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
              <path d="M4 8h9a4 4 0 1 0 0-4" />
              <path d="M20 16h-9a4 4 0 1 0 0 4" />
            </svg>
            <span>替换</span>
          </button>

          <button class="m-grid-item" :class="{ active: store.isNight }" @click="store.toggleNight()">
            <svg v-if="!store.isNight" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 12.8A9 9 0 1 1 11.2 3 7 7 0 0 0 21 12.8z" />
            </svg>
            <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="4" />
              <path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M6.3 17.7l-1.4 1.4M19.1 4.9l-1.4 1.4" />
            </svg>
            <span>夜间</span>
          </button>

          <button class="m-grid-item" :class="{ active: store.activePanel === 'catalog' }" @click="store.togglePanel('catalog')">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
              <path d="M4 5h16M4 10h16M4 15h16M4 20h9" />
            </svg>
            <span>目录</span>
          </button>

          <button class="m-grid-item" :class="{ active: isSpeaking }" @click="$emit('tts')">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
              <path d="M4 14v-2a8 8 0 0 1 16 0v2" />
              <path d="M18 16a2 2 0 0 1-2 2h-1a1 1 0 0 1-1-1v-3a1 1 0 0 1 1-1h3z" />
              <path d="M6 16a2 2 0 0 0 2 2h1a1 1 0 0 0 1-1v-3a1 1 0 0 0-1-1H6z" />
            </svg>
            <span>朗读</span>
          </button>

          <button class="m-grid-item" :class="{ active: store.activePanel === 'settings' }" @click="store.togglePanel('settings')">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
              <path d="M5 20V10M12 20V4M19 20v-7" />
            </svg>
            <span>界面</span>
          </button>

          <button class="m-grid-item" @click="openSettingsDrawer">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="3" />
              <path d="M19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.9-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1A1.7 1.7 0 0 0 9 19.4a1.7 1.7 0 0 0-1.9.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.9 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1A1.7 1.7 0 0 0 4.6 9a1.7 1.7 0 0 0-.3-1.9l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.9.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.9-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.9V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z" />
            </svg>
            <span>设置</span>
          </button>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onBeforeUnmount, ref } from 'vue'
import { useReaderStore } from '../../stores/reader'
import { useAppStore } from '../../stores/app'

const props = defineProps<{
  show: boolean
  isSpeaking?: boolean
  isPaused?: boolean
}>()

const emit = defineEmits<{
  goHome: []
  scrollTop: []
  scrollBottom: []
  prev: []
  next: []
  bookmark: []
  search: []
  info: []
  ai: []
  tts: []
  progress: []
}>()

const store = useReaderStore()
const appStore = useAppStore()

const BRIGHTNESS_KEY = 'reader-brightness'
const AUTO_BRIGHTNESS_KEY = 'reader-auto-brightness'

const brightness = ref(Number(localStorage.getItem(BRIGHTNESS_KEY) ?? 100))
const autoBrightness = ref(localStorage.getItem(AUTO_BRIGHTNESS_KEY) === 'true')

const isDark = computed(() => store.isNight || appStore.theme === 'dark')

const theme = computed(() => {
  if (store.isNight || appStore.theme === 'dark') {
    return {
      ...store.currentTheme,
      popup: 'var(--color-bg-elevated)',
      fontColor: 'var(--color-text)',
    }
  }
  return store.currentTheme
})

const progressPercent = computed(() => {
  const raw = parseFloat(String(store.readingProgress).replace('%', ''))
  if (Number.isNaN(raw)) return 0
  return Math.max(0, Math.min(100, raw))
})

function persistBrightness() {
  localStorage.setItem(BRIGHTNESS_KEY, String(brightness.value))
}

function setBrightness(value: number) {
  brightness.value = Math.max(10, Math.min(100, Math.round(value)))
  autoBrightness.value = false
  localStorage.setItem(AUTO_BRIGHTNESS_KEY, 'false')
  persistBrightness()
}

function stepBrightness(delta: number) {
  setBrightness(brightness.value + delta)
}

function toggleAutoBrightness() {
  autoBrightness.value = !autoBrightness.value
  localStorage.setItem(AUTO_BRIGHTNESS_KEY, String(autoBrightness.value))
  if (autoBrightness.value) {
    brightness.value = 100
    persistBrightness()
  }
}

/** 章节进度拖动 */
function startProgressDrag(e: MouseEvent | TouchEvent) {
  const track = (e.currentTarget as HTMLElement).querySelector('.m-track-bg') as HTMLElement | null
  if (!track) return
  const rect = track.getBoundingClientRect()
  const total = store.chapters.length || 1

  const apply = (clientX: number) => {
    const ratio = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width))
    const target = Math.min(total - 1, Math.max(0, Math.round(ratio * (total - 1))))
    if (target !== store.currentIndex) {
      void store.loadChapter(target)
    }
  }

  apply('touches' in e ? e.touches[0].clientX : e.clientX)

  const onMove = (ev: MouseEvent | TouchEvent) => {
    apply('touches' in ev ? ev.touches[0].clientX : ev.clientX)
  }
  const onUp = () => {
    document.removeEventListener('mousemove', onMove)
    document.removeEventListener('touchmove', onMove)
    document.removeEventListener('mouseup', onUp)
    document.removeEventListener('touchend', onUp)
  }
  document.addEventListener('mousemove', onMove)
  document.addEventListener('touchmove', onMove)
  document.addEventListener('mouseup', onUp)
  document.addEventListener('touchend', onUp)
}

/** 明亮度拖动 */
function startBrightnessDrag(e: MouseEvent | TouchEvent) {
  const track = (e.currentTarget as HTMLElement).querySelector('.m-track-bg') as HTMLElement | null
  if (!track) return
  const rect = track.getBoundingClientRect()

  const apply = (clientX: number) => {
    const ratio = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width))
    setBrightness(ratio * 100)
  }

  apply('touches' in e ? e.touches[0].clientX : e.clientX)

  const onMove = (ev: MouseEvent | TouchEvent) => {
    apply('touches' in ev ? ev.touches[0].clientX : ev.clientX)
  }
  const onUp = () => {
    document.removeEventListener('mousemove', onMove)
    document.removeEventListener('touchmove', onMove)
    document.removeEventListener('mouseup', onUp)
    document.removeEventListener('touchend', onUp)
  }
  document.addEventListener('mousemove', onMove)
  document.addEventListener('touchmove', onMove)
  document.addEventListener('mouseup', onUp)
  document.addEventListener('touchend', onUp)
}

function openSettingsDrawer() {
  appStore.showSettingsDrawer = true
}

let mediaQuery: MediaQueryList | null = null

function syncAutoBrightness() {
  if (autoBrightness.value) {
    brightness.value = 100
    persistBrightness()
  }
}

onMounted(() => {
  mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
  mediaQuery.addEventListener?.('change', syncAutoBrightness)
  syncAutoBrightness()
})

onBeforeUnmount(() => {
  mediaQuery?.removeEventListener?.('change', syncAutoBrightness)
  mediaQuery = null
})

/* 保持 props 被引用，避免类型检查告警 */
void props
void emit
</script>

<style scoped>
.mobile-controls {
  position: absolute;
  inset: 0;
  z-index: 30;
  pointer-events: none;
}

/* 屏幕垂直居中悬浮控制面板（不透明实底，保证清晰可读） */
.m-panel {
  position: absolute;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
  width: min(560px, calc(100vw - 32px));
  padding: 14px 14px 10px;
  border-radius: 24px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  box-sizing: border-box;
  pointer-events: auto;
  background: #ffffff;
  border: 1px solid rgba(0, 0, 0, 0.08);
  box-shadow:
    0 24px 56px rgba(0, 0, 0, 0.28),
    0 6px 18px rgba(0, 0, 0, 0.12);
  color: #1a1a1a;
}

.m-panel.is-dark {
  background: #1e1e24;
  border-color: rgba(255, 255, 255, 0.14);
  box-shadow:
    0 26px 60px rgba(0, 0, 0, 0.62),
    0 6px 20px rgba(0, 0, 0, 0.4);
  color: #f5f5f5;
}

/* 亮度遮罩 */
.reader-brightness-mask {
  position: fixed;
  inset: 0;
  z-index: 25;
  pointer-events: none;
  transition: background 0.15s ease;
}

.m-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.m-icon-btn {
  flex: 0 0 auto;
  width: 34px;
  height: 34px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: inherit;
  border-radius: 50%;
  cursor: pointer;
  opacity: 0.85;
  transition: opacity 0.15s ease, background 0.15s ease;
}

.m-icon-btn svg {
  width: 19px;
  height: 19px;
}

.m-icon-btn.small {
  width: 26px;
  height: 26px;
}

.m-icon-btn.small svg {
  width: 15px;
  height: 15px;
}

.m-icon-btn:hover:not(.disabled) {
  opacity: 1;
  background: rgba(128, 128, 128, 0.12);
}

.m-icon-btn.disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

/* A 自适应按钮：中性单色，无彩色 */
.m-letter-btn {
  flex: 0 0 auto;
  width: 34px;
  height: 34px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  border: 1px solid rgba(128, 128, 128, 0.35);
  background: transparent;
  color: inherit;
  font-size: 14px;
  font-weight: 700;
  cursor: pointer;
  opacity: 0.85;
  transition: all 0.15s ease;
}

.m-letter-btn.active {
  opacity: 1;
  border-color: rgba(128, 128, 128, 0.75);
  background: rgba(128, 128, 128, 0.18);
}

/* 通用滑轨 */
.m-track {
  flex: 1 1 auto;
  min-width: 0;
  height: 30px;
  display: flex;
  align-items: center;
  cursor: pointer;
  touch-action: none;
}

.m-track-bg {
  position: relative;
  width: 100%;
  height: 6px;
  border-radius: 999px;
  background: rgba(128, 128, 128, 0.28);
}

.m-track-fill {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  border-radius: 999px;
  background: currentColor;
  opacity: 0.75;
}

.m-track-thumb {
  position: absolute;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.28);
}

/* 功能网格 */
.m-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 4px 0;
  padding-top: 2px;
}

.m-grid-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 5px;
  padding: 8px 0;
  border: none;
  background: transparent;
  color: inherit;
  cursor: pointer;
  border-radius: 12px;
  opacity: 0.82;
  transition: opacity 0.15s ease, background 0.15s ease;
}

.m-grid-item svg {
  width: 21px;
  height: 21px;
}

.m-grid-item span {
  font-size: 11.5px;
  line-height: 1;
  white-space: nowrap;
}

.m-grid-item:hover,
.m-grid-item.active {
  opacity: 1;
  background: rgba(128, 128, 128, 0.12);
}

/* 弹出动画 */
.panel-pop-enter-active,
.panel-pop-leave-active {
  transition: opacity 0.22s ease, transform 0.22s ease;
}

.panel-pop-enter-from,
.panel-pop-leave-to {
  opacity: 0;
  transform: translate(-50%, -50%) scale(0.94);
}

@media (max-width: 380px) {
  .m-panel {
    padding: 12px 10px 8px;
    border-radius: 20px;
  }
  .m-grid-item span {
    font-size: 10.5px;
  }
}
</style>
