<template>
  <div class="subtitle-editor">
    <div class="editor-header">
      <div class="header-actions">
        <el-button type="primary" size="small" @click="addNewSubtitle">
          添加字幕
        </el-button>
        <el-button size="small" @click="exportSubtitles">
          导出字幕
        </el-button>
      </div>
    </div>
    
    <div class="editor-content">
      <!-- 字幕列表 -->
      <div v-if="videoStore.subtitles.length > 0" class="subtitle-list">
        <div class="list-header">
          <div class="col-time">时间</div>
          <div class="col-text">字幕内容</div>
          <div class="col-actions">操作</div>
        </div>
        
        <div 
          v-for="subtitle in sortedSubtitles" 
          :key="subtitle.id"
          class="subtitle-item"
          :class="{ 'active': currentSubtitle?.id === subtitle.id }"
        >
          <div class="col-time">
            <div class="time-display">
              {{ formatTime(subtitle.startTime) }} - {{ formatTime(subtitle.endTime) }}
            </div>
            <div class="time-edit" v-if="editingId === subtitle.id">
              <el-input 
                v-model="editForm.startTime" 
                size="small" 
                placeholder="开始时间"
                class="time-input"
              />
              <span class="time-separator">-</span>
              <el-input 
                v-model="editForm.endTime" 
                size="small" 
                placeholder="结束时间"
                class="time-input"
              />
            </div>
          </div>
          
          <div class="col-text">
            <div v-if="editingId !== subtitle.id" class="text-display">
              {{ subtitle.text }}
            </div>
            <div v-else class="text-edit-container">
              <el-input
                v-model="editForm.text"
                type="textarea"
                :rows="4"
                placeholder="请输入字幕内容"
                class="text-input"
                :autosize="{ minRows: 3, maxRows: 8 }"
                resize="vertical"
                show-word-limit
                :maxlength="200"
                @keydown="handleTextareaKeydown"
                ref="textareaRef"
              />
              <div class="edit-tips">
                <span class="tip-text">支持多行文本，按 Ctrl+Enter 快速保存，Esc 取消编辑</span>
              </div>
            </div>
          </div>
          
          <div class="col-actions">
            <div v-if="editingId !== subtitle.id" class="action-buttons">
              <el-button size="small" @click="startEdit(subtitle)">
                编辑
              </el-button>
              <el-button size="small" @click="jumpToTime(subtitle.startTime)">
                跳转
              </el-button>
              <el-button size="small" type="danger" @click="deleteSubtitle(subtitle.id)">
                删除
              </el-button>
            </div>
            <div v-else class="edit-buttons">
              <el-button size="small" type="primary" @click="saveEdit()">
                保存
              </el-button>
              <el-button size="small" @click="cancelEdit()">
                取消
              </el-button>
            </div>
          </div>
        </div>
      </div>
      
      <!-- 空状态 -->
      <div v-else class="empty-state">
        <p>暂无字幕数据</p>
        <p>请先导入视频文件或添加字幕</p>
      </div>
    </div>
    
    <!-- 添加字幕对话框 -->
    <el-dialog v-model="showAddDialog" title="添加字幕" width="500px">
      <el-form :model="addForm" label-width="80px">
        <el-form-item label="开始时间">
          <el-input v-model="addForm.startTime" placeholder="格式：HH:MM:SS" />
        </el-form-item>
        <el-form-item label="结束时间">
          <el-input v-model="addForm.endTime" placeholder="格式：HH:MM:SS" />
        </el-form-item>
        <el-form-item label="字幕内容">
          <el-input 
            v-model="addForm.text" 
            type="textarea" 
            :rows="3" 
            placeholder="请输入字幕内容"
          />
        </el-form-item>
      </el-form>
      
      <template #footer>
        <el-button @click="showAddDialog = false">取消</el-button>
        <el-button type="primary" @click="confirmAddSubtitle">确定</el-button>
      </template>
    </el-dialog>
    
    <!-- 导出字幕对话框 -->
    <el-dialog v-model="showExportDialog" title="导出字幕" width="400px">
      <el-form :model="exportForm" label-width="80px">
        <el-form-item label="文件名">
          <el-input 
            v-model="exportForm.fileName" 
            placeholder="请输入文件名（不含扩展名）"
          />
        </el-form-item>
        <el-form-item label="格式">
          <el-select v-model="exportForm.format" style="width: 100%">
            <el-option label="SRT (SubRip)" value="srt" />
            <el-option label="VTT (WebVTT)" value="vtt" />
            <el-option label="ASS (Advanced SSA)" value="ass" />
            <el-option label="TXT (纯文本)" value="txt" />
            <el-option label="JSON" value="json" />
          </el-select>
        </el-form-item>
      </el-form>
      
      <template #footer>
        <el-button @click="showExportDialog = false">取消</el-button>
        <el-button type="primary" @click="confirmExportSubtitles">导出</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { useVideoStore, useSettingsStore } from '../stores';
