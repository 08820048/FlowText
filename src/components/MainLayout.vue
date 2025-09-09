<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
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
// 拖拽时的临时宽度（用于优化性能）
const dragTempWidth = ref(60);
// 动画帧ID
let animationFrameId: number | null = null;

// 切换右侧面板展开状态
function toggleRightPanel(panelName?: string) {
  if (panelName) {
    if (activePanel.value === panelName && rightPanelExpanded.value && !sidebarCollapsed.value) {
      // 点击当前激活面板 → 收起侧边栏
      sidebarCollapsed.value = true;
      rightPanelExpanded.value = false;
    } else {
      // 点击其他面板或收起状态 → 切换并展开
      activePanel.value = panelName;
      rightPanelExpanded.value = true;
      sidebarCollapsed.value = false;
    }
  } else {
    // 没有指定面板名称，直接切换展开状态
    rightPanelExpanded.value = !rightPanelExpanded.value;
  }
}

// 切换侧边栏收起状态
function toggleSidebar() {
  sidebarCollapsed.value = !sidebarCollapsed.value;
  if (sidebarCollapsed.value) {
    rightPanelExpanded.value = false;
  } else {
    // 展开侧边栏时，自动展开面板
    rightPanelExpanded.value = true;
  }
}

// 开始拖拽分割线
function startDrag(event: MouseEvent) {
  if (sidebarCollapsed.value) return;

  isDragging.value = true;
  dragTempWidth.value = leftPanelWidth.value;

  // 添加全局样式以优化拖拽性能
  document.body.style.userSelect = 'none';
  document.body.style.cursor = 'col-resize';

  document.addEventListener('mousemove', onDrag, { passive: true });
  document.addEventListener('mouseup', stopDrag);
  event.preventDefault();
}

// 拖拽过程中 - 使用requestAnimationFrame优化
function onDrag(event: MouseEvent) {
  if (!isDragging.value) return;

  // 取消之前的动画帧
  if (animationFrameId) {
    cancelAnimationFrame(animationFrameId);
  }

  // 使用requestAnimationFrame确保流畅渲染
  animationFrameId = requestAnimationFrame(() => {
    const container = document.querySelector('.main-content') as HTMLElement;
    if (!container) return;

    const containerRect = container.getBoundingClientRect();
    const mouseX = event.clientX - containerRect.left;
    const containerWidth = containerRect.width;

    // 计算新的左侧面板宽度百分比
    let newWidth = (mouseX / containerWidth) * 100;

    // 限制最小和最大宽度
    newWidth = Math.max(20, Math.min(80, newWidth));

    // 更新临时宽度用于实时显示
    dragTempWidth.value = newWidth;

    // 直接操作CSS变量以获得最佳性能
    const leftPanel = document.querySelector('.left-panel') as HTMLElement;
    const rightPanel = document.querySelector('.right-panel') as HTMLElement;

    if (leftPanel && rightPanel) {
      leftPanel.style.width = `${newWidth}%`;
      rightPanel.style.width = `${100 - newWidth}%`;
    }
  });
}

// 停止拖拽
function stopDrag() {
  isDragging.value = false;

  // 清理动画帧
  if (animationFrameId) {
    cancelAnimationFrame(animationFrameId);
    animationFrameId = null;
  }

  // 恢复全局样式
  document.body.style.userSelect = '';
  document.body.style.cursor = '';

  // 更新最终宽度
  leftPanelWidth.value = dragTempWidth.value;

  document.removeEventListener('mousemove', onDrag);
  document.removeEventListener('mouseup', stopDrag);
}

// 初始化设置
onMounted(() => {
  settingsStore.initSettings();
  themeManager.init();
});

// 组件卸载时清理
onUnmounted(() => {
  // 清理拖拽相关的事件监听器和动画帧
  if (animationFrameId) {
    cancelAnimationFrame(animationFrameId);
  }
  document.removeEventListener('mousemove', onDrag);
  document.removeEventListener('mouseup', stopDrag);

  // 恢复全局样式
  document.body.style.userSelect = '';
  document.body.style.cursor = '';
});
</script>

