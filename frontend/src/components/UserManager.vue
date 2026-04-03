<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="modelValue" class="modal-overlay" @click="close"></div>
    </Transition>
    <Transition name="scale">
      <div v-if="modelValue" class="modal-container" @click.self="close">
        <section class="user-manager-modal">
          <header class="modal-header">
            <div>
              <h2>用户管理</h2>
              <p class="subtitle">管理账号、密码与服务器备份/本地存储权限</p>
            </div>
            <div class="header-actions">
              <button class="icon-btn" :class="{ spinning: loading }" @click="loadUsers" title="刷新">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
                  <path d="M3 3v5h5" />
                  <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" />
                  <path d="M16 16h5v5" />
                </svg>
              </button>
              <button class="icon-btn" @click="close" title="关闭">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M18 6 6 18M6 6l12 12" />
                </svg>
              </button>
            </div>
          </header>

          <div v-if="!canManageUsers" class="notice warning">
            <strong>{{ unavailableTitle }}</strong>
            <span>{{ unavailableMessage }}</span>
          </div>

          <template v-else>
            <div class="summary-grid">
              <div class="summary-card">
                <span>{{ users.length }}</span>
                <small>账号总数</small>
              </div>
              <div class="summary-card">
                <span>{{ adminCount }}</span>
                <small>管理员</small>
              </div>
              <div class="summary-card">
                <span>{{ backupEnabledCount }}</span>
                <small>已开服务器备份</small>
              </div>
              <div class="summary-card">
                <span>{{ localStoreEnabledCount }}</span>
                <small>已开本地存储</small>
              </div>
            </div>

            <section class="panel">
              <div class="panel-head">
                <div>
                  <h3>新增用户</h3>
                  <p>用户名仅支持小写字母和数字，密码至少 8 位</p>
                </div>
              </div>
              <form class="create-form" @submit.prevent="handleCreateUser">
                <label class="field">
                  <span>用户名</span>
                  <input v-model.trim="createForm.username" type="text" placeholder="例如 reader01" autocomplete="off" />
                </label>
                <label class="field">
                  <span>密码</span>
                  <input v-model="createForm.password" type="password" placeholder="输入初始密码" autocomplete="new-password" />
                </label>
                <button class="action-btn primary" type="submit" :disabled="working">
                  {{ working ? '处理中...' : '创建用户' }}
                </button>
              </form>
            </section>

            <section class="panel list-panel">
              <div class="panel-head">
                <div>
                  <h3>账号列表</h3>
                  <p>可直接调整权限、重置密码或删除账号</p>
                </div>
              </div>

              <div v-if="loading" class="empty-state">正在加载用户列表...</div>
              <div v-else-if="users.length === 0" class="empty-state">暂无用户数据</div>
              <div v-else class="user-list">
                <article v-for="user in sortedUsers" :key="user.username" class="user-card">
                  <div class="user-card-top">
                    <div class="user-meta">
                      <div class="user-name-row">
                        <strong>{{ user.username }}</strong>
                        <span v-if="user.username === currentUsername" class="badge accent">当前账号</span>
                        <span v-if="user.isAdmin" class="badge">管理员</span>
                      </div>
                      <div class="user-times">
                        <span>创建时间：{{ formatTime(user.createdAt) }}</span>
                        <span>最近登录：{{ formatTime(user.lastLoginAt) }}</span>
                      </div>
                    </div>
                    <button
                      class="mini-btn danger"
                      :disabled="working || user.username === currentUsername"
                      @click="handleDeleteUser(user)"
                    >
                      删除
                    </button>
                  </div>

                  <div class="permission-grid">
                    <label class="permission-item">
                      <span>服务器备份</span>
                      <input
                        type="checkbox"
                        :checked="!!user.enableWebdav"
                        :disabled="working"
                        @change="handleTogglePermission(user, 'enableWebdav', ($event.target as HTMLInputElement).checked)"
                      />
                    </label>
                    <label class="permission-item">
                      <span>本地存储</span>
                      <input
                        type="checkbox"
                        :checked="!!user.enableLocalStore"
                        :disabled="working"
                        @change="handleTogglePermission(user, 'enableLocalStore', ($event.target as HTMLInputElement).checked)"
                      />
                    </label>
                  </div>

                  <div class="reset-panel">
                    <button
                      class="mini-btn"
                      :disabled="working"
                      @click="handleSetDefaultBookSources(user)"
                    >
                      设为默认书源
                    </button>
                    <button
                      class="mini-btn"
                      :class="{ active: resetTarget === user.username }"
                      @click="toggleResetTarget(user.username)"
                    >
                      {{ resetTarget === user.username ? '收起重置密码' : '重置密码' }}
                    </button>
                    <div v-if="resetTarget === user.username" class="reset-form">
                      <input
                        v-model="resetPasswordValue"
                        type="password"
                        placeholder="输入新密码"
                        autocomplete="new-password"
                      />
                      <button class="mini-btn primary" :disabled="working" @click="handleResetPassword(user.username)">
                        保存新密码
                      </button>
                    </div>
                  </div>
                </article>
              </div>
            </section>
          </template>
        </section>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { addUser, deleteUsers, getUserList, resetPassword, updateUser } from '../api/user'
