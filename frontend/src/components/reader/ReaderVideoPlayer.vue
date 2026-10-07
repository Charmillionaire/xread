<template>
  <div class="reader-artplayer-container" ref="artContainerRef"></div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch, nextTick } from 'vue'
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

function destroyHls() {
  if (hlsInstance) {
    try {
      hlsInstance.destroy()
    } catch {}
    hlsInstance = null
  }
}

function initPlayer() {
  if (!artContainerRef.value || !props.url) return

  if (art) {
    art.destroy(false)
    art = null
  }
  destroyHls()

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

  // 如果链接包含 m3u8 且使用 customType 处理
  if (isM3u8) {
    art.type = 'm3u8'
  }

  art.on('video:timeupdate', () => {
    if (art && art.video) {
      emit('timeupdate', art.video.currentTime || 0, art.video.duration || 0)
    }
  })

  art.on('video:ended', () => {
    emit('ended')
  })
}

watch(
  () => props.url,
  (newUrl) => {
    if (!newUrl) return
    nextTick(() => {
      initPlayer()
    })
  },
)

onMounted(() => {
  initPlayer()
})

onBeforeUnmount(() => {
  destroyHls()
  if (art) {
    art.destroy(false)
    art = null
  }
})
</script>

<style scoped>
.reader-artplayer-container {
  width: 100%;
  height: 100%;
  min-height: 400px;
  max-height: 80vh;
  aspect-ratio: 16 / 9;
  border-radius: var(--radius-lg, 12px);
  overflow: hidden;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.25);
  background: #000;
  margin: 0 auto;
}

@media (max-width: 768px) {
  .reader-artplayer-container {
    min-height: 280px;
    max-height: 70vh;
    border-radius: 8px;
  }
}
</style>
