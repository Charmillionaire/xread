<template>
  <div class="ui-custom-card">
    <div class="ui-custom-head">
      <span class="ui-custom-title">前端设置</span>
      <button class="ui-reset-btn" @click="handleReset">恢复默认</button>
    </div>

    <div class="ui-row">
      <label class="ui-row-label" for="ui-blur">背景模糊度</label>
      <input
        id="ui-blur"
        v-model.number="config.blur"
        class="ui-range"
        type="range"
        min="0"
        max="1000"
        step="1"
        @input="handleChange"
      />
      <span class="ui-range-value">{{ config.blur }}</span>
    </div>

    <div class="ui-row">
      <label class="ui-row-label" for="ui-opacity">界面透明度</label>
      <input
        id="ui-opacity"
        v-model.number="config.opacity"
        class="ui-range"
        type="range"
        min="10"
        max="100"
        step="1"
        @input="handleChange"
      />
      <span class="ui-range-value">{{ config.opacity }}%</span>
    </div>

    <div class="ui-row">
      <span class="ui-row-label">自定义背景颜色</span>
      <button
        class="ui-switch"
        type="button"
        role="switch"
        :aria-checked="config.enableCustomColor"
        :class="{ on: config.enableCustomColor }"
        @click="toggleColor"
      >
        <i />
      </button>
    </div>

    <div v-if="config.enableCustomColor" class="ui-row">
      <span class="ui-row-label">背景颜色</span>
      <label class="ui-color-picker">
        <input v-model="config.bgColor" type="color" @input="handleChange" />
        <span class="ui-color-dot" :style="{ background: config.bgColor }" />
      </label>
    </div>

    <div class="ui-row">
      <span class="ui-row-label">自定义主题色</span>
      <button
        class="ui-switch"
        type="button"
        role="switch"
        :aria-checked="config.enableCustomPrimary"
        :class="{ on: config.enableCustomPrimary }"
        @click="togglePrimary"
      >
        <i />
      </button>
    </div>

    <div v-if="config.enableCustomPrimary" class="ui-primary-block">
      <div class="ui-row">
        <span class="ui-row-label">主题颜色</span>
        <label class="ui-color-picker">
          <input v-model="config.primaryColor" type="color" @input="handlePrimaryChange" />
          <span class="ui-color-dot" :style="{ background: config.primaryColor }" />
        </label>
      </div>
      <div class="ui-row">
        <span class="ui-row-label">预设颜色</span>
        <div class="ui-preset-group">
          <button
            v-for="preset in primaryPresets"
            :key="preset.value"
            type="button"
            class="ui-preset-dot"
            :class="{ active: sameColor(config.primaryColor, preset.value) }"
            :style="{ background: preset.value }"
            :title="preset.label"
            @click="setPrimaryColor(preset.value)"
          />
        </div>
      </div>
    </div>

    <div class="ui-row">
      <span class="ui-row-label">自定义背景图片</span>
      <button
        class="ui-switch"
        type="button"
        role="switch"
        :aria-checked="config.enableCustomImage"
        :class="{ on: config.enableCustomImage }"
        @click="toggleImage"
      >
        <i />
      </button>
    </div>

    <div v-if="config.enableCustomImage" class="ui-image-block">
      <div class="ui-row">
        <span class="ui-row-label">图片地址</span>
        <input
          v-model.trim="config.bgImage"
          class="ui-text-input"
          type="text"
          placeholder="https://example.com/bg.jpg"
          @input="handleChange"
        />
      </div>
      <div class="ui-row">
        <span class="ui-row-label">本地上传</span>
        <button class="ui-file-btn" type="button" @click="triggerUpload">选择图片</button>
        <input
          ref="fileInputRef"
          class="ui-file-input"
          type="file"
          accept="image/*"
          @change="handleFileChange"
        />
      </div>
      <div class="ui-row">
        <span class="ui-row-label">缩放方式</span>
        <div class="ui-size-group">
          <button
            v-for="opt in sizeOptions"
            :key="opt.value"
            type="button"
            class="ui-size-option"
            :class="{ active: config.bgSize === opt.value }"
            @click="setBgSize(opt.value)"
          >
            {{ opt.label }}
          </button>
        </div>
      </div>
      <div v-if="config.bgImage" class="ui-preview">
        <img :src="config.bgImage" alt="背景预览" @error="previewFailed = true" v-if="!previewFailed" />
        <span v-else class="ui-preview-error">图片无法加载</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, watch } from 'vue'
import {
  defaultUiThemeConfig,
  loadCustomUiTheme,
  resetCustomUiTheme,
  saveCustomUiTheme,
  PRIMARY_COLOR_PRESETS,
  type CustomUiThemeConfig,
} from '../utils/themeCustom'

const config = reactive<CustomUiThemeConfig>(loadCustomUiTheme())
const fileInputRef = ref<HTMLInputElement | null>(null)
const previewFailed = ref(false)
const primaryPresets = PRIMARY_COLOR_PRESETS

const sizeOptions: Array<{ label: string; value: CustomUiThemeConfig['bgSize'] }> = [
  { label: '填充', value: 'cover' },
  { label: '适应', value: 'contain' },
  { label: '原始', value: 'auto' },
]

watch(
  () => config.bgImage,
  () => {
    previewFailed.value = false
  },
)

function handleChange() {
  saveCustomUiTheme({ ...config })
}

function toggleColor() {
  config.enableCustomColor = !config.enableCustomColor
  // 两个开关互斥：同时开启会让背景图被纯色覆盖
  if (config.enableCustomColor) config.enableCustomImage = false
  handleChange()
}

