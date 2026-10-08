export interface CustomUiThemeConfig {
  blur: number // 背景模糊度 (px)
  opacity: number // 透明度 (%) 100 = 完全不透明
  enableCustomColor: boolean
  bgColor: string
  enableCustomImage: boolean
  bgImage: string
  bgSize: 'cover' | 'contain' | 'auto'
}

export const UI_THEME_STORAGE_KEY = 'xread-custom-ui-theme'
const STYLE_TAG_ID = 'xread-custom-ui-theme-style'

export const defaultUiThemeConfig: CustomUiThemeConfig = {
  blur: 0,
  opacity: 100,
  enableCustomColor: false,
  bgColor: '#f5c4c4',
  enableCustomImage: false,
  bgImage: '',
  bgSize: 'cover',
}

export function loadCustomUiTheme(): CustomUiThemeConfig {
  try {
    const raw = localStorage.getItem(UI_THEME_STORAGE_KEY)
    if (raw) {
      return { ...defaultUiThemeConfig, ...JSON.parse(raw) }
    }
  } catch {
    // ignore malformed storage
  }
  return { ...defaultUiThemeConfig }
}

let styleTag: HTMLStyleElement | null = null

function ensureStyleTag(): HTMLStyleElement | null {
  if (typeof document === 'undefined') return null
  if (styleTag && styleTag.isConnected) return styleTag
  styleTag = document.getElementById(STYLE_TAG_ID) as HTMLStyleElement | null
  if (!styleTag) {
    styleTag = document.createElement('style')
    styleTag.id = STYLE_TAG_ID
    document.head.appendChild(styleTag)
  }
  return styleTag
}

export function saveCustomUiTheme(config: CustomUiThemeConfig) {
  try {
    localStorage.setItem(UI_THEME_STORAGE_KEY, JSON.stringify(config))
  } catch {
    // ignore quota errors
  }
  applyCustomUiTheme(config)
}

export function applyCustomUiTheme(config: CustomUiThemeConfig) {
  const tag = ensureStyleTag()
  if (!tag) return

  const rules: string[] = []
  const alpha = Math.max(0.05, Math.min(1, config.opacity / 100))

  // ── 自定义背景图片 ──
  if (config.enableCustomImage && config.bgImage) {
    rules.push(`
      body, #app, .app-main, .app-topbar, .app-bottom-nav {
        background-color: transparent !important;
      }
      body::before {
        content: "";
        position: fixed;
        inset: 0;
        z-index: 0;
        background-image: url("${config.bgImage}");
        background-size: ${config.bgSize || 'cover'};
        background-position: center;
        background-repeat: no-repeat;
        filter: blur(${config.blur}px);
        opacity: ${alpha};
        transform: scale(1.06);
        pointer-events: none;
      }
      #app { position: relative; z-index: 1; }
    `)
  } else {
    // 无背景图时，模糊度作用于毛玻璃效果
    if (config.blur > 0) {
      rules.push(`
        .drawer-content, .settings-drawer, .modal-container, .card, .book-card,
        .recent-search-input, .stat-cell {
          backdrop-filter: blur(${config.blur}px) !important;
          -webkit-backdrop-filter: blur(${config.blur}px) !important;
        }
      `)
    }
    if (config.enableCustomColor && config.bgColor) {
      rules.push(`
        :root {
          --color-bg: ${config.bgColor} !important;
        }
        body, #app, .app-main {
          background-color: ${config.bgColor} !important;
        }
        .app-topbar {
          background-color: transparent !important;
          border-bottom: none !important;
        }
      `)
    }
  }

  // ── 透明度（无背景图时作用于面板/卡片底色） ──
  if (!config.enableCustomImage && config.opacity < 100) {
    rules.push(`
      .card, .stat-cell, .book-card, .recent-search-input, .filter-chip, .ranking-chip {
        background-color: rgba(255, 255, 255, ${alpha}) !important;
      }
      [data-theme='dark'] .card,
      [data-theme='dark'] .stat-cell,
      [data-theme='dark'] .book-card,
      [data-theme='dark'] .recent-search-input,
      [data-theme='dark'] .filter-chip,
      [data-theme='dark'] .ranking-chip {
        background-color: rgba(30, 30, 30, ${alpha}) !important;
      }
    `)
  }

  tag.textContent = rules.join('\n')
}

export function resetCustomUiTheme() {
  try {
    localStorage.removeItem(UI_THEME_STORAGE_KEY)
  } catch {
    // ignore
  }
  applyCustomUiTheme({ ...defaultUiThemeConfig })
}
