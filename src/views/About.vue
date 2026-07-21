<script setup lang="ts">
import { onMounted, ref } from "vue";
import { Connection, Document, Setting } from "@element-plus/icons-vue";
import { getAppVersion, openPath } from "../api/inspection";

const version = ref<string>("");
const repoUrl = "https://github.com/souvc/AutoInspection";
const issuesUrl = "https://github.com/souvc/AutoInspection/issues";

onMounted(async () => {
  try { version.value = await getAppVersion(); } catch { /* ignore */ }
});

function open(url: string) {
  openPath(url).catch(() => window.open(url, "_blank"));
}
</script>

<template>
  <div class="ai-page about-page">
    <h3 class="ai-section-title">关于我们</h3>

    <div class="ai-card about-card">
      <div class="about-header">
        <img src="/logo.svg" class="about-logo" alt="logo" />
        <div class="about-meta">
          <h2>AutoInspection</h2>
          <p class="muted">跨平台主机自动巡检桌面客户端</p>
          <p class="version" v-if="version">v{{ version }}</p>
        </div>
      </div>

      <p class="about-desc">
        基于 Rust + Tauri v2 + Vue3 构建，整合 Linux / Windows 开源巡检脚本，
        一套客户端覆盖本机巡检、远程 SSH 巡检、HTML 报告在线预览、历史记录管理。
      </p>

      <div class="about-links">
        <el-button @click="open(repoUrl)">
          <el-icon style="margin-right: 6px"><Connection /></el-icon>
          GitHub 仓库
        </el-button>
        <el-button @click="open(issuesUrl)">
          <el-icon style="margin-right: 6px"><Document /></el-icon>
          提交 Issue
        </el-button>
        <el-button @click="open('https://github.com/souvc/AutoInspection/releases')">
          <el-icon style="margin-right: 6px"><Setting /></el-icon>
          查看更新
        </el-button>
      </div>
    </div>

    <div class="ai-card about-card">
      <h3 class="ai-section-title">联系我们</h3>
      <div class="qr-row">
        <div class="qr-item">
          <img src="/qr-wechat.png" alt="微信反馈群" />
          <div class="qr-title">问题反馈</div>
          <div class="qr-desc muted">扫码加群，反馈 Bug 或提需求</div>
        </div>
        <div class="qr-item">
          <img src="/qr-reward.png" alt="赞赏码" />
          <div class="qr-title">赞赏作者</div>
          <div class="qr-desc muted">如果觉得有用，欢迎投喂</div>
        </div>
      </div>
    </div>

    <div class="ai-card about-card about-tech">
      <h3 class="ai-section-title">技术栈</h3>
      <div class="tech-row">
        <div class="tech-tag">Rust</div>
        <div class="tech-tag">Tauri v2</div>
        <div class="tech-tag">Vue 3</div>
        <div class="tech-tag">Element Plus</div>
        <div class="tech-tag">Vite</div>
        <div class="tech-tag">TypeScript</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.about-page {
  align-items: center;
}

.about-card {
  max-width: 720px;
  width: 100%;
}

.about-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 16px;
}

.about-logo {
  width: 56px;
  height: 56px;
  border-radius: 12px;
  flex-shrink: 0;
}

.about-meta h2 {
  margin: 0 0 4px;
  font-size: 18px;
  font-weight: 700;
  letter-spacing: -0.01em;
}

.muted {
  color: var(--app-text-muted);
  font-size: 13px;
  margin: 0;
}

.version {
  color: var(--app-primary);
  font-size: 12.5px;
  font-weight: 500;
  margin: 2px 0 0;
}

.about-desc {
  color: var(--app-text-soft);
  font-size: 14px;
  line-height: 1.7;
  margin: 0 0 16px;
}

.about-links {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.qr-row {
  display: flex;
  gap: 32px;
  justify-content: center;
  margin-top: 12px;
}

.qr-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 16px 24px;
  border: 1px solid var(--app-border-soft);
  border-radius: var(--app-radius);
  background: var(--app-bg-soft);
  min-width: 160px;
}

.qr-item img {
  width: 140px;
  height: 140px;
  border-radius: 4px;
}

.qr-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--app-text);
  margin-top: 6px;
}

.qr-desc {
  font-size: 12px;
  text-align: center;
}

.tech-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 8px;
}

.tech-tag {
  padding: 5px 12px;
  background: var(--app-primary-soft);
  color: var(--app-primary);
  font-size: 12.5px;
  font-weight: 500;
  border-radius: 999px;
}
</style>