function toggleImage() {
  config.enableCustomImage = !config.enableCustomImage
  if (config.enableCustomImage) config.enableCustomColor = false
  handleChange()
}

function togglePrimary() {
  config.enableCustomPrimary = !config.enableCustomPrimary
  handleChange()
}

function handlePrimaryChange() {
  handleChange()
}

function setPrimaryColor(value: string) {
  config.primaryColor = value
  config.enableCustomPrimary = true
  handleChange()
}

function sameColor(a: string, b: string) {
  return (a || '').toLowerCase() === (b || '').toLowerCase()
}

function setBgSize(value: CustomUiThemeConfig['bgSize']) {
  config.bgSize = value
  handleChange()
}

function triggerUpload() {
  fileInputRef.value?.click()
}

function handleFileChange(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  const reader = new FileReader()
  reader.onload = () => {
    config.bgImage = String(reader.result || '')
    previewFailed.value = false
    handleChange()
  }
  reader.readAsDataURL(file)
  input.value = ''
}

function handleReset() {
  Object.assign(config, { ...defaultUiThemeConfig })
  resetCustomUiTheme()
}
</script>

<style scoped>
.ui-custom-card {
  padding: var(--space-3) var(--space-4);
  border-radius: var(--radius-lg);
  border: 1px solid var(--color-border-light);
  background: var(--color-bg-elevated);
}

.ui-custom-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-2);
}

.ui-custom-title {
  font-size: var(--text-sm);
  font-weight: 600;
}

.ui-reset-btn {
  padding: 4px 10px;
  border-radius: var(--radius-full);
  border: 1px solid var(--color-border-light);
  background: transparent;
  color: var(--color-text-tertiary);
  font-size: var(--text-xs);
  cursor: pointer;
}

.ui-reset-btn:hover {
  color: var(--color-primary);
  border-color: var(--color-primary-border);
}

.ui-row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: 9px 0;
  border-bottom: 1px solid var(--color-divider);
}

.ui-row:last-child {
  border-bottom: none;
}

.ui-row-label {
  flex-shrink: 0;
  min-width: 96px;
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
}

.ui-range {
  flex: 1;
  min-width: 0;
  height: 4px;
  accent-color: var(--color-primary);
  cursor: pointer;
}

.ui-range-value {
  flex-shrink: 0;
  min-width: 46px;
  text-align: right;
  font-size: var(--text-xs);
  color: var(--color-text-tertiary);
}

.ui-switch {
  margin-left: auto;
  width: 38px;
  height: 22px;
  padding: 2px;
  border: none;
  border-radius: var(--radius-full);
  background: var(--color-bg-sunken);
  cursor: pointer;
  transition: background var(--duration-fast) var(--ease-out);
}

.ui-switch i {
  display: block;
  width: 18px;
  height: 18px;
  border-radius: var(--radius-full);
  background: #ffffff;
  box-shadow: var(--shadow-xs);
  transition: transform var(--duration-fast) var(--ease-out);
}

.ui-switch.on {
  background: var(--color-primary);
}

.ui-switch.on i {
  transform: translateX(16px);
}

.ui-color-picker {
  margin-left: auto;
  position: relative;
  width: 28px;
  height: 28px;
  cursor: pointer;
}

.ui-color-picker input {
  position: absolute;
  inset: 0;
  opacity: 0;
  cursor: pointer;
}

.ui-color-dot {
  display: block;
  width: 28px;
  height: 28px;
  border-radius: var(--radius-full);
  border: 2px solid var(--color-border-light);
  box-shadow: var(--shadow-xs);
}

.ui-primary-block {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.ui-preset-group {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 8px;
}

.ui-preset-dot {
  width: 22px;
  height: 22px;
  border-radius: var(--radius-full);
  border: 2px solid transparent;
  box-shadow: var(--shadow-xs);
  cursor: pointer;
  padding: 0;
  transition: transform var(--duration-fast) var(--ease-out), border-color var(--duration-fast) var(--ease-out);
}

.ui-preset-dot:hover {
  transform: scale(1.1);
}

.ui-preset-dot.active {
  border-color: var(--color-text-secondary);
  transform: scale(1.08);
}

.ui-text-input {
  flex: 1;
  min-width: 0;
  padding: 7px 10px;
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border-light);
  background: var(--color-bg);
  color: var(--color-text);
  font-size: var(--text-xs);
}

.ui-file-btn {
  margin-left: auto;
  padding: 6px 12px;
  border-radius: var(--radius-full);
  border: 1px solid var(--color-border-light);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  font-size: var(--text-xs);
  cursor: pointer;
}

.ui-file-btn:hover {
  color: var(--color-primary);
  border-color: var(--color-primary-border);
}

.ui-file-input {
  display: none;
}

.ui-size-group {
  margin-left: auto;
  display: flex;
  gap: 6px;
}

.ui-size-option {
  padding: 5px 10px;
  border-radius: var(--radius-full);
  border: 1px solid var(--color-border-light);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  font-size: var(--text-xs);
  cursor: pointer;
}

.ui-size-option.active {
  border-color: var(--color-primary-border);
  background: var(--color-primary-bg);
  color: var(--color-primary);
}

.ui-preview {
  margin-top: var(--space-2);
  height: 84px;
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border-light);
  background: var(--color-bg-sunken);
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
}

.ui-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.ui-preview-error {
  font-size: var(--text-xs);
  color: var(--color-text-tertiary);
}
</style>
