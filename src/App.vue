<template>
  <teleport to="body">
    <top-bar />
  </teleport>
  <el-scrollbar wrap-class="root-wrap" view-class="root-view">
    <tabs />
  </el-scrollbar>
</template>

<script setup lang="ts">
import TopBar from "./views/top_bar.vue";
import Tabs from "./views/tabs.vue";
</script>


<style>
#root {
  box-sizing: border-box;
  width: 100%;
  height: var(--body-height);
  margin-top: var(--top-bar-height);
}
.root-wrap {
  box-sizing: border-box;
  width: 100%;
  padding: 0.3rem 1rem 1rem;
  overflow: auto;

}
.root-view {
  display: flex;
  flex-direction: column;
  gap: var(--gap-size);
}
</style>