import { setAsDefaultBookSources } from '../api/source'
import { useAppStore } from '../stores/app'
import type { UserInfo } from '../types'

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const appStore = useAppStore()

const users = ref<UserInfo[]>([])
const loading = ref(false)
const working = ref(false)
const resetTarget = ref('')
const resetPasswordValue = ref('')
const createForm = reactive({
  username: '',
  password: '',
})

const currentUsername = computed(() => appStore.userInfo?.username || '')
const canManageUsers = computed(() => appStore.isSecureMode && appStore.isLoggedIn && !!appStore.userInfo?.isAdmin)
const adminCount = computed(() => users.value.filter((user) => user.isAdmin).length)
const backupEnabledCount = computed(() => users.value.filter((user) => user.enableWebdav).length)
const localStoreEnabledCount = computed(() => users.value.filter((user) => user.enableLocalStore).length)
const sortedUsers = computed(() =>
  [...users.value].sort((a, b) => {
    if (!!a.isAdmin !== !!b.isAdmin) return a.isAdmin ? -1 : 1
    return a.username.localeCompare(b.username)
  }),
)

const unavailableTitle = computed(() => {
  if (!appStore.isSecureMode) return '当前未开启安全模式'
  if (!appStore.isLoggedIn) return '需要先登录管理员账号'
  return '当前账号没有用户管理权限'
})

const unavailableMessage = computed(() => {
  if (!appStore.isSecureMode) return '用户管理仅在多用户安全模式下可用。'
  if (!appStore.isLoggedIn) return '登录管理员账号后，才能管理其他用户。'
  return '请使用管理员账号登录后再试。'
})

watch(
  () => props.modelValue,
  (visible) => {
    if (visible && canManageUsers.value) {
      void loadUsers()
    }
    if (!visible) {
      createForm.username = ''
      createForm.password = ''
      resetTarget.value = ''
      resetPasswordValue.value = ''
    }
  },
)

function close() {
  emit('update:modelValue', false)
}

function formatTime(value?: number) {
  if (!value) return '-'
  return new Date(value).toLocaleString()
}

function applyUserList(list: UserInfo[]) {
  users.value = list
  const current = list.find((item) => item.username === currentUsername.value)
  if (current && appStore.userInfo) {
    appStore.updateUserInfo({
      ...appStore.userInfo,
      ...current,
      accessToken: appStore.userInfo.accessToken,
    })
  }
}

async function loadUsers() {
  if (!canManageUsers.value) return
  loading.value = true
  try {
    applyUserList(await getUserList())
  } catch (error) {
    appStore.showToast((error as Error).message || '加载用户列表失败', 'error')
  } finally {
    loading.value = false
  }
}

async function handleCreateUser() {
  if (!createForm.username || !createForm.password) {
    appStore.showToast('请填写用户名和密码', 'warning')
    return
  }
  working.value = true
  try {
    applyUserList(await addUser(createForm.username, createForm.password))
    createForm.username = ''
    createForm.password = ''
    appStore.showToast('用户创建成功', 'success')
  } catch (error) {
    appStore.showToast((error as Error).message || '创建用户失败', 'error')
  } finally {
    working.value = false
  }
}

async function handleTogglePermission(
  user: UserInfo,
  key: 'enableWebdav' | 'enableLocalStore',
  value: boolean,
) {
  working.value = true
  try {
    applyUserList(await updateUser(user.username, { [key]: value }))
    appStore.showToast('用户权限已更新', 'success')
  } catch (error) {
    appStore.showToast((error as Error).message || '更新用户权限失败', 'error')
  } finally {
    working.value = false
  }
}

function toggleResetTarget(username: string) {
  if (resetTarget.value === username) {
    resetTarget.value = ''
    resetPasswordValue.value = ''
    return
  }
  resetTarget.value = username
  resetPasswordValue.value = ''
}

async function handleResetPassword(username: string) {
  if (!resetPasswordValue.value) {
    appStore.showToast('请输入新密码', 'warning')
    return
  }
  working.value = true
  try {
    await resetPassword(username, resetPasswordValue.value)
    resetTarget.value = ''
    resetPasswordValue.value = ''
    appStore.showToast(`已重置 ${username} 的密码`, 'success')
  } catch (error) {
    appStore.showToast((error as Error).message || '重置密码失败', 'error')
  } finally {
    working.value = false
  }
}

async function handleDeleteUser(user: UserInfo) {
  if (user.username === currentUsername.value) {
    appStore.showToast('不能删除当前登录账号', 'warning')
    return
  }
  if (!confirm(`确定删除用户 "${user.username}" 吗？`)) return
  working.value = true
  try {
    applyUserList(await deleteUsers([user.username]))
    appStore.showToast(`已删除用户 ${user.username}`, 'success')
  } catch (error) {
    appStore.showToast((error as Error).message || '删除用户失败', 'error')
  } finally {
    working.value = false
  }
}

