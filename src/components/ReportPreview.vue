<script setup lang="ts">
import { store } from "../store/app";
</script>

<template>
  <div class="report-preview">
    <div v-if="!store.report.loaded" class="report-empty">
      <div class="re-icon">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
          <polyline points="14 2 14 8 20 8"/>
          <line x1="16" y1="13" x2="8" y2="13"/>
          <line x1="16" y1="17" x2="8" y2="17"/>
          <polyline points="10 9 9 9 8 9"/>
        </svg>
      </div>
      <div class="re-title">巡检报告预览</div>
      <div class="re-desc">巡检完成后，报告将在这里预览</div>
    </div>
    <iframe
      v-else
      class="report-frame"
      :srcdoc="store.report.html"
      sandbox="allow-scripts allow-same-origin"
    ></iframe>
  </div>
</template>

<style scoped>
.report-preview {
  height: 100%;
  background: #ffffff;
  border: 1px solid var(--app-border-soft);
  border-radius: var(--app-radius-sm);
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
}

.report-frame {
  width: 100%;
  height: 100%;
  border: none;
  background: #fff;
}

.report-empty {
  text-align: center;
  color: var(--app-text-muted);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 24px;
}

.re-icon {
  color: var(--app-text-muted);
  margin-bottom: 4px;
  opacity: 0.5;
}

.re-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--app-text-soft);
}

.re-desc {
  font-size: 12.5px;
}
</style>