<template>
  <div class="main-layout">
    <!-- 自定义标题栏 -->
    <div class="custom-titlebar" data-tauri-drag-region>
      <!-- 右侧功能图标 -->
      <div class="titlebar-actions">
        <div
          class="titlebar-icon"
          :class="{ active: activePanel === 'recognition' }"
          @click="toggleRightPanel('recognition')"
          :title="sidebarCollapsed ? '展开侧边栏' : '语音识别'"
        >
          <el-icon size="16"><Microphone /></el-icon>
        </div>
        <div
          class="titlebar-icon"
          :class="{ active: activePanel === 'subtitle' }"
          @click="toggleRightPanel('subtitle')"
          :title="sidebarCollapsed ? '展开侧边栏' : '字幕编辑'"
        >
          <el-icon size="16"><Edit /></el-icon>
        </div>
        <div
          class="titlebar-icon"
          :class="{ active: activePanel === 'settings' }"
          @click="toggleRightPanel('settings')"
          :title="sidebarCollapsed ? '展开侧边栏' : '设置'"
        >
          <el-icon size="16"><Setting /></el-icon>
        </div>
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
        :style="{ width: sidebarCollapsed ? '0' : `${100 - leftPanelWidth}%` }"
        v-show="!sidebarCollapsed"
      >
        <!-- 面板内容区域 -->
        <div class="panel-content" v-show="rightPanelExpanded">
          <div class="panel-header">
            <span class="panel-title">
              {{ activePanel === 'recognition' ? '语音识别' :
                 activePanel === 'subtitle' ? '字幕编辑' : '设置' }}
            </span>
            <el-icon class="collapse-icon" @click="toggleSidebar" size="16" title="收起侧边栏">
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

/* 自定义标题栏 */
.custom-titlebar {
  height: 32px;
  background: transparent;
  display: flex;
  justify-content: flex-end;
  align-items: center;
  padding: 0 16px;
  /* 确保在系统标题栏下方 */
  position: relative;
  z-index: 1000;
  /* 允许拖拽窗口 */
  -webkit-app-region: drag;
}

.titlebar-actions {
  display: flex;
  gap: 6px;
  align-items: center;
  /* 防止拖拽区域影响按钮点击 */
  -webkit-app-region: no-drag;
  /* 确保图标不会被顶到界面外 */
  margin-right: 0;
}

.titlebar-icon {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  border-radius: 3px;
  transition: all 0.15s ease;
  color: #64748b;
}

.titlebar-icon:hover {
  background: rgba(15, 220, 120, 0.1);
  color: #0fdc78;
}

.titlebar-icon.active {
  background: #0fdc78;
  color: #ffffff;
}





.main-content {
  display: flex;
  flex: 1;
  overflow: hidden;
  padding: 0;
  background: #f8f9fa;
  position: relative;
  /* 启用硬件加速 */
  transform: translateZ(0);
  will-change: auto;
}

.main-content.dragging {
  cursor: col-resize;
  user-select: none;
  /* 拖拽时启用硬件加速 */
  will-change: contents;
}

.main-content.dragging * {
  pointer-events: none;
}

.left-panel {
  display: flex;
  flex-direction: column;
  background: #ffffff;
  overflow: hidden;
  min-width: 200px;
  /* 优化拖拽性能 */
  transform: translateZ(0);
  backface-visibility: hidden;
  /* 只在非拖拽时启用过渡动画 */
  transition: width 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.left-panel.full-width {
  width: 100% !important;
}

/* 拖拽时禁用过渡动画以提升性能 */
.main-content.dragging .left-panel,
.main-content.dragging .right-panel {
  transition: none !important;
  will-change: width;
}

/* 可拖拽的分割线 */
.resize-handle {
  width: 4px;
  background: transparent;
  cursor: col-resize;
  position: relative;
  flex-shrink: 0;
  /* 硬件加速 */
  transform: translateZ(0);
  /* 增加点击区域 */
  z-index: 10;
}

.resize-handle:hover {
  background: #0fdc78;
  /* 悬停时的过渡效果 */
  transition: background-color 0.15s ease;
}

.resize-handle:active {
  background: #0bc96a;
}

/* 扩大点击区域 */
.resize-handle::before {
  content: '';
  position: absolute;
  left: -4px;
  right: -4px;
  top: 0;
  bottom: 0;
  background: transparent;
  cursor: col-resize;
}

/* 拖拽时的视觉反馈 */
.main-content.dragging .resize-handle {
  background: #0fdc78;
  box-shadow: 0 0 0 1px rgba(15, 220, 120, 0.3);
}

.left-panel > :first-child {
  border-bottom: 1px solid #e9ecef;
}

.right-panel {
  display: flex;
  background: #ffffff;
  overflow: hidden;
  height: 100%;
  /* 优化拖拽性能 */
  transform: translateZ(0);
  backface-visibility: hidden;
  /* 只在非拖拽时启用过渡动画 */
  transition: width 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.right-panel.collapsed {
  width: 0 !important;
  min-width: 0;
  max-width: 0;
}

/* 面板内容区域 */
.panel-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 250px;
  overflow: hidden;
  width: 100%;
  height: 100%;
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
  overflow-y: auto;
  overflow-x: hidden;
  height: 0; /* 强制flex子元素计算高度 */
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

  .header-actions {
    gap: 4px;
  }

  .header-icon {
    width: 24px;
    height: 24px;
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