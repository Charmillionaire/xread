<template>
  <div class="reader-artplayer-stage">
    <div
      v-if="props.title"
      class="reader-video-title"
      :class="{ 'is-hidden': titleHidden }"
    >
      <span class="reader-video-title-text">{{ props.title }}</span>
    </div>
    <div class="reader-artplayer-wrapper" :style="wrapperStyle">
      <div class="reader-artplayer-container" ref="artContainerRef"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch, nextTick } from 'vue'
import Artplayer from 'artplayer'
import Hls from 'hls.js'

const props = withDefaults(
  defineProps<{
    url: string
    title?: string
    isNight?: boolean
    hasPrev?: boolean
    hasNext?: boolean
  }>(),
  {
    title: '',
    isNight: false,
    hasPrev: false,
    hasNext: false,
  },
)

const emit = defineEmits<{
  (e: 'prev'): void
  (e: 'next'): void
  (e: 'ended'): void
  (e: 'timeupdate', current: number, duration: number): void
}>()

const artContainerRef = ref<HTMLDivElement | null>(null)
let art: Artplayer | null = null
let hlsInstance: Hls | null = null
let resizeObserver: ResizeObserver | null = null

// 视频真实宽高比（宽 / 高），未加载完成时为 null
const videoRatio = ref<number | null>(null)
// 精确像素尺寸，保证容器与视频比例完全一致，彻底消除黑边
const boxWidth = ref<number | null>(null)
const boxHeight = ref<number | null>(null)

const titleHidden = ref(false)

const wrapperStyle = computed(() => {
  if (boxWidth.value && boxHeight.value) {
    return {
      width: `${boxWidth.value}px`,
      height: `${boxHeight.value}px`,
    }
  }
  // 视频元数据加载前，先给一个中性占位，避免闪烁与黑边
  return {
    width: '100%',
    height: '56vh',
  }
})

function destroyHls() {
  if (hlsInstance) {
    try {
      hlsInstance.destroy()
    } catch {}
    hlsInstance = null
  }
}

/**
 * 依据视频真实分辨率与可用视口，计算与视频比例完全一致的精确像素尺寸。
 * 容器比例 === 视频比例，因此不会出现任何左右 / 上下黑边。
 */
function fitToVideo() {
  if (!art || !art.video) return
  const vw = art.video.videoWidth || 0
  const vh = art.video.videoHeight || 0
  if (!vw || !vh) return

  const ratio = vw / vh
  videoRatio.value = ratio

  const holder = artContainerRef.value?.parentElement?.parentElement
  const availW = holder && holder.clientWidth > 0 ? holder.clientWidth : window.innerWidth
  const isMobile = window.innerWidth <= 768
  const availH = window.innerHeight * (isMobile ? 0.62 : 0.74)

  let w = availW
  let h = w / ratio
  if (h > availH) {
    h = availH
    w = h * ratio
  }

  boxWidth.value = Math.max(1, Math.floor(w))
  boxHeight.value = Math.max(1, Math.floor(h))
}

function onWindowResize() {
  fitToVideo()
}

