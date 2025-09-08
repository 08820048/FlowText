<script setup lang="ts">
import { ref, onMounted } from 'vue';
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

// 初始化设置
onMounted(() => {
  settingsStore.initSettings();
  themeManager.init();
  settingsStore.applyTheme();
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
    </div>
    
    <!-- 主内容区 -->
    <div class="main-content">
      <!-- 左侧面板 -->
      <div class="left-panel">
        <VideoImport />
        <VideoPlayer />
      </div>
      
      <!-- 右侧面板 -->
      <div class="right-panel">
        <el-tabs v-model="activePanel" class="right-tabs">
          <el-tab-pane label="语音识别" name="recognition">
            <RecognitionPanel />
          </el-tab-pane>
          
          <el-tab-pane label="字幕编辑" name="subtitle">
            <SubtitleEditor />
          </el-tab-pane>
          
          <el-tab-pane label="设置" name="settings">
            <SettingsPanel />
          </el-tab-pane>
        </el-tabs>
      </div>
    </div>
    
    <!-- 任务状态栏 -->
    <TaskStatusBar />
  </div>
</template>

<style scoped>
.main-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100%;
  overflow: hidden;
  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
}

.app-header {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  padding: 12px 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
  backdrop-filter: blur(10px);
}

.app-title {
  display: flex;
  align-items: baseline;
}

.app-title h1 {
  margin: 0;
  font-size: 22px;
  font-weight: 700;
  letter-spacing: 0.5px;
}

.app-subtitle {
  margin-left: 10px;
  font-size: 14px;
  opacity: 0.9;
  font-weight: 300;
}

.main-content {
  display: flex;
  flex: 1;
  overflow: hidden;
  gap: 1px;
  padding: 8px;
}

.left-panel {
  width: 60%;
  display: flex;
  flex-direction: column;
  background: white;
  border-radius: 12px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
  overflow: hidden;
  transition: all 0.3s ease;
}

.left-panel:hover {
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.12);
}

.left-panel > :first-child {
  border-bottom: 1px solid #f0f2f5;
}

.right-panel {
  width: 40%;
  background: white;
  border-radius: 12px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
  overflow: hidden;
  transition: all 0.3s ease;
}

.right-panel:hover {
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.12);
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
    gap: 8px;
  }
  
  .left-panel,
  .right-panel {
    width: 100%;
  }
  
  .left-panel {
    height: 50%;
  }
  
  .right-panel {
    height: 50%;
  }
}

@media (max-width: 768px) {
  .app-header {
    padding: 8px 12px;
  }
  
  .app-title h1 {
    font-size: 18px;
  }
  
  .app-subtitle {
    font-size: 12px;
  }
  
  .main-content {
    padding: 4px;
  }
}

:deep(.el-tabs__content) {
  flex: 1;
  overflow: auto;
  padding: 0;
}

:deep(.el-tabs__header) {
  margin: 0;
  background: linear-gradient(135deg, #f8f9fa 0%, #e9ecef 100%);
  border-bottom: 1px solid #dee2e6;
  border-radius: 12px 12px 0 0;
}

:deep(.el-tabs__nav-wrap) {
  padding: 0 20px;
}

:deep(.el-tabs__item) {
  font-weight: 500;
  transition: all 0.3s ease;
}

:deep(.el-tabs__item:hover) {
  color: #667eea;
}

:deep(.el-tabs__item.is-active) {
  color: #667eea;
  font-weight: 600;
}

:deep(.el-tabs__active-bar) {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  height: 3px;
}

:deep(.el-tab-pane) {
  height: 100%;
  overflow: auto;
}
</style>