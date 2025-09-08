<template>
  <div class="task-status-bar" v-if="showStatusBar">
    <!-- 运行中的任务 -->
    <div v-if="runningTasks.length > 0" class="running-tasks">
      <div v-for="task in runningTasks" :key="task.id" class="task-item">
        <div class="task-info">
          <span class="task-name">{{ task.name }}</span>
          <span class="task-progress">{{ Math.round(task.progress) }}%</span>
          <span v-if="task.message" class="task-message">{{ task.message }}</span>
        </div>
        <el-progress
          :percentage="task.progress"
          :show-text="false"
          :stroke-width="4"
          class="task-progress-bar"
        />
        <div v-if="getTimeRemaining(task.id) !== '未知'" class="time-remaining">
          剩余: {{ getTimeRemaining(task.id) }}
        </div>
      </div>
    </div>

    <!-- 错误信息 -->
    <div v-if="recentErrors.length > 0" class="error-section">
      <el-button
        type="danger"
        size="small"
        @click="showErrorDialog = true"
        class="error-button"
      >
        <el-icon><Warning /></el-icon>
        {{ recentErrors.length }} 个错误
      </el-button>
    </div>

    <!-- 控制按钮 -->
    <div class="controls">
      <el-button
        v-if="runningTasks.length > 0"
        type="info"
        size="small"
        @click="showTaskDialog = true"
      >
        <el-icon><List /></el-icon>
        任务 ({{ runningTasks.length }})
      </el-button>
      
      <el-button
        type="text"
        size="small"
        @click="clearCompleted"
        v-if="allTasks.some(t => t.status === 'completed' || t.status === 'failed')"
      >
        清理
      </el-button>
    </div>

    <!-- 任务详情对话框 -->
    <el-dialog
      v-model="showTaskDialog"
      title="任务状态"
      width="600px"
      :close-on-click-modal="false"
    >
      <div class="task-dialog-content">
        <div v-for="task in allTasks" :key="task.id" class="task-detail-item">
          <div class="task-header">
            <span class="task-name">{{ task.name }}</span>
            <el-tag
              :type="getTaskTagType(task.status)"
              size="small"
            >
              {{ getTaskStatusText(task.status) }}
            </el-tag>
          </div>
          
          <div v-if="task.status === 'running'" class="task-progress-section">
            <el-progress
              :percentage="task.progress"
              :status="task.status === 'failed' ? 'exception' : undefined"
            />
            <div class="progress-info">
              <span>{{ Math.round(task.progress) }}%</span>
              <span v-if="getTimeRemaining(task.id) !== '未知'">
                剩余: {{ getTimeRemaining(task.id) }}
              </span>
            </div>
          </div>
          
          <div v-if="task.message" class="task-message">
            {{ task.message }}
          </div>
          
          <div v-if="task.error" class="task-error">
            <el-alert
              :title="task.error"
              type="error"
              :closable="false"
              show-icon
            />
          </div>
          
          <div class="task-times">
            <span v-if="task.startTime">
              开始: {{ formatTime(task.startTime) }}
            </span>
            <span v-if="task.endTime">
              结束: {{ formatTime(task.endTime) }}
            </span>
          </div>
        </div>
      </div>
      
      <template #footer>
        <el-button @click="showTaskDialog = false">关闭</el-button>
        <el-button type="primary" @click="clearCompleted">清理已完成</el-button>
      </template>
    </el-dialog>

    <!-- 错误详情对话框 -->
    <el-dialog
      v-model="showErrorDialog"
      title="错误信息"
      width="700px"
      :close-on-click-modal="false"
    >
      <div class="error-dialog-content">
        <div v-for="error in recentErrors" :key="error.id" class="error-item">
          <div class="error-header">
            <el-tag
              :type="getErrorTagType(error.severity)"
              size="small"
            >
              {{ error.type }}
            </el-tag>
            <span class="error-time">{{ formatTime(error.timestamp) }}</span>
          </div>
          
          <div class="error-message">
            {{ error.message }}
          </div>
          
          <div v-if="error.details" class="error-details">
            <el-collapse>
              <el-collapse-item title="详细信息">
                <pre>{{ error.details }}</pre>
              </el-collapse-item>
            </el-collapse>
          </div>
          
          <div v-if="error.context" class="error-context">
            <strong>上下文:</strong>
            <pre>{{ JSON.stringify(error.context, null, 2) }}</pre>
          </div>
        </div>
      </div>
      
      <template #footer>
        <el-button @click="showErrorDialog = false">关闭</el-button>
        <el-button type="danger" @click="clearErrors">清除错误</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { Warning, List } from '@element-plus/icons-vue';
import { ProgressMonitor } from '../utils/progressMonitor';
import { ErrorHandler, type AppError } from '../utils/errorHandler';
import type { ProgressTask } from '../utils/progressMonitor';

// 对话框状态
const showTaskDialog = ref(false);
const showErrorDialog = ref(false);

// 获取任务数据
const allTasks = ProgressMonitor.getAllTasks();
const runningTasks = ProgressMonitor.getRunningTasks();

// 错误数据
const errors = ref<AppError[]>([]);

