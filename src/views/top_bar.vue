
<script setup lang="ts">
  import {appWindow} from "@tauri-apps/api/window";
  import {computed, ref} from "vue";
  const min = () => appWindow.minimize()
  const max = () => appWindow.toggleMaximize()
  const close = () => appWindow.close()

  const FiMaximize_max = '<svg fill="none" stroke-width="2" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" viewBox="0 0 24 24" height="1em" width="1em" style="overflow: visible; color: currentcolor;"><path d="M8 3v3a2 2 0 0 1-2 2H3m18 0h-3a2 2 0 0 1-2-2V3m0 18v-3a2 2 0 0 1 2-2h3M3 16h3a2 2 0 0 1 2 2v3"></path></svg>'
  const FiMaximize_min = '<svg fill="none" stroke-width="2" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" viewBox="0 0 24 24" height="1em" width="1em" style="overflow: visible; color: currentcolor;"><path d="M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3"></path></svg>'
  const FiMinimize = '<svg fill="none" stroke-width="2" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" viewBox="0 0 24 24" height="1em" width="1em" style="overflow: visible; color: currentcolor;"><path d="M5 12 19 12"></path></svg>'
  const FiX = '<svg fill="none" stroke-width="2" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" viewBox="0 0 24 24" height="1em" width="1em" style="overflow: visible; color: currentcolor;"><path d="M18 6 6 18"></path><path d="M6 6 18 18"></path></svg>'

  let isMax = ref(false)
  const max_style = computed(() => isMax.value ? "height: 0" : "height: 5px");
  const max_icon = computed(() => !isMax.value ? FiMaximize_min : FiMaximize_max)
  appWindow.onResized(() => {
    appWindow.isMaximized().then((res) => isMax.value = res)
    console.debug(isMax)
  })

</script>

<template>
  <div data-tauri-drag-region class="title-bar" >
    <canvas class="title-bar-drag-line" v-bind:style="max_style" />
    <div>
      <button class="title-bar-button" @click="min" v-html="FiMinimize" />
      <button class="title-bar-button" @click="max" v-html="max_icon" />
      <button class="title-bar-button" @click="close" v-html="FiX" />
    </div>
  </div>
</template>

<style>
.title-bar {
  height: var(--top-bar-height);
  width: 100%;
  background: white;
  user-select: none;
  display: flex;
  justify-content: space-between;
  position: absolute;
  top: 0;
}
.title-bar-button {
  display: inline-flex;
  justify-content: center;
  align-items: center;
  width: 30px;
  height: 30px;
  background: transparent;
  border: none;
}
.title-bar-button:hover {
  background: rgba(0, 0, 0, 0.07);
  border: none;
}
.title-bar-drag-line {
  width: 100%;
  flex: 1;
  user-select: none;
  background: transparent;
  border: none;
}
</style>
