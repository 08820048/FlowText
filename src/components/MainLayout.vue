<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { Microphone, Edit, Setting, ArrowRight } from '@element-plus/icons-vue';
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

// 右侧面板是否展开
const rightPanelExpanded = ref(true);

// 切换右侧面板展开状态
function toggleRightPanel(panelName?: string) {
  if (panelName) {
    if (activePanel.value === panelName && rightPanelExpanded.value) {
      // 如果点击的是当前激活的面板且已展开，则收起
      rightPanelExpanded.value = false;
    } else {
      // 否则切换到该面板并展开
      activePanel.value = panelName;
      rightPanelExpanded.value = true;
    }
  } else {
    // 如果没有指定面板，则切换展开状态
    rightPanelExpanded.value = !rightPanelExpanded.value;
  }
}

// 初始化设置
onMounted(() => {
  settingsStore.initSettings();
  themeManager.init();
});
</script>

<template>
  <div class="main-layout">
    <!-- 顶部工具栏 -->
    <div class="app-header">
      <!-- 预留空间用于未来功能 -->
      <div class="header-actions">
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
        <!-- 侧边栏图标区域 -->
        <div class="sidebar-icons">
          <div
            class="sidebar-icon"
            :class="{ active: activePanel === 'recognition' }"
            @click="toggleRightPanel('recognition')"
            title="语音识别"
          >
            <el-icon size="20"><Microphone /></el-icon>
          </div>
          <div
            class="sidebar-icon"
            :class="{ active: activePanel === 'subtitle' }"
            @click="toggleRightPanel('subtitle')"
            title="字幕编辑"
          >
            <el-icon size="20"><Edit /></el-icon>
          </div>
          <div
            class="sidebar-icon"
            :class="{ active: activePanel === 'settings' }"
            @click="toggleRightPanel('settings')"
            title="设置"
          >
            <el-icon size="20"><Setting /></el-icon>
          </div>
        </div>

        <!-- 面板内容区域 -->
        <div class="panel-content" v-show="rightPanelExpanded">
          <div class="panel-header">
            <span class="panel-title">
              {{ activePanel === 'recognition' ? '语音识别' :
                 activePanel === 'subtitle' ? '字幕编辑' : '设置' }}
            </span>
            <el-icon class="collapse-icon" @click="toggleRightPanel()" size="16">
              <ArrowRight />
            </el-icon>
          </div>

          <div class="panel-body">
            <RecognitionPanel v-if="activePanel === 'recognition'" />
            <SubtitleEditor v-if="activePanel === 'subtitle'" />
            <SettingsPanel v-if="activePanel === 'settings'" />
          </div>
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
  background: #ffffff;
  color: #0fdc78;
  padding: 8px 16px;
  display: flex;
  justify-content: flex-end;
  align-items: center;
  border-bottom: 2px solid #0fdc78;
  min-height: 40px;
}

.header-actions {
  display: flex;
  align-items: center;
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
  display: flex;
  background: #ffffff;
  overflow: hidden;
  transition: all 0.3s ease;
  min-width: 48px; /* 侧边栏图标宽度 */
}

/* 侧边栏图标区域 */
.sidebar-icons {
  width: 48px;
  background: #f8fafc;
  border-left: 1px solid #e2e8f0;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 8px 0;
  gap: 4px;
}

.sidebar-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s ease;
  color: #64748b;
}

.sidebar-icon:hover {
  background: rgba(15, 220, 120, 0.1);
  color: #0fdc78;
}

.sidebar-icon.active {
  background: #0fdc78;
  color: #ffffff;
}

/* 面板内容区域 */
.panel-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  width: 320px;
  border-left: 1px solid #e2e8f0;
}

.panel-header {
  height: 40px;
  background: #f8fafc;
  border-bottom: 1px solid #e2e8f0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
}

.panel-title {
  font-size: 13px;
  font-weight: 600;
  color: #374151;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.collapse-icon {
  cursor: pointer;
  color: #64748b;
  transition: all 0.2s ease;
}

.collapse-icon:hover {
  color: #0fdc78;
}

.panel-body {
  flex: 1;
  overflow: hidden;
}




/* 响应式设计 */
@media (max-width: 1200px) {
  .main-content {
    flex-direction: column;
    gap: 2px;
  }

  .left-panel {
    width: 100% !important;
    height: 50%;
    border-right: none;
    border-bottom: 2px solid #0fdc78;
  }

  .right-panel {
    width: 100% !important;
    height: 50%;
    min-width: unset;
  }

  .panel-content {
    width: auto;
  }

  .sidebar-icons {
    flex-direction: row;
    width: 100%;
    height: 48px;
    padding: 0 8px;
    justify-content: flex-start;
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