// 计算属性
const showStatusBar = computed(() => {
  return runningTasks.value.length > 0 || recentErrors.value.length > 0;
});

const recentErrors = computed(() => {
  const now = new Date();
  const oneHourAgo = new Date(now.getTime() - 60 * 60 * 1000);
  return errors.value.filter(error => error.timestamp > oneHourAgo);
});

// 定时更新错误列表
let errorUpdateInterval: number;

onMounted(() => {
  updateErrors();
  errorUpdateInterval = window.setInterval(updateErrors, 5000);
});

onUnmounted(() => {
  if (errorUpdateInterval) {
    clearInterval(errorUpdateInterval);
  }
});

/**
 * 更新错误列表
 */
function updateErrors() {
  errors.value = ErrorHandler.getErrors();
}

/**
 * 获取任务状态标签类型
 * @param status 任务状态
 * @returns 标签类型
 */
function getTaskTagType(status: string): string {
  switch (status) {
    case 'completed':
      return 'success';
    case 'failed':
      return 'danger';
    case 'running':
      return 'primary';
    case 'cancelled':
      return 'warning';
    default:
      return 'info';
  }
}

/**
 * 获取任务状态文本
 * @param status 任务状态
 * @returns 状态文本
 */
function getTaskStatusText(status: string): string {
  switch (status) {
    case 'idle':
      return '待机';
    case 'pending':
      return '等待中';
    case 'running':
      return '运行中';
    case 'completed':
      return '已完成';
    case 'failed':
      return '失败';
    case 'cancelled':
      return '已取消';
    default:
      return '未知';
  }
}

/**
 * 获取错误标签类型
 * @param severity 错误严重程度
 * @returns 标签类型
 */
function getErrorTagType(severity: string): string {
  switch (severity) {
    case 'low':
      return 'info';
    case 'medium':
      return 'warning';
    case 'high':
      return 'danger';
    case 'critical':
      return 'danger';
    default:
      return 'info';
  }
}

/**
 * 获取剩余时间
 * @param taskId 任务ID
 * @returns 格式化的剩余时间
 */
function getTimeRemaining(taskId: string): string {
  return ProgressMonitor.getFormattedTimeRemaining(taskId);
}

/**
 * 格式化时间
 * @param date 日期对象
 * @returns 格式化的时间字符串
 */
function formatTime(date: Date): string {
  return date.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  });
}

/**
 * 清理已完成的任务
 */
function clearCompleted() {
  ProgressMonitor.clearCompletedTasks();
  showTaskDialog.value = false;
}

/**
 * 清除错误
 */
function clearErrors() {
  ErrorHandler.clearErrors();
  updateErrors();
  showErrorDialog.value = false;
}
</script>

<style scoped>
.task-status-bar {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  background: #f5f7fa;
  border-top: 1px solid #e4e7ed;
  padding: 8px 16px;
  display: flex;
  align-items: center;
  gap: 16px;
  z-index: 1000;
  max-height: 120px;
  overflow-y: auto;
}

.running-tasks {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.task-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 4px 8px;
  background: white;
  border-radius: 4px;
  border: 1px solid #e4e7ed;
}

.task-info {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 200px;
}

.task-name {
  font-weight: 500;
  color: #303133;
}

.task-progress {
  font-size: 12px;
  color: #409eff;
  font-weight: 500;
}

.task-message {
  font-size: 12px;
  color: #909399;
}

.task-progress-bar {
  flex: 1;
  min-width: 100px;
}

.time-remaining {
  font-size: 12px;
  color: #909399;
  white-space: nowrap;
}

.error-section {
  display: flex;
  align-items: center;
}

.error-button {
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0% {
    box-shadow: 0 0 0 0 rgba(245, 108, 108, 0.7);
  }
  70% {
    box-shadow: 0 0 0 10px rgba(245, 108, 108, 0);
  }
  100% {
    box-shadow: 0 0 0 0 rgba(245, 108, 108, 0);
  }
}

.controls {
  display: flex;
  align-items: center;
  gap: 8px;
}

.task-dialog-content {
  max-height: 400px;
  overflow-y: auto;
}

.task-detail-item {
  padding: 16px;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  margin-bottom: 12px;
}

.task-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.task-progress-section {
  margin: 12px 0;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  margin-top: 4px;
  font-size: 12px;
  color: #909399;
}

.task-message {
  margin: 8px 0;
  padding: 8px;
  background: #f5f7fa;
  border-radius: 4px;
  font-size: 14px;
}

.task-error {
  margin: 8px 0;
}

.task-times {
  display: flex;
  gap: 16px;
  margin-top: 8px;
  font-size: 12px;
  color: #909399;
}

.error-dialog-content {
  max-height: 500px;
  overflow-y: auto;
}

.error-item {
  padding: 16px;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  margin-bottom: 12px;
}

.error-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.error-time {
  font-size: 12px;
  color: #909399;
}

.error-message {
  margin: 8px 0;
  font-size: 14px;
  color: #303133;
}

.error-details,
.error-context {
  margin: 8px 0;
}

.error-details pre,
.error-context pre {
  background: #f5f7fa;
  padding: 8px;
  border-radius: 4px;
  font-size: 12px;
  overflow-x: auto;
}
</style>