<script setup lang="ts">
import { computed, ref } from "vue";
import { ElMessage } from "element-plus";
import { VideoPlay, VideoPause } from "@element-plus/icons-vue";
import { store, resetLogs, clearReport } from "../store/app";
import { startInspection, stopInspection, type InspectionRequest } from "../api/inspection";
import PermissionBanner from "../components/PermissionBanner.vue";
import LogConsole from "../components/LogConsole.vue";
import ReportPreview from "../components/ReportPreview.vue";

const targetSystem = ref<string>(store.osInfo.platform || "linux");
const fast = ref(false);
const modules = ref<string[]>(["largefile", "update", "ssl"]);

const moduleOptions = [
  { value: "largefile", label: "大文件扫描" },
  { value: "update", label: "系统更新检查" },
  { value: "ssl", label: "SSL 证书检查" },
];

const matchHost = computed(
  () => targetSystem.value === store.osInfo.platform
);

async function onStart() {
  if (store.running) return;
  if (targetSystem.value === "macos") {
    ElMessage.warning("macOS 巡检脚本即将支持，敬请期待");
    return;
  }
  if (!matchHost.value) {
    ElMessage.warning(
      `本机巡检需在 ${targetSystem.value} 系统上运行，但当前系统是 ${store.osInfo.platform}。请选择匹配的系统，或使用「远程巡检」。`
    );
    return;
  }
  const req: InspectionRequest = {
    target_system: targetSystem.value,
    mode: "local",
    modules: modules.value,
    fast: fast.value,
    remote: { host: "", port: 22, username: "", auth: "key", password: "", key_path: "" },
    output_dir: "",
  };
  resetLogs();
  clearReport();
  store.running = true;
  try {
    await startInspection(req);
  } catch (e) {
    store.running = false;
    ElMessage.error("启动失败：" + String(e));
  }
}

async function onStop() {
  if (!store.running) return;
  try {
    await stopInspection();
    ElMessage.info("已发送终止信号");
  } catch (e) {
    ElMessage.error("停止失败：" + String(e));
  }
}
</script>

<template>
  <div class="ai-page">
    <div class="ai-card config-card">
      <h3 class="ai-section-title">本机巡检配置</h3>
      <el-form label-width="96px" label-position="left" class="config-form">
        <el-form-item label="目标系统">
          <el-select v-model="targetSystem" style="width: 240px">
            <el-option label="Windows 主机巡检" value="windows" />
            <el-option label="Linux 主机巡检" value="linux" />
            <el-option label="macOS 主机巡检（即将支持）" value="macos" disabled />
          </el-select>
          <span v-if="!matchHost" class="form-tip">
            当前系统为 {{ store.osInfo.name }}，本机巡检需选择匹配的系统
          </span>
        </el-form-item>

        <el-form-item label="快速模式">
          <el-switch v-model="fast" />
          <span class="form-tip">开启后跳过耗时检查，约 10-15 秒完成</span>
        </el-form-item>

        <el-form-item label="可选检查项">
          <el-checkbox-group v-model="modules">
            <el-checkbox v-for="o in moduleOptions" :key="o.value" :value="o.value">
              {{ o.label }}
            </el-checkbox>
          </el-checkbox-group>
        </el-form-item>
      </el-form>

      <PermissionBanner />

      <div class="actions">
        <el-button
          type="primary"
          :icon="VideoPlay"
          :loading="store.running"
          :disabled="store.running"
          @click="onStart"
        >
          {{ store.running ? "巡检中..." : "开始巡检" }}
        </el-button>
        <el-button
          :icon="VideoPause"
          :disabled="!store.running"
          @click="onStop"
        >
          停止任务
        </el-button>
      </div>
    </div>

    <div class="split">
      <div class="pane">
        <div class="pane-head">
          <span>实时执行日志</span>
          <span v-if="store.running" class="pane-badge running">运行中</span>
        </div>
        <div class="pane-body"><LogConsole /></div>
      </div>
      <div class="pane">
        <div class="pane-head">巡检报告预览</div>
        <div class="pane-body"><ReportPreview /></div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.config-card {
  flex-shrink: 0;
}

.config-form {
  margin-bottom: 4px;
}

.form-tip {
  margin-left: 12px;
  font-size: 12.5px;
  color: var(--app-text-muted);
}

.actions {
  margin-top: 16px;
  display: flex;
  gap: 10px;
}

.split {
  flex: 1;
  display: flex;
  gap: 14px;
  min-height: 0;
}

.pane {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  background: var(--app-panel);
  border: 1px solid var(--app-border);
  border-radius: var(--app-radius);
  overflow: hidden;
  box-shadow: var(--app-shadow-sm);
}

.pane-head {
  padding: 10px 14px;
  font-size: 13px;
  font-weight: 600;
  color: var(--app-text);
  border-bottom: 1px solid var(--app-border-soft);
  background: var(--app-bg-soft);
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.pane-badge {
  font-size: 11px;
  padding: 1px 7px;
  border-radius: 999px;
  font-weight: 500;
}
.pane-badge.running {
  background: var(--app-primary-soft);
  color: var(--app-primary);
}

.pane-body {
  flex: 1;
  min-height: 0;
  padding: 10px;
}
</style>