async function handleSetDefaultBookSources(user: UserInfo) {
  if (!confirm(`确定将用户 "${user.username}" 的书源设为默认书源吗？新注册用户将继承这套书源。`)) return
  working.value = true
  try {
    await setAsDefaultBookSources(user.username)
    appStore.showToast(`已将 ${user.username} 的书源设为默认书源`, 'success')
  } catch (error) {
    appStore.showToast((error as Error).message || '设置默认书源失败', 'error')
  } finally {
    working.value = false
  }
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.45);
  backdrop-filter: blur(6px);
  z-index: var(--z-overlay);
}

.modal-container {
  position: fixed;
  inset: 0;
  z-index: var(--z-modal);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-6);
}

.user-manager-modal {
  width: min(1080px, 100%);
  max-height: min(88vh, 920px);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border-light);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-xl);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.modal-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
  padding: var(--space-5) var(--space-6);
  border-bottom: 1px solid var(--color-divider);
}

.modal-header h2 {
  font-size: var(--text-xl);
  font-weight: 700;
}

.subtitle {
  margin-top: var(--space-1);
  color: var(--color-text-secondary);
  font-size: var(--text-sm);
}

.header-actions {
  display: flex;
  gap: var(--space-2);
}

.icon-btn,
.action-btn,
.mini-btn {
  border: none;
  font: inherit;
}

.icon-btn {
  width: 38px;
  height: 38px;
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon-btn:hover {
  background: var(--color-bg-hover);
}

.icon-btn svg {
  width: 18px;
  height: 18px;
}

.icon-btn.spinning svg {
  animation: spin 1s linear infinite;
}

.summary-grid,
.panel {
  margin: var(--space-4) var(--space-6) 0;
}

.summary-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: var(--space-3);
}

.summary-card,
.panel,
.notice,
.user-card {
  border: 1px solid var(--color-border-light);
  background: var(--color-bg);
  border-radius: var(--radius-lg);
}

.summary-card {
  padding: var(--space-4);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.summary-card span {
  font-size: var(--text-2xl);
  font-weight: 700;
}

.summary-card small,
.panel-head p,
.user-times,
.notice span {
  color: var(--color-text-tertiary);
}

.notice {
  margin: var(--space-4) var(--space-6) 0;
  display: grid;
  gap: 4px;
  padding: var(--space-3) var(--space-4);
  font-size: var(--text-sm);
  background: rgba(201, 127, 58, 0.12);
  border-color: rgba(201, 127, 58, 0.18);
}

.panel {
  padding: var(--space-4);
}

.panel-head {
  display: flex;
  justify-content: space-between;
  gap: var(--space-4);
  margin-bottom: var(--space-4);
}

.panel-head h3 {
  font-size: var(--text-base);
  font-weight: 700;
}

.panel-head p {
  margin-top: 4px;
  font-size: var(--text-sm);
}

.create-form,
.reset-form {
  display: flex;
  gap: var(--space-3);
  align-items: end;
}

.field {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.field span {
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
}

.field input,
.reset-form input {
  min-height: 40px;
  padding: 0 var(--space-3);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border);
  background: var(--color-bg-elevated);
  color: inherit;
}

.action-btn,
.mini-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border-light);
  background: var(--color-bg-sunken);
  color: var(--color-text);
  transition: all var(--duration-fast);
}

.action-btn {
  min-height: 40px;
  padding: 0 var(--space-4);
  font-size: var(--text-sm);
  font-weight: 600;
  white-space: nowrap;
}

.mini-btn {
  min-height: 32px;
  padding: 0 var(--space-3);
  font-size: var(--text-xs);
  font-weight: 600;
}

.action-btn.primary,
.mini-btn.primary {
  background: var(--color-primary);
  border-color: var(--color-primary);
  color: #fff;
}

.mini-btn.danger {
  color: var(--color-danger);
}

.action-btn:disabled,
.mini-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.list-panel {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.user-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  overflow: auto;
  padding-right: var(--space-1);
}

.user-card {
  padding: var(--space-4);
}

.user-card-top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-3);
}

.user-meta {
  min-width: 0;
}

.user-name-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-2);
}

.user-times {
  margin-top: var(--space-2);
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-3);
  font-size: var(--text-xs);
}

.badge {
  padding: 2px 8px;
  border-radius: 999px;
  background: var(--color-bg-sunken);
  color: var(--color-text-secondary);
  font-size: 11px;
  font-weight: 600;
}

.badge.accent {
  background: rgba(201, 127, 58, 0.12);
  color: var(--color-primary-dark);
}

.permission-grid {
  margin-top: var(--space-4);
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-3);
}

.permission-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-3);
  border-radius: var(--radius-md);
  background: var(--color-bg-sunken);
}

.reset-panel {
  margin-top: var(--space-4);
  display: grid;
  gap: var(--space-3);
}

.reset-form input {
  flex: 1;
}

.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 220px;
  color: var(--color-text-tertiary);
  font-size: var(--text-sm);
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 900px) {
  .modal-container {
    padding: var(--space-3);
  }

  .user-manager-modal {
    max-height: 92vh;
  }

  .summary-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .create-form,
  .reset-form {
    flex-direction: column;
    align-items: stretch;
  }

  .permission-grid {
    grid-template-columns: 1fr;
  }
}
</style>
