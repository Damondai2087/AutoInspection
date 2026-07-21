<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import { store } from "../store/app";

const box = ref<HTMLElement | null>(null);

watch(
  () => store.logs.length,
  async () => {
    await nextTick();
    if (box.value) {
      box.value.scrollTop = box.value.scrollHeight;
    }
  }
);
</script>

<template>
  <div class="log-console" ref="box">
    <div v-if="store.logs.length === 0" class="log-empty">
      <div class="empty-icon">
        <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="4 17 10 11 4 5"/>
          <line x1="12" y1="19" x2="20" y2="19"/>
        </svg>
      </div>
      <div class="empty-title">等待执行日志</div>
      <div class="empty-desc">开始巡检后，执行过程将实时显示在这里</div>
    </div>
    <template v-else>
      <div
        v-for="(l, i) in store.logs"
        :key="i"
        class="log-line"
        :class="'lv-' + l.level"
      >
        <span class="log-ts">{{ l.ts }}</span>
        <span class="log-text">{{ l.text }}</span>
      </div>
    </template>
  </div>
</template>

<style scoped>
.log-console {
  height: 100%;
  overflow-y: auto;
  background: var(--app-log-bg);
  border-radius: var(--app-radius-sm);
  padding: 10px 12px;
  font-family: "Cascadia Code", "Consolas", "Menlo", "SF Mono", monospace;
  font-size: 12.5px;
  line-height: 1.7;
}

.log-empty {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  font-family: system-ui, -apple-system, sans-serif;
  color: var(--app-text-muted);
}

.empty-icon {
  color: var(--app-text-muted);
  opacity: 0.5;
  margin-bottom: 4px;
}

.empty-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--app-text-soft);
}

.empty-desc {
  font-size: 12px;
}

.log-line {
  white-space: pre-wrap;
  word-break: break-all;
}

.log-ts {
  margin-right: 8px;
  color: var(--app-log-ts);
  user-select: none;
}

.lv-info .log-text { color: var(--app-log-info); }
.lv-warn .log-text { color: var(--app-log-warn); font-weight: 500; }
.lv-error .log-text { color: var(--app-log-error); font-weight: 500; }
</style>