function initPlayer() {
  if (!artContainerRef.value || !props.url) return

  if (art) {
    art.destroy(false)
    art = null
  }
  destroyHls()
  videoRatio.value = null
  boxWidth.value = null
  boxHeight.value = null

  const isM3u8 = /\.m3u8(?:\?|$)/i.test(props.url)

  art = new Artplayer({
    container: artContainerRef.value,
    url: props.url,
    volume: 0.8,
    isLive: false,
    muted: false,
    autoplay: true,
    pip: true,
    autoSize: false,
    autoMini: false,
    screenshot: true,
    setting: true,
    loop: false,
    flip: true,
    playbackRate: true,
    aspectRatio: true,
    fullscreen: true,
    fullscreenWeb: true,
    miniProgressBar: true,
    mutex: true,
    backdrop: true,
    playsInline: true,
    gesture: true,
    fastForward: true,
    autoOrientation: true,
    autoPlayback: true,
    theme: '#3b82f6',
    lang: 'zh-cn',
    controls: [
      {
        name: 'prev-chapter',
        index: 11,
        position: 'left',
        html: `<button class="art-custom-btn" style="background:none;border:none;color:#fff;cursor:${props.hasPrev ? 'pointer' : 'not-allowed'};opacity:${props.hasPrev ? '1' : '0.4'};display:flex;align-items:center;padding:0 6px;" title="上一集">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor"><path d="M6 6h2v12H6zm3.5 6l8.5 6V6z"/></svg>
        </button>`,
        click: () => {
          if (props.hasPrev) emit('prev')
        },
      },
      {
        name: 'next-chapter',
        index: 12,
        position: 'left',
        html: `<button class="art-custom-btn" style="background:none;border:none;color:#fff;cursor:${props.hasNext ? 'pointer' : 'not-allowed'};opacity:${props.hasNext ? '1' : '0.4'};display:flex;align-items:center;padding:0 6px;" title="下一集">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor"><path d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z"/></svg>
        </button>`,
        click: () => {
          if (props.hasNext) emit('next')
        },
      },
    ],
    customType: {
      m3u8: function (video: HTMLVideoElement, url: string, artInstance: Artplayer) {
        if (Hls.isSupported()) {
          destroyHls()
          const hls = new Hls({
            enableWorker: true,
            lowLatencyMode: false,
          })
          hlsInstance = hls
          hls.loadSource(url)
          hls.attachMedia(video)
          hls.on(Hls.Events.ERROR, (_event, data) => {
            if (data.fatal) {
              switch (data.type) {
                case Hls.ErrorTypes.NETWORK_ERROR:
                  hls.startLoad()
                  break
                case Hls.ErrorTypes.MEDIA_ERROR:
                  hls.recoverMediaError()
                  break
                default:
                  artInstance.notice.show = '播放器流媒体加载失败'
                  break
              }
            }
          })
        } else if (video.canPlayType('application/vnd.apple.mpegurl')) {
          video.src = url
        } else {
          artInstance.notice.show = '当前浏览器不支持播放该 m3u8 视频'
        }
      },
    },
  })

  if (isM3u8) {
    art.type = 'm3u8'
  }

  // 元数据就绪后按真实分辨率自适应，消除黑边
  art.on('video:loadedmetadata', () => {
    fitToVideo()
  })
  art.on('video:canplay', () => {
    fitToVideo()
  })

  // 控制层显示/隐藏时，同步顶部标题的显隐，保持沉浸感
  art.on('control', (state: boolean) => {
    titleHidden.value = !state
  })

  art.on('video:play', () => {
    titleHidden.value = true
  })
  art.on('video:pause', () => {
    titleHidden.value = false
  })

  art.on('video:timeupdate', () => {
    if (art && art.video) {
      emit('timeupdate', art.video.currentTime || 0, art.video.duration || 0)
    }
  })

  art.on('video:ended', () => {
    emit('ended')
  })

  nextTick(() => {
    fitToVideo()
  })
}

watch(
  () => props.url,
  (newUrl) => {
    if (!newUrl) return
    nextTick(() => {
      titleHidden.value = false
      initPlayer()
    })
  },
)

onMounted(() => {
  initPlayer()
  window.addEventListener('resize', onWindowResize)
  const holder = artContainerRef.value?.parentElement?.parentElement
  if (holder && typeof ResizeObserver !== 'undefined') {
    resizeObserver = new ResizeObserver(() => {
      fitToVideo()
    })
    resizeObserver.observe(holder)
  }
})

onBeforeUnmount(() => {
  destroyHls()
  window.removeEventListener('resize', onWindowResize)
  if (resizeObserver) {
    resizeObserver.disconnect()
    resizeObserver = null
  }
  if (art) {
    art.destroy(false)
    art = null
  }
})
</script>

<style scoped>
/* 无边框、无阴影、无背景的沉浸舞台 */
.reader-artplayer-stage {
  position: relative;
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  background: transparent;
}

/* 顶部视频标题：悬浮在画面上方，控制层隐藏时同步淡出 */
.reader-video-title {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  z-index: 20;
  padding: 14px 16px 26px;
  display: flex;
  align-items: center;
  pointer-events: none;
  background: linear-gradient(180deg, rgba(0, 0, 0, 0.62) 0%, rgba(0, 0, 0, 0) 100%);
  transition: opacity 0.35s ease;
}

.reader-video-title.is-hidden {
  opacity: 0;
}

.reader-video-title-text {
  color: #fff;
  font-size: 15px;
  font-weight: 600;
  line-height: 1.4;
  letter-spacing: 0.2px;
  text-shadow: 0 1px 6px rgba(0, 0, 0, 0.55);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.reader-artplayer-wrapper {
  margin: 0 auto;
  max-width: 100%;
  background: transparent;
  overflow: hidden;
  transition: width 0.2s ease, height 0.2s ease;
}

.reader-artplayer-container {
  width: 100%;
  height: 100%;
  border: none;
  border-radius: 0;
  box-shadow: none;
  background: transparent;
}

.reader-artplayer-container :deep(.art-video-player) {
  border-radius: 0;
  background: transparent;
}

.reader-artplayer-container :deep(video) {
  object-fit: contain;
  background: transparent;
}

@media (max-width: 768px) {
  .reader-video-title {
    padding: 10px 12px 22px;
  }
  .reader-video-title-text {
    font-size: 14px;
  }
}
</style>
