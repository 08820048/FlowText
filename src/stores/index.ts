import { createPinia } from 'pinia';
import { useVideoStore } from './videoStore';
import { useSettingsStore } from './settingsStore';

// 创建Pinia实例
const pinia = createPinia();

// 导出Pinia实例和存储
export {
  pinia,
  useVideoStore,
  useSettingsStore
};