import { ProgressMonitor } from '../utils/progressMonitor';
import { ErrorHandler, ErrorType, ErrorSeverity } from '../utils/errorHandler';
import type { Subtitle, SubtitleFormat } from '../types';

// 引入存储
const videoStore = useVideoStore();
const settingsStore = useSettingsStore();

// 编辑状态
const editingId = ref<string | null>(null);
const editForm = ref({
  startTime: '',
  endTime: '',
  text: ''
});

// 文本框引用
const textareaRef = ref();

// 添加字幕对话框
const showAddDialog = ref(false);
const addForm = ref({
  startTime: '',
  endTime: '',
  text: ''
});

// 计算属性：排序后的字幕列表
const sortedSubtitles = computed(() => {
  return [...videoStore.subtitles].sort((a, b) => a.startTime - b.startTime);
});

// 计算属性：当前字幕
const currentSubtitle = computed(() => videoStore.currentSubtitle);

/**
 * 格式化时间显示
 * @param seconds 秒数
 * @returns 格式化的时间字符串 (HH:MM:SS)
 */
function formatTime(seconds: number | undefined | null): string {
  // 处理无效输入
  if (seconds == null || isNaN(seconds) || seconds < 0) {
    return '00:00:00';
  }
  
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);
  
  return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
}

/**
 * 解析时间字符串为秒数
 * @param timeStr 时间字符串 (HH:MM:SS)
 * @returns 秒数
 */
function parseTime(timeStr: string): number {
  const parts = timeStr.split(':').map(Number);
  if (parts.length !== 3) return 0;
  
  const [hours, minutes, seconds] = parts;
  return hours * 3600 + minutes * 60 + seconds;
}

/**
 * 开始编辑字幕
 * @param subtitle 要编辑的字幕
 */
function startEdit(subtitle: Subtitle) {
  editingId.value = subtitle.id;
  editForm.value = {
    startTime: formatTime(subtitle.startTime),
    endTime: formatTime(subtitle.endTime),
    text: subtitle.text
  };

  // 下一帧聚焦到文本框
  nextTick(() => {
    if (textareaRef.value) {
      textareaRef.value.focus();
      // 选中所有文本
      const textarea = textareaRef.value.textarea || textareaRef.value.$el?.querySelector('textarea');
      if (textarea) {
        textarea.select();
      }
    }
  });
}

/**
 * 保存编辑
 */
function saveEdit() {
  if (!editingId.value) return;
  
  const startTime = parseTime(editForm.value.startTime);
  const endTime = parseTime(editForm.value.endTime);
  
  if (startTime >= endTime) {
    ElMessage.error('开始时间必须小于结束时间');
    return;
  }
  
  if (!editForm.value.text.trim()) {
    ElMessage.error('字幕内容不能为空');
    return;
  }
  
  const updatedSubtitle: Subtitle = {
    id: editingId.value,
    startTime,
    endTime,
    text: editForm.value.text.trim()
  };
  
  videoStore.updateSubtitle(updatedSubtitle);
  cancelEdit();
  ElMessage.success('字幕更新成功');
}

/**
 * 取消编辑
 */
function cancelEdit() {
  editingId.value = null;
  editForm.value = {
    startTime: '',
    endTime: '',
    text: ''
  };
}

/**
 * 处理文本框键盘事件
 */
function handleTextareaKeydown(event: KeyboardEvent) {
  // Ctrl+Enter 保存
  if (event.ctrlKey && event.key === 'Enter') {
    event.preventDefault();
    saveEdit();
  }
  // Esc 取消编辑
  else if (event.key === 'Escape') {
    event.preventDefault();
    cancelEdit();
  }
}

/**
 * 跳转到指定时间
 * @param time 时间（秒）
 */
