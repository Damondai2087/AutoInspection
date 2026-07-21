<script setup lang="ts">
import { computed } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { FolderOpened, View, Delete } from "@element-plus/icons-vue";
import { store } from "../store/app";
import { openPath, readReport, deleteHistoryItem, getHistory } from "../api/inspection";

const sysLabel: Record<string, string> = {
  windows: "Windows",
  linux: "Linux",
  macos: "macOS",
};
const modeLabel: Record<string, string> = { local: "本机", remote: "远程" };

const statusMeta: Record<string, { text: string; type: string }> = {
  ok: { text: "正常", type: "success" },
  warn: { text: "警告", type: "warning" },
  critical: { text: "严重", type: "danger" },
  failed: { text: "失败", type: "info" },
};

const list = computed(() => store.history.slice().reverse());

async function preview(item: (typeof store.history)[number]) {
  if (!item.report_path) {
    ElMessage.warning("该记录没有报告文件");
    return;
  }
  try {
    const html = await readReport(item.report_path);
    store.report = { path: item.report_path, html, loaded: true };
    store.activeNav = "local";
    ElMessage.success("已在「本机巡检」预览区打开报告");
  } catch (e) {
    ElMessage.error("报告读取失败：" + String(e));
  }
}

async function openReport(item: (typeof store.history)[number]) {
  if (!item.report_path) {
    ElMessage.warning("该记录没有报告文件");
    return;
  }
  try {
    await openPath(item.report_path);
  } catch (e) {
    ElMessage.error("打开失败：" + String(e));
  }
}

async function openDir(item: (typeof store.history)[number]) {
  if (!item.report_path) return;
  const dir = item.report_path.substring(0, item.report_path.lastIndexOf("/") >= 0
    ? item.report_path.lastIndexOf("/")
    : item.report_path.lastIndexOf("\\"));
  try {
    await openPath(dir);
  } catch (e) {
    ElMessage.error("打开目录失败：" + String(e));
  }
}

async function remove(item: (typeof store.history)[number]) {
  try {
    await ElMessageBox.confirm(
      `确定删除这条巡检记录吗？\n\n时间：${item.timestamp}\n目标：${item.target}`,
      "删除确认",
      { confirmButtonText: "删除", cancelButtonText: "取消", type: "warning" }
    );
  } catch {
    return; // 取消
  }
  try {
    await deleteHistoryItem(item.id);
    store.history = await getHistory();
    ElMessage.success("已删除");
  } catch (e) {
    ElMessage.error("删除失败：" + String(e));
  }
}
</script>

<template>
  <div class="ai-page">
    <h3 class="ai-section-title">历史巡检记录</h3>
    <div class="ai-card table-card">
      <el-empty v-if="list.length === 0" description="暂无巡检记录" />
      <el-table v-else :data="list" style="width: 100%" stripe>
        <el-table-column prop="timestamp" label="时间" width="170" />
        <el-table-column label="目标系统" width="100">
          <template #default="{ row }">{{ sysLabel[row.target_system] || row.target_system }}</template>
        </el-table-column>
        <el-table-column label="模式" width="70">
          <template #default="{ row }">{{ modeLabel[row.mode] || row.mode }}</template>
        </el-table-column>
        <el-table-column prop="target" label="巡检目标" width="180" />
        <el-table-column label="状态" width="80">
          <template #default="{ row }">
            <el-tag :type="(statusMeta[row.status]?.type as any) || 'info'" size="small">
              {{ statusMeta[row.status]?.text || row.status }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="summary" label="摘要" min-width="160" />
        <el-table-column label="操作" width="240" fixed="right">
          <template #default="{ row }">
            <el-button :icon="View" size="small" @click="preview(row)">预览</el-button>
            <el-button :icon="FolderOpened" size="small" @click="openDir(row)">目录</el-button>
            <el-button size="small" type="primary" link @click="openReport(row)">打开</el-button>
            <el-button size="small" type="danger" link :icon="Delete" @click="remove(row)" />
          </template>
        </el-table-column>
      </el-table>
    </div>
  </div>
</template>

<style scoped>
.table-card {
  flex: 1;
  overflow: auto;
  display: flex;
  flex-direction: column;
}
</style>
