<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { ElMessage } from "element-plus";
import {
  Monitor,
  Connection,
  Document,
  Setting,
} from "@element-plus/icons-vue";
import LocalInspection from "./views/LocalInspection.vue";
import RemoteInspection from "./views/RemoteInspection.vue";
import HistoryReport from "./views/HistoryReport.vue";
import Settings from "./views/Settings.vue";
import {
  store,
  pushLog,
  clearReport,
  type InspectionResult,
  type LogLine,
} from "./store/app";
import {
  onInspectionLog,
  onInspectionDone,
  getOsInfo,
  getSettings,
  checkPermission,
  getHistory,
  readReport,
  getAppVersion,
} from "./api/inspection";

const unlisteners = ref<Array<() => void>>([]);
const appVersion = ref<string>("");

const navItems = [
  { index: "local", label: "本机巡检", icon: Monitor },
  { index: "remote", label: "远程巡检", icon: Connection },
  { index: "history", label: "历史报告", icon: Document },
  { index: "settings", label: "系统设置", icon: Setting },
];

function nowTs(): string {
  const d = new Date();
  const p = (n: number) => String(n).padStart(2, "0");
  return `${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
}

function applyTheme(theme: string) {
  if (theme === "light") {
    document.documentElement.classList.remove("dark");
  } else {
    document.documentElement.classList.add("dark");
  }
}

async function refreshHistory() {
  try {
    store.history = await getHistory();
  } catch {
    /* 忽略 */
  }
}

async function handleDone(result: InspectionResult) {
  store.running = false;
  if (result.success && result.report_path) {
    try {
      const html = await readReport(result.report_path);
      store.report = { path: result.report_path, html, loaded: true };
      ElMessage.success("巡检完成，报告已生成");
    } catch (e) {
      ElMessage.warning("报告读取失败：" + String(e));
    }
    await refreshHistory();
  } else {
    clearReport();
    ElMessage.error("巡检失败：" + result.message);
  }
}

async function init() {
  applyTheme(store.settings.theme || "light");
  try { store.osInfo = await getOsInfo(); } catch { /* ignore */ }
  try { store.settings = await getSettings(); applyTheme(store.settings.theme || "light"); } catch { /* ignore */ }
  try { store.permission = await checkPermission(); } catch { /* ignore */ }
  try { appVersion.value = await getAppVersion(); } catch { /* ignore */ }
  await refreshHistory();
}

onMounted(async () => {
  const offLog = await onInspectionLog((line: LogLine) => {
    pushLog({ ...line, ts: nowTs() });
  });
  const offDone = await onInspectionDone((r: InspectionResult) => {
    handleDone(r);
  });
  unlisteners.value = [offLog, offDone];
  await init();
});

onUnmounted(() => {
  unlisteners.value.forEach((u) => u());
});

function onNavClick(index: string) {
  store.activeNav = index;
  if (index === "history") refreshHistory();
}
</script>

<template>
  <div class="app-shell">
    <!-- Sidebar -->
    <aside class="sidebar">
      <div class="sidebar-brand">
        <div class="brand-icon">
          <svg width="18" height="18" viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M16 4L7 9.2V14.6C7 20.6 10.6 25.8 16 27.2C21.4 25.8 25 20.6 25 14.6V9.2L16 4Z" stroke="white" stroke-width="2" stroke-linejoin="round"/>
            <path d="M12 16L14.5 18.5L20 13" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </div>
        <div class="brand-text">
          <span class="brand-name">AutoInspection</span>
          <span class="brand-sub">跨平台主机巡检</span>
        </div>
      </div>

      <nav class="sidebar-nav">
        <button
          v-for="item in navItems"
          :key="item.index"
          class="nav-item"
          :class="{ active: store.activeNav === item.index }"
          @click="onNavClick(item.index)"
        >
          <el-icon :size="16"><component :is="item.icon" /></el-icon>
          <span>{{ item.label }}</span>
        </button>
      </nav>

      <div class="sidebar-foot">
        <div class="foot-info">
          <span class="foot-dot" :class="store.osInfo.platform"></span>
          <span>{{ store.osInfo.name || "检测中" }}</span>
        </div>
        <span v-if="appVersion" class="foot-ver">v{{ appVersion }}</span>
      </div>
    </aside>

    <!-- Main Content -->
    <main class="main">
      <LocalInspection v-if="store.activeNav === 'local'" />
      <RemoteInspection v-else-if="store.activeNav === 'remote'" />
      <HistoryReport v-else-if="store.activeNav === 'history'" />
      <Settings v-else-if="store.activeNav === 'settings'" />
    </main>
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  height: 100%;
  overflow: hidden;
}

/* ---- Sidebar ---- */
.sidebar {
  width: 220px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  background: var(--app-sidebar-bg);
  border-right: 1px solid var(--app-sidebar-border);
}

.sidebar-brand {
  padding: 16px 14px 12px;
  display: flex;
  align-items: center;
  gap: 10px;
}

.brand-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--app-primary);
  color: #fff;
  border-radius: var(--app-radius-sm);
  flex-shrink: 0;
}

.brand-text {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.brand-name {
  font-weight: 700;
  font-size: 15px;
  color: var(--app-text);
  letter-spacing: -0.01em;
  line-height: 1.2;
}

.brand-sub {
  font-size: 11px;
  color: var(--app-text-muted);
  line-height: 1.2;
}

/* ---- Navigation ---- */
.sidebar-nav {
  flex: 1;
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow-y: auto;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 8px 10px;
  border: none;
  border-radius: var(--app-radius-sm);
  background: transparent;
  color: var(--app-text-soft);
  font-size: 13.5px;
  font-family: inherit;
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
  text-align: left;
  line-height: 1.4;
}

.nav-item:hover {
  background: var(--app-sidebar-accent);
  color: var(--app-text);
}

.nav-item.active {
  background: var(--app-primary-soft);
  color: var(--app-primary);
  font-weight: 500;
}

.nav-item.active :deep(.el-icon) {
  color: var(--app-primary);
}

/* ---- Footer ---- */
.sidebar-foot {
  padding: 10px 14px;
  font-size: 12px;
  color: var(--app-text-muted);
  border-top: 1px solid var(--app-sidebar-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.foot-info {
  display: flex;
  align-items: center;
  gap: 6px;
}

.foot-ver {
  font-size: 11px;
  opacity: 0.6;
}

.foot-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--app-info);
  flex-shrink: 0;
}
.foot-dot.windows { background: #0078d4; }
.foot-dot.linux { background: #fcc624; }
.foot-dot.macos { background: #a2aaad; }

/* ---- Main ---- */
.main {
  flex: 1;
  min-width: 0;
  background: var(--app-bg);
  overflow: hidden;
}
</style>