function jumpToTime(time: number) {
  console.log('=== 字幕跳转功能 ===');
  console.log('跳转时间:', time);
  console.log('格式化时间:', formatTime(time));

  // 验证时间参数
  if (typeof time !== 'number' || isNaN(time) || time < 0) {
    console.error('无效的时间参数:', time);
    ElMessage.error('无效的时间参数');
    return;
  }

  // 更新视频播放器的时间（通过store，会触发VideoPlayer组件的监听器）
  console.log('更新store中的currentTime');
  videoStore.updateCurrentTime(time);

  console.log(`已跳转到时间: ${formatTime(time)}`);
  ElMessage.success(`已跳转到 ${formatTime(time)}`);
}

/**
 * 删除字幕
 * @param id 字幕ID
 */
function deleteSubtitle(id: string) {
  ElMessageBox.confirm('确定要删除这条字幕吗？', '确认删除', {
    type: 'warning'
  }).then(() => {
    videoStore.removeSubtitle(id);
    ElMessage.success('字幕删除成功');
  }).catch(() => {
    // 用户取消删除
  });
}

/**
 * 添加新字幕
 */
function addNewSubtitle() {
  // 设置默认时间为当前播放时间
  const currentTime = videoStore.currentTime;
  addForm.value = {
    startTime: formatTime(currentTime),
    endTime: formatTime(currentTime + 3), // 默认3秒时长
    text: ''
  };
  showAddDialog.value = true;
}

/**
 * 确认添加字幕
 */
function confirmAddSubtitle() {
  const startTime = parseTime(addForm.value.startTime);
  const endTime = parseTime(addForm.value.endTime);
  
  if (startTime >= endTime) {
    ElMessage.error('开始时间必须小于结束时间');
    return;
  }
  
  if (!addForm.value.text.trim()) {
    ElMessage.error('字幕内容不能为空');
    return;
  }
  
  const newSubtitle: Subtitle = {
    id: Date.now().toString(),
    startTime,
    endTime,
    text: addForm.value.text.trim()
  };
  
  videoStore.addSubtitle(newSubtitle);
  showAddDialog.value = false;
  ElMessage.success('字幕添加成功');
}

// 导出对话框状态
const showExportDialog = ref(false);
const exportForm = ref({
  format: 'srt' as SubtitleFormat,
  fileName: ''
});

/**
 * 导出字幕
 */
function exportSubtitles() {
  if (videoStore.subtitles.length === 0) {
    ElMessage.warning('暂无字幕数据可导出');
    return;
  }
  
  // 设置默认文件名
  const defaultName = videoStore.currentVideo?.fileName?.replace(/\.[^/.]+$/, '') || 'subtitles';
  exportForm.value.fileName = defaultName;
  showExportDialog.value = true;
}

/**
 * 确认导出字幕
 */
async function confirmExportSubtitles() {
  if (!exportForm.value.fileName.trim()) {
    ElMessage.error('请输入文件名');
    return;
  }
  
  try {
    // 创建进度任务
    const progressTaskId = ProgressMonitor.createTask(
      `导出字幕 - ${exportForm.value.format.toUpperCase()}`,
      5000, // 预估5秒
      {
        fileName: exportForm.value.fileName,
        format: exportForm.value.format,
        subtitleCount: videoStore.subtitles.length
      }
    );
    
    try {
      ProgressMonitor.startTask(progressTaskId, '正在准备导出数据...');
      ProgressMonitor.updateProgress(progressTaskId, 20, '正在准备导出数据...');
      
      const { exportSubtitles: exportSubtitlesUtil, exportSubtitlesToPath } = await import('../utils/videoUtils');

      ProgressMonitor.updateProgress(progressTaskId, 60, '正在生成字幕文件...');

      // 检查是否设置了自定义导出路径
      const customExportPath = settingsStore.settings.exportPath;
      let filePath: string;

      if (customExportPath && customExportPath.trim()) {
        // 使用自定义路径导出
        filePath = await exportSubtitlesToPath(
          videoStore.subtitles,
          exportForm.value.format,
          exportForm.value.fileName.trim(),
          customExportPath
        );
      } else {
        // 使用默认路径导出
        filePath = await exportSubtitlesUtil(
          videoStore.subtitles,
          exportForm.value.format,
          exportForm.value.fileName.trim()
        );
      }
      
      ProgressMonitor.completeTask(progressTaskId, `字幕导出成功: ${filePath}`);

      showExportDialog.value = false;

      // 显示成功消息，并提供打开文件夹选项
      ElMessageBox.confirm(
        `字幕导出成功：${filePath}`,
        '导出完成',
        {
          confirmButtonText: '打开文件夹',
          cancelButtonText: '关闭',
          type: 'success',
          showClose: false
        }
      ).then(async () => {
        // 用户选择打开文件夹
        try {
          const { invoke } = await import('@tauri-apps/api/core');
          const folderPath = customExportPath && customExportPath.trim()
            ? customExportPath
            : process.cwd(); // 如果没有自定义路径，使用当前工作目录
          await invoke('open_folder', { path: folderPath });
        } catch (error) {
          console.error('打开文件夹失败:', error);
          ElMessage.error('打开文件夹失败');
        }
      }).catch(() => {
        // 用户选择关闭，不做任何操作
      });
    } catch (error) {
      ProgressMonitor.failTask(progressTaskId, `导出失败: ${error}`);
      throw error;
    }
  } catch (error) {
    ElMessage.error(`导出字幕失败：${error instanceof Error ? error.message : String(error)}`);
  }
}


