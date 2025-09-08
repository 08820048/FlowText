<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { Setting, Close } from '@element-plus/icons-vue';
import { useSettingsStore } from '../stores';
import { themeManager } from '../utils/themeManager';
import VideoImport from './VideoImport.vue';
import VideoPlayer from './VideoPlayer.vue';
import SubtitleEditor from './SubtitleEditor.vue';
import RecognitionPanel from './RecognitionPanel.vue';
import SettingsPanel from './SettingsPanel.vue';
import TaskStatusBar from './TaskStatusBar.vue';

// 引入设置存储
const settingsStore = useSettingsStore();

// 当前激活的面板
const activePanel = ref('recognition');

// 设置侧边栏显示状态
const showSettingsSidebar = ref(false);

// 切换设置侧边栏
const toggleSettingsSidebar = () => {
  showSettingsSidebar.value = !showSettingsSidebar.value;
};

// 初始化设置
onMounted(() => {
  settingsStore.initSettings();
  themeManager.init();
});
</script>

<template>
  <div class="main-layout">
    <!-- 顶部标题栏 -->
    <div class="app-header">
      <div class="app-title">
        <h1>FlowText</h1>
        <span class="app-subtitle">智能视频字幕提取工具</span>
      </div>
      
      <!-- 右侧设置按钮 -->
      <div class="header-actions">
        <el-button 
          @click="toggleSettingsSidebar"
          :type="showSettingsSidebar ? 'primary' : 'default'"
          class="settings-btn"
        >
          <el-icon><Setting /></el-icon>
          设置
        </el-button>
      </div>
    </div>
    
    <!-- 主内容区 -->
    <div class="main-content">
      <!-- 左侧面板 -->
      <div class="left-panel">
        <VideoImport />
        <VideoPlayer />
      </div>
      
      <!-- 右侧面板 -->
      <div class="right-panel" :class="{ 'with-sidebar': showSettingsSidebar }">
        <el-tabs v-model="activePanel" class="right-tabs">
          <el-tab-pane label="语音识别" name="recognition">
            <RecognitionPanel />
          </el-tab-pane>
          
          <el-tab-pane label="字幕编辑" name="subtitle">
            <SubtitleEditor />
          </el-tab-pane>
        </el-tabs>
      </div>
      
      <!-- 设置侧边栏 -->
      <div v-if="showSettingsSidebar" class="settings-sidebar">
        <div class="sidebar-header">
          <h3>设置</h3>
          <el-button 
            @click="toggleSettingsSidebar"
            type="text"
            class="close-btn"
          >
            <el-icon><Close /></el-icon>
          </el-button>
        </div>
        <div class="sidebar-content">
          <SettingsPanel />
        </div>
      </div>
    </div>
    
    <!-- 任务状态栏 -->
    <TaskStatusBar />
  </div>
</template>

<style scoped>
/* 扁平化UI风格 - 主题色：#0fdc78(绿色)、#000000(黑色)、#ffffff(白色) */
.main-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100%;
  overflow: hidden;
  background: #ffffff;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

.app-header {
  background: #000000;
  color: #ffffff;
  padding: 16px 24px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 2px solid #0fdc78;
}

.app-title {
  display: flex;
  align-items: baseline;
}

.app-title h1 {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  letter-spacing: 0.5px;
  color: #0fdc78;
}

.app-subtitle {
  margin-left: 12px;
  font-size: 14px;
  color: #ffffff;
  font-weight: 400;
}

.header-actions {
  display: flex;
  align-items: center;
}

.settings-btn {
  background: transparent !important;
  border: 2px solid #0fdc78 !important;
  color: #0fdc78 !important;
  font-weight: 500;
}

.settings-btn:hover {
  background: #0fdc78 !important;
  color: #000000 !important;
}

.settings-btn.el-button--primary {
  background: #0fdc78 !important;
  color: #000000 !important;
}

.settings-btn.el-button--primary:hover {
  background: #ffffff !important;
  color: #0fdc78 !important;
}

