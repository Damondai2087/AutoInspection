<script setup lang="ts">
import { ElMessage } from "element-plus";
import { Folder } from "@element-plus/icons-vue";
import { store } from "../store/app";
import { saveSettings, pickDirectory } from "../api/inspection";

async function browseOutput() {
  const dir = await pickDirectory();
  if (dir) store.settings.output_dir = dir;
}
async function browseScript() {
  const dir = await pickDirectory();
  if (dir) store.settings.custom_script_dir = dir;
}

function applyTheme(theme: string) {
  if (theme === "light") document.documentElement.classList.remove("dark");
  else document.documentElement.classList.add("dark");
}

async function onSave() {
  try {
    await saveSettings(store.settings);
    applyTheme(store.settings.theme || "light");
    ElMessage.success("设置已保存");
  } catch (e) {
    ElMessage.error("保存失败：" + String(e));
  }
}
</script>

<template>
  <div class="ai-page">
    <h3 class="ai-section-title">系统设置</h3>
    <div class="ai-card settings-card">
      <el-form label-width="120px" label-position="left">
        <el-form-item label="默认输出目录">
          <el-input v-model="store.settings.output_dir" placeholder="报告默认保存目录" style="width: 320px" />
          <el-button :icon="Folder" @click="browseOutput" style="margin-left: 8px">选择</el-button>
        </el-form-item>

        <el-form-item label="主题">
          <el-radio-group v-model="store.settings.theme">
            <el-radio value="light">浅色</el-radio>
            <el-radio value="dark">深色</el-radio>
          </el-radio-group>
        </el-form-item>

        <el-form-item label="自定义脚本目录">
          <el-input v-model="store.settings.custom_script_dir" placeholder="留空则使用内置脚本" style="width: 320px" />
          <el-button :icon="Folder" @click="browseScript" style="margin-left: 8px">选择</el-button>
        </el-form-item>
        <el-form-item>
          <span class="form-tip">指定目录需包含 linux_inspect.sh 与 win_inspection_html.py，用于覆盖内置脚本。</span>
        </el-form-item>
      </el-form>

      <div class="settings-actions">
        <el-button type="primary" @click="onSave">保存设置</el-button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-card {
  max-width: 640px;
}

.form-tip {
  font-size: 12.5px;
  color: var(--app-text-muted);
}

.settings-actions {
  margin-top: 8px;
  padding-top: 16px;
  border-top: 1px solid var(--app-border-soft);
}
</style>