</script>

<style scoped>
.subtitle-editor {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 20px;
  background: #ffffff;
  overflow: hidden;
}

.editor-header {
  display: flex;
  justify-content: flex-start;
  align-items: center;
  margin-bottom: 20px;
  padding: 12px 0;
}

.header-actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.editor-content {
  flex: 1;
  overflow-y: auto;
  background: white;
  border-radius: 8px;
  border: 1px solid #e9ecef;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}

/* 字幕列表样式 */
.subtitle-list {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 16px;
}

.list-header {
  display: grid;
  grid-template-columns: 200px 1fr 180px;
  gap: 16px;
  padding: 16px 20px;
  background: #f8f9fa;
  color: #495057;
  border-radius: 8px;
  font-weight: 600;
  margin-bottom: 16px;
  position: sticky;
  top: 0;
  z-index: 10;
  border: 1px solid #e9ecef;
}

.list-header .col-time,
.list-header .col-text,
.list-header .col-actions {
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
}

.subtitle-item {
  display: grid;
  grid-template-columns: 200px 1fr 180px;
  gap: 16px;
  padding: 16px 20px;
  border: 1px solid #e2e8f0;
  margin-bottom: 12px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  background: white;
  position: relative;
  overflow: hidden;
  border-radius: 8px;
  min-height: 80px;
  align-items: start;
}

.subtitle-item::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 3px;
  background: #0fdc78;
  transform: scaleX(0);
  transition: transform 0.3s ease;
}

.subtitle-item:hover {
  border-color: #0fdc78;
  background: #f8fff8;
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(15, 220, 120, 0.1);
}

.subtitle-item:hover::before {
  transform: scaleX(1);
}

.subtitle-item.active {
  border-color: #0fdc78;
  background: #f0fff4;
  box-shadow: 0 2px 8px rgba(15, 220, 120, 0.15);
}

.subtitle-item.active::before {
  transform: scaleX(1);
}

.col-time {
  display: flex;
  flex-direction: column;
  gap: 4px;
  align-items: center;
  justify-content: center;
}

.time-display {
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 12px;
  color: #475569;
  background: #f1f5f9;
  padding: 6px 10px;
  border-radius: 4px;
  text-align: center;
  border: 1px solid #e2e8f0;
  font-weight: 500;
  letter-spacing: 0.5px;
  white-space: nowrap;
  min-width: 80px;
}

.time-edit {
  display: flex;
  align-items: center;
  gap: 8px;
}

.time-input {
  flex: 1;
}

.time-separator {
  color: #909399;
  font-weight: bold;
}

.col-text {
  display: flex;
  align-items: flex-start;
  padding: 0 8px;
  min-height: 40px;
}

.text-display {
  line-height: 1.6;
  color: #303133;
  word-break: break-word;
  overflow-wrap: break-word;
  max-width: 100%;
  font-size: 14px;
  padding: 8px 12px;
  background: #f8f9fa;
  border-radius: 6px;
  border: 1px solid #e9ecef;
  min-height: 60px;
  display: flex;
  align-items: center;
  transition: all 0.2s ease;
}

.text-display:hover {
  background: #f1f3f4;
  border-color: #d0d7de;
}

.text-edit-container {
  width: 100%;
  position: relative;
}

.text-input {
  width: 100%;
}

.edit-tips {
  margin-top: 8px;
  padding: 0 4px;
}

.tip-text {
  font-size: 12px;
  color: #6c757d;
  font-style: italic;
}