.main-content {
  display: flex;
  flex: 1;
  overflow: hidden;
  gap: 2px;
  padding: 0;
  background: #f8f9fa;
}

.left-panel {
  width: 60%;
  display: flex;
  flex-direction: column;
  background: #ffffff;
  border-right: 2px solid #0fdc78;
  overflow: hidden;
}

.left-panel > :first-child {
  border-bottom: 1px solid #e9ecef;
}

.right-panel {
  width: 40%;
  background: #ffffff;
  overflow: hidden;
  transition: width 0.3s ease;
}

.right-panel.with-sidebar {
  width: 25%;
}

/* 设置侧边栏样式 */
.settings-sidebar {
  width: 35%;
  background: #ffffff;
  border-left: 2px solid #0fdc78;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  animation: slideIn 0.3s ease;
}

@keyframes slideIn {
  from {
    width: 0;
    opacity: 0;
  }
  to {
    width: 35%;
    opacity: 1;
  }
}

.sidebar-header {
  background: #000000;
  color: #ffffff;
  padding: 16px 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 2px solid #0fdc78;
}

.sidebar-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #0fdc78;
}

.close-btn.el-button--text {
  color: #0fdc78 !important;
  padding: 4px !important;
  background: transparent !important;
}

.close-btn.el-button--text:hover {
  color: #0fdc78 !important;
  background: transparent !important;
}

.sidebar-content {
  flex: 1;
  overflow: auto;
  padding: 0;
}

.right-tabs {
  height: 100%;
  display: flex;
  flex-direction: column;
}

/* 响应式设计 */
@media (max-width: 1200px) {
  .main-content {
    flex-direction: column;
    gap: 2px;
  }
  
  .left-panel,
  .right-panel {
    width: 100% !important;
    border-right: none;
  }
  
  .left-panel {
    height: 50%;
    border-bottom: 2px solid #0fdc78;
  }
  
  .right-panel {
    height: 50%;
  }
  
  .settings-sidebar {
    position: fixed;
    top: 0;
    right: 0;
    width: 100% !important;
    height: 100vh;
    z-index: 1000;
    animation: slideInMobile 0.3s ease;
  }
  
  @keyframes slideInMobile {
    from {
      transform: translateX(100%);
    }
    to {
      transform: translateX(0);
    }
  }
}

@media (max-width: 768px) {
  .app-header {
    padding: 12px 16px;
  }
  
  .app-title h1 {
    font-size: 20px;
  }
  
  .app-subtitle {
    font-size: 12px;
  }
}

/* Element Plus 扁平化样式覆盖 */
:deep(.el-tabs__content) {
  flex: 1;
  overflow: auto;
  padding: 0;
}

:deep(.el-tabs__header) {
  margin: 0;
  background: #000000;
  border-bottom: 2px solid #0fdc78;
}

:deep(.el-tabs__nav-wrap) {
  padding: 0 24px;
}

:deep(.el-tabs__item) {
  font-weight: 500;
  color: #ffffff;
  border: none;
  padding: 16px 20px;
}

:deep(.el-tabs__item:hover) {
  color: #0fdc78;
  background: rgba(15, 220, 120, 0.1);
}

:deep(.el-tabs__item.is-active) {
  color: #0fdc78;
  font-weight: 600;
  background: rgba(15, 220, 120, 0.15);
}

:deep(.el-tabs__active-bar) {
  background: #0fdc78;
  height: 3px;
}

:deep(.el-tab-pane) {
  height: 100%;
  overflow: auto;
  background: #ffffff;
}

/* 扁平化按钮样式 */
:deep(.el-button) {
  border: 2px solid #0fdc78;
  font-weight: 500;
}

:deep(.el-button--primary) {
  background: #0fdc78;
  border-color: #0fdc78;
  color: #000000;
}

:deep(.el-button--primary:hover) {
  background: #000000;
  border-color: #000000;
  color: #0fdc78;
}

:deep(.el-button:not(.el-button--primary)) {
  background: #ffffff;
  color: #000000;
}

:deep(.el-button:not(.el-button--primary):hover) {
  background: #0fdc78;
  color: #000000;
}
</style>