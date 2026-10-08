export interface CustomUiThemeConfig {
  blur: number // 背景模糊度 (px)
  opacity: number // 透明度 (%) 100 = 完全不透明
  enableCustomColor: boolean
  bgColor: string
  enableCustomImage: boolean
  bgImage: string
  bgSize: 'cover' | 'contain' | 'auto'
  /** 是否启用自定义全局主题色（按钮/高亮/激活态） */
  enableCustomPrimary: boolean
  /** 自定义主题色（十六进制） */
  primaryColor: string
}

export const DEFAULT_PRIMARY_COLOR = '#3b82f6'

/** 界面设置里提供的预设主题色（首个为默认蓝色） */
export const PRIMARY_COLOR_PRESETS = [
  { label: '天蓝', value: '#3b82f6' },
  { label: '粉红', value: '#f43f5e' },
  { label: '紫罗兰', value: '#8b5cf6' },
  { label: '翡翠绿', value: '#10b981' },
  { label: '琥珀橙', value: '#f59e0b' },
  { label: '青碧', value: '#06b6d4' },
]

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
  enableCustomPrimary: false,
  primaryColor: DEFAULT_PRIMARY_COLOR,
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

/** 由主色推导出浅色 / 深色 / 淡底 / 边框四个派生色。 */
export function derivePrimaryPalette(hex: string) {
  const normalized = hex.trim().replace('#', '')
  if (!/^[0-9a-fA-F]{6}$/.test(normalized)) return null
  const r = parseInt(normalized.slice(0, 2), 16)
  const g = parseInt(normalized.slice(2, 4), 16)
  const b = parseInt(normalized.slice(4, 6), 16)

  const clamp = (v: number) => Math.max(0, Math.min(255, Math.round(v)))
  const lighten = (ratio: number) =>
    `rgb(${clamp(r + (255 - r) * ratio)}, ${clamp(g + (255 - g) * ratio)}, ${clamp(b + (255 - b) * ratio)})`
  const darken = (ratio: number) =>
    `rgb(${clamp(r * (1 - ratio))}, ${clamp(g * (1 - ratio))}, ${clamp(b * (1 - ratio))})`

  return {
    primary: `rgb(${r}, ${g}, ${b})`,
    dark: darken(0.16),
    light: lighten(0.24),
    bg: `rgba(${r}, ${g}, ${b}, 0.08)`,
    bgStrong: `rgba(${r}, ${g}, ${b}, 0.1)`,
    border: `rgba(${r}, ${g}, ${b}, 0.25)`,
    borderStrong: `rgba(${r}, ${g}, ${b}, 0.3)`,
  }
}

/** 供组件展示用：把任意颜色转成十六进制。 */
export function toHexColor(hex: string) {
  const normalized = hex.trim()
  return /^#[0-9a-fA-F]{6}$/.test(normalized) ? normalized : DEFAULT_PRIMARY_COLOR
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

  // ── 自定义全局主题色（按钮 / 高亮 / 选中态 / 进度条） ──
  if (config.enableCustomPrimary) {
    const palette = derivePrimaryPalette(toHexColor(config.primaryColor))
    if (palette) {
      rules.push(`
        :root {
          --color-primary: ${palette.primary} !important;
          --color-primary-light: ${palette.light} !important;
          --color-primary-dark: ${palette.dark} !important;
          --color-primary-bg: ${palette.bg} !important;
          --color-primary-border: ${palette.border} !important;
        }
        [data-theme='dark'] {
          --color-primary: ${palette.light} !important;
          --color-primary-light: ${palette.light} !important;
          --color-primary-dark: ${palette.primary} !important;
          --color-primary-bg: ${palette.bgStrong} !important;
          --color-primary-border: ${palette.borderStrong} !important;
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