.col-actions {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 8px;
  min-width: 80px;
  position: relative;
}

.action-buttons,
.edit-buttons {
  display: flex;
  gap: 6px;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 60px;
  margin: 0 auto;
}

/* 强制按钮对齐样式 */
:deep(.action-buttons .el-button),
:deep(.edit-buttons .el-button) {
  width: 60px !important;
  min-width: 60px !important;
  max-width: 60px !important;
  justify-content: center !important;
  text-align: center !important;
  font-size: 12px !important;
  padding: 4px 8px !important;
  border-radius: 4px !important;
  box-sizing: border-box !important;
  margin: 0 !important;
  display: flex !important;
  align-items: center !important;
}

/* 确保按钮内的文字居中 */
:deep(.action-buttons .el-button span),
:deep(.edit-buttons .el-button span) {
  width: 100% !important;
  text-align: center !important;
  display: block !important;
}

/* 空状态样式 */
.empty-state {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  color: #6c757d;
  background: white;
  margin: 16px;
  padding: 40px;
  border-radius: 8px;
}

.empty-state p {
  margin: 12px 0;
  font-size: 16px;
  line-height: 1.6;
}

.empty-state p:first-child {
  font-weight: 600;
  font-size: 18px;
  color: #495057;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .list-header,
  .subtitle-item {
    grid-template-columns: 1fr;
    gap: 12px;
    padding: 12px 16px;
  }
  
  .col-time {
    order: 1;
  }
  
  .col-text {
    order: 2;
    padding: 0;
  }
  
  .col-actions {
    order: 3;
    justify-content: flex-start;
    padding-top: 0;
  }
  
  .action-buttons,
  .edit-buttons {
    justify-content: flex-start;
    flex-direction: row;
    gap: 8px;
  }
  
  .subtitle-item {
    min-height: auto;
  }
}

/* 扁平化滚动条样式 */
.subtitle-list {
  overflow-y: auto;
  max-height: calc(100vh - 200px);
  padding-right: 8px;
}

.subtitle-list::-webkit-scrollbar {
  width: 8px;
}

.subtitle-list::-webkit-scrollbar-track {
  background: #f1f1f1;
  border-radius: 4px;
}

.subtitle-list::-webkit-scrollbar-thumb {
  background: #c1c1c1;
  border-radius: 4px;
}

.subtitle-list::-webkit-scrollbar-thumb:hover {
  background: #a8a8a8;
}

/* 扁平化Element Plus组件样式 */
:deep(.el-button) {
  border-radius: 0;
  font-weight: 500;
  border: 2px solid #0fdc78;
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

:deep(.el-button--danger) {
  background: #dc3545;
  border-color: #dc3545;
  color: white;
}

:deep(.el-button--danger:hover) {
  background: #c82333;
  border-color: #c82333;
  color: white;
}

:deep(.el-input__wrapper) {
  transition: all 0.3s ease;
  border-radius: 0;
}

:deep(.el-input__wrapper:hover) {
  border-radius: 0;
}

:deep(.el-textarea__inner) {
  transition: all 0.3s ease;
  border-radius: 8px !important;
  border: 2px solid #e9ecef !important;
  padding: 12px 16px !important;
  font-size: 14px !important;
  line-height: 1.6 !important;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif !important;
  resize: vertical !important;
  min-height: 80px !important;
  background: #ffffff !important;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05) !important;
}

:deep(.el-textarea__inner:focus) {
  border-color: #0fdc78 !important;
  box-shadow: 0 0 0 3px rgba(15, 220, 120, 0.1), 0 2px 8px rgba(0, 0, 0, 0.1) !important;
  outline: none !important;
}

:deep(.el-textarea__inner:hover) {
  border-color: #0fdc78 !important;
}

:deep(.el-textarea .el-input__count) {
  background: rgba(255, 255, 255, 0.9) !important;
  border-radius: 4px !important;
  padding: 2px 6px !important;
  font-size: 11px !important;
  color: #6c757d !important;
  border: 1px solid #e9ecef !important;
}

:deep(.el-dialog) {
  border-radius: 0;
  overflow: hidden;
}

:deep(.el-dialog__header) {
  background: #000000;
  color: white;
  padding: 20px;
}

:deep(.el-dialog__title) {
  color: white;
  font-weight: 600;
}

:deep(.el-dialog__headerbtn .el-dialog__close) {
  color: white;
}

:deep(.el-dialog__body) {
  padding: 24px;
}
</style>