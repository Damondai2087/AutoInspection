<script setup lang="ts">
import { store } from "../store/app";
import { Check, Warning } from "@element-plus/icons-vue";
</script>

<template>
  <div
    v-if="store.permission.message"
    class="perm-banner"
    :class="store.permission.is_elevated ? 'ok' : 'warn'"
  >
    <el-icon :size="15">
      <Check v-if="store.permission.is_elevated" />
      <Warning v-else />
    </el-icon>
    <span>{{ store.permission.message }}</span>
  </div>
</template>

<style scoped>
.perm-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 12px;
  border-radius: var(--app-radius-sm);
  font-size: 13px;
  line-height: 1.5;
  margin-top: 16px;
}

.perm-banner.ok {
  background: var(--app-success-soft);
  color: var(--app-success);
  border: 1px solid color-mix(in srgb, var(--app-success) 20%, transparent);
}

.perm-banner.warn {
  background: var(--app-warn-soft);
  color: var(--app-warn);
  border: 1px solid color-mix(in srgb, var(--app-warn) 20%, transparent);
}
</style>
