<script setup lang="ts">
import MainLayout from "./components/MainLayout.vue";
</script>

<template>
  <MainLayout />
</template>

<style>
/* 扁平化全局样式 */
:root {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #000000;
  background-color: #ffffff;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  /* 扁平化设计 - 移除阴影效果 */
}

.row {
  display: flex;
  justify-content: center;
}

a {
  font-weight: 500;
  color: #0fdc78;
  text-decoration: inherit;
}

a:hover {
  color: #0bc96a;
}

h1 {
  text-align: center;
}

/* 扁平化输入框和按钮样式 */
input,
button {
  border: 2px solid #0fdc78;
  border-radius: 0;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #000000;
  background-color: #ffffff;
}

button {
  cursor: pointer;
  background-color: #0fdc78;
  color: #ffffff;
}

button:hover {
  background-color: #ffffff;
  border-color: #0fdc78;
  color: #0fdc78;
}

button:active {
  background-color: #0fdc78;
  color: #ffffff;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

/* 深色主题样式已移除，仅支持浅色主题 */

</style>