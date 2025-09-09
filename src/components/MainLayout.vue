<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { Microphone, Edit, Setting, ArrowRight, ArrowLeft } from '@element-plus/icons-vue';
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
// 侧边栏是否收起
const sidebarCollapsed = ref(false);

// 左侧面板宽度（百分比）
const leftPanelWidth = ref(60);
// 是否正在拖拽分割线
const isDragging = ref(false);

// 切换右侧面板展开状态
function toggleRightPanel(panelName?: string) {
  if (panelName) {
    // 如果侧边栏收起，先展开侧边栏
    if (sidebarCollapsed.value) {
      sidebarCollapsed.value = false;
      activePanel.value = panelName;
      rightPanelExpanded.value = true;
      return;
    }

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

// 切换侧边栏收起状态
function toggleSidebar() {
  sidebarCollapsed.value = !sidebarCollapsed.value;
  if (sidebarCollapsed.value) {
    rightPanelExpanded.value = false;
  }
}

// 开始拖拽分割线
function startDrag(event: MouseEvent) {
  if (sidebarCollapsed.value) return;

  isDragging.value = true;
  document.addEventListener('mousemove', onDrag);
  document.addEventListener('mouseup', stopDrag);
  event.preventDefault();
}

// 拖拽过程中
function onDrag(event: MouseEvent) {
  if (!isDragging.value) return;

  const container = document.querySelector('.main-content') as HTMLElement;
  if (!container) return;

  const containerRect = container.getBoundingClientRect();
  const mouseX = event.clientX - containerRect.left;
  const containerWidth = containerRect.width;

  // 计算新的左侧面板宽度百分比
  let newWidth = (mouseX / containerWidth) * 100;

  // 限制最小和最大宽度
  newWidth = Math.max(20, Math.min(80, newWidth));

  leftPanelWidth.value = newWidth;
}

// 停止拖拽
function stopDrag() {
  isDragging.value = false;
  document.removeEventListener('mousemove', onDrag);
  document.removeEventListener('mouseup', stopDrag);
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
    <div class="main-content" :class="{ dragging: isDragging }">
      <!-- 左侧面板 -->
      <div
        class="left-panel"
        :class="{ 'full-width': sidebarCollapsed }"
        :style="{ width: sidebarCollapsed ? '100%' : `${leftPanelWidth}%` }"
      >
        <VideoImport />
        <VideoPlayer />
      </div>

      <!-- 可拖拽的分割线 -->
      <div
        class="resize-handle"
        v-show="!sidebarCollapsed"
        @mousedown="startDrag"
      ></div>

      <!-- 右侧面板 -->
      <div
        class="right-panel"
        :class="{ collapsed: sidebarCollapsed }"
        :style="{ width: sidebarCollapsed ? '48px' : `${100 - leftPanelWidth}%` }"
      >
        <!-- 侧边栏图标区域 -->
        <div class="sidebar-icons">
          <div
            class="sidebar-icon"
            :class="{ active: activePanel === 'recognition' }"
            @click="toggleRightPanel('recognition')"
            :title="sidebarCollapsed ? '展开侧边栏' : '语音识别'"
          >
            <el-icon size="20"><Microphone /></el-icon>
          </div>
          <div
            class="sidebar-icon"
            :class="{ active: activePanel === 'subtitle' }"
            @click="toggleRightPanel('subtitle')"
            :title="sidebarCollapsed ? '展开侧边栏' : '字幕编辑'"
          >
            <el-icon size="20"><Edit /></el-icon>
          </div>
          <div
            class="sidebar-icon"
            :class="{ active: activePanel === 'settings' }"
            @click="toggleRightPanel('settings')"
            :title="sidebarCollapsed ? '展开侧边栏' : '设置'"
          >
            <el-icon size="20"><Setting /></el-icon>
          </div>

          <!-- 分隔线 -->
          <div class="sidebar-divider"></div>

          <!-- 收起/展开按钮 -->
          <div
            class="sidebar-icon collapse-sidebar-btn"
            @click="toggleSidebar"
            :title="sidebarCollapsed ? '展开侧边栏' : '收起侧边栏'"
          >
            <el-icon size="16">
              <ArrowLeft v-if="!sidebarCollapsed" />
              <ArrowRight v-else />
            </el-icon>
          </div>
        </div>

        <!-- 面板内容区域 -->
        <div class="panel-content" v-show="rightPanelExpanded && !sidebarCollapsed">
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
  padding: 0;
  background: #f8f9fa;
  position: relative;
}

.main-content.dragging {
  cursor: col-resize;
  user-select: none;
}

.main-content.dragging * {
  pointer-events: none;
}

.left-panel {
  display: flex;
  flex-direction: column;
  background: #ffffff;
  overflow: hidden;
  transition: width 0.3s ease;
  min-width: 200px;
}

.left-panel.full-width {
  width: 100% !important;
}

/* 可拖拽的分割线 */
.resize-handle {
  width: 4px;
  background: transparent;
  cursor: col-resize;
  position: relative;
  flex-shrink: 0;
}

.resize-handle:hover {
  background: #0fdc78;
}

.resize-handle::before {
  content: '';
  position: absolute;
  left: -2px;
  right: -2px;
  top: 0;
  bottom: 0;
  background: transparent;
}

.left-panel > :first-child {
  border-bottom: 1px solid #e9ecef;
}

.right-panel {
  display: flex;
  background: #ffffff;
  overflow: hidden;
  transition: width 0.3s ease;
  min-width: 48px; /* 侧边栏图标宽度 */
}

.right-panel.collapsed {
  width: 48px !important;
  min-width: 48px;
  max-width: 48px;
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

.sidebar-divider {
  width: 32px;
  height: 1px;
  background: #e2e8f0;
  margin: 8px 4px;
}

.collapse-sidebar-btn {
  margin-top: auto;
  margin-bottom: 8px;
}

/* 面板内容区域 */
.panel-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 250px;
  border-left: 1px solid #e2e8f0;
  overflow: hidden;
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
  }

  .left-panel {
    width: 100% !important;
    height: 50%;
    border-right: none;
    border-bottom: 2px solid #0fdc78;
  }

  .resize-handle {
    display: none;
  }

  .right-panel {
    width: 100% !important;
    height: 50%;
    min-width: unset;
  }

  .panel-content {
    min-width: unset;
  }

  .sidebar-icons {
    flex-direction: row;
    width: 100%;
    height: 48px;
    padding: 0 8px;
    justify-content: flex-start;
  }

  .collapse-sidebar-btn {
    margin-top: 0;
    margin-left: auto;
    margin-bottom: 0;
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