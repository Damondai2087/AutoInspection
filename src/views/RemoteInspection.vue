<script setup lang="ts">
import { ref } from "vue";
import { ElMessage } from "element-plus";
import { VideoPlay, VideoPause, Folder } from "@element-plus/icons-vue";
import { store, resetLogs, clearReport } from "../store/app";
import {
  startInspection,
  stopInspection,
  pickDirectory,
  type InspectionRequest,
} from "../api/inspection";
import PermissionBanner from "../components/PermissionBanner.vue";
import LogConsole from "../components/LogConsole.vue";
import ReportPreview from "../components/ReportPreview.vue";

const targetSystem = ref<string>("linux");
const fast = ref(false);
const modules = ref<string[]>(["largefile", "update", "ssl"]);

const remote = ref({
  host: "",
  port: 22,
  username: "",
  auth: "key",
  password: "",
  key_path: "",
});

const moduleOptions = [
  { value: "largefile", label: "大文件扫描" },
  { value: "update", label: "系统更新检查" },
  { value: "ssl", label: "SSL 证书检查" },
];

async function browseKey() {
  const dir = await pickDirectory();
  if (dir) {
    remote.value.key_path = dir;
  }
}

async function onStart() {
  if (store.running) return;
  if (targetSystem.value !== "linux") {
    ElMessage.warning("目前仅支持远程 Linux 巡检（SSH），Windows/macOS 远程即将支持");
    return;
  }
  if (!remote.value.host || !remote.value.username) {
    ElMessage.warning("请填写远程主机 IP 和登录账号");
    return;
  }
  const req: InspectionRequest = {
    target_system: targetSystem.value,
    mode: "remote",
    modules: modules.value,
    fast: fast.value,
    remote: {
      host: remote.value.host,
      port: remote.value.port || 22,
      username: remote.value.username,
      auth: remote.value.auth,
      password: remote.value.password,
      key_path: remote.value.key_path,
    },
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
      <h3 class="ai-section-title">远程巡检配置</h3>

      <div class="config-grid">
        <div class="config-col">
          <el-form label-width="96px" label-position="left">
            <el-form-item label="目标系统">
              <el-select v-model="targetSystem" style="width: 200px">
                <el-option label="Linux 主机（SSH）" value="linux" />
                <el-option label="Windows 主机（即将支持）" value="windows" disabled />
                <el-option label="macOS 主机（即将支持）" value="macos" disabled />
              </el-select>
            </el-form-item>
            <el-form-item label="主机 IP">
              <el-input v-model="remote.host" placeholder="192.168.1.10" style="width: 200px" />
            </el-form-item>
            <el-form-item label="端口">
              <el-input v-model.number="remote.port" style="width: 120px" />
            </el-form-item>
            <el-form-item label="登录账号">
              <el-input v-model="remote.username" placeholder="root" style="width: 200px" />
            </el-form-item>
          </el-form>
        </div>
        <div class="config-col">
          <el-form label-width="96px" label-position="left">
            <el-form-item label="认证方式">
              <el-radio-group v-model="remote.auth">
                <el-radio value="key">密钥</el-radio>
                <el-radio value="password">密码</el-radio>
              </el-radio-group>
            </el-form-item>
            <el-form-item v-if="remote.auth === 'key'" label="密钥路径">
              <el-input v-model="remote.key_path" placeholder="私钥文件路径" style="width: 260px" />
              <el-button :icon="Folder" @click="browseKey" style="margin-left: 8px">选择</el-button>
            </el-form-item>
            <el-form-item v-else label="密码">
              <el-input
                v-model="remote.password"
                type="password"
                show-password
                placeholder="远程主机密码"
                style="width: 260px"
              />
            </el-form-item>
          </el-form>
        </div>
      </div>

      <div class="config-options">
        <el-form label-width="96px" label-position="left" class="inline-form">
          <el-form-item label="快速模式">
            <el-switch v-model="fast" />
            <span class="form-tip">开启后跳过耗时检查项</span>
          </el-form-item>
          <el-form-item label="可选检查项">
            <el-checkbox-group v-model="modules">
              <el-checkbox v-for="o in moduleOptions" :key="o.value" :value="o.value">
                {{ o.label }}
              </el-checkbox>
            </el-checkbox-group>
          </el-form-item>
        </el-form>
      </div>

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

.config-grid {
  display: flex;
  gap: 32px;
  margin-bottom: 12px;
}

.config-col {
  flex: 1;
}

.config-options {
  border-top: 1px solid var(--app-border-soft);
  padding-top: 12px;
}

.inline-form {
  display: flex;
  flex-wrap: wrap;
  gap: 0 32px;
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
