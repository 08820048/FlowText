import { createApp } from "vue";
import App from "./App.vue";
import { pinia } from "./stores";
import ElementPlus from "element-plus";
import "element-plus/dist/index.css";
import * as ElementPlusIconsVue from "@element-plus/icons-vue";

// 创建Vue应用实例
const app = createApp(App);

// 注册Element Plus图标
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component);
}

// 使用插件
app.use(pinia);
app.use(ElementPlus);

// 挂载应用
app.mount("#app");
