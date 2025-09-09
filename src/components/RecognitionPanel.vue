<script setup lang="ts">
import { ref, computed } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Microphone, Close } from '@element-plus/icons-vue';
import { useVideoStore, useSettingsStore } from '../stores';
import { extractAudio } from '../utils/videoUtils';
import { startRecognition, getRecognitionStatus, cancelRecognition } from '../utils/recognitionUtils';
import { ProgressMonitor } from '../utils/progressMonitor';
import { ErrorHandler, ErrorType, ErrorSeverity, withErrorHandling } from '../utils/errorHandler';
import type { RecognitionEngine } from '../types';

// 引入存储
const videoStore = useVideoStore();
const settingsStore = useSettingsStore();

// 识别设置
const recognitionSettings = ref({
  engine: 'whisper' as RecognitionEngine, // 默认使用whisper，不需要API密钥
  language: settingsStore.settings.defaultLanguage
});

// 加载状态
const loading = ref({
  extract: false,
  recognize: false
});

// 当前任务ID
const currentTaskId = ref<string | null>(null);
const currentProgressTaskId = ref<string | null>(null);

// 识别进度
const recognitionProgress = ref(0);

// 识别状态
const recognitionStatus = ref<'idle' | 'extracting' | 'recognizing' | 'completed' | 'failed'>('idle');

// 错误信息
const errorMessage = ref('');

// 计算属性：是否可以开始识别
const canStartRecognition = computed(() => {
  const result = (
    videoStore.isVideoLoaded &&
    videoStore.selectedAudioTrackId !== null &&
    recognitionStatus.value !== 'extracting' &&
    recognitionStatus.value !== 'recognizing'
  );
  
  console.log('canStartRecognition 计算:', {
    isVideoLoaded: videoStore.isVideoLoaded,
    selectedAudioTrackId: videoStore.selectedAudioTrackId,
    recognitionStatus: recognitionStatus.value,
    result: result
  });
  
  return result;
});

// 计算属性：是否可以取消识别
const canCancelRecognition = computed(() => {
  return (
    recognitionStatus.value === 'extracting' ||
    recognitionStatus.value === 'recognizing'
  );
});

// 支持的语言列表
const supportedLanguages = ref([
  { code: 'zh', name: '中文' },
  { code: 'en', name: '英语' },
  { code: 'ja', name: '日语' },
  { code: 'ko', name: '韩语' },
  { code: 'fr', name: '法语' },
  { code: 'de', name: '德语' },
  { code: 'es', name: '西班牙语' },
  { code: 'ru', name: '俄语' }
]);

/**
 * 开始识别流程
 */
async function startRecognitionProcess() {
  console.log('=== 开始识别按钮点击 ===');
  console.log('当前视频:', videoStore.currentVideo);
  console.log('选择的音频轨道ID:', videoStore.selectedAudioTrackId);
  console.log('识别设置:', recognitionSettings.value);
  
  // 先显示一个简单的提示，确认按钮点击有效
  ElMessage.info('按钮点击成功！开始处理识别请求...');
  
  if (!videoStore.currentVideo || videoStore.selectedAudioTrackId === null) {
    ElMessage.warning('请先导入视频并选择音频轨道');
    return;
  }
  
  try {
    // 使用Whisper本地识别，无需API密钥检查
    const engine = recognitionSettings.value.engine;
    console.log('使用引擎:', engine);
    
    // 确认开始识别
    await ElMessageBox.confirm(
      `将使用Whisper本地模型进行语音识别，是否继续？`,
      '开始识别',
      {
        confirmButtonText: '开始',
        cancelButtonText: '取消',
        type: 'info'
      }
    );
    
    // 创建进度任务
    const engineName = 'Whisper本地模型';
    const progressTaskId = ProgressMonitor.createTask(
      `语音识别 - ${engineName}`,
      120000, // 预估2分钟
      {
        video: videoStore.currentVideo!.fileName,
        engine,
        language: recognitionSettings.value.language
      }
    );
    
    currentProgressTaskId.value = progressTaskId;
    
    // 重置状态
    recognitionStatus.value = 'extracting';
    recognitionProgress.value = 0;
    errorMessage.value = '';
    
    // 提取音频阶段
    ProgressMonitor.startTask(progressTaskId, '正在提取音频...');
    ProgressMonitor.updateProgress(progressTaskId, 10, '正在提取音频...');
    
    loading.value.extract = true;
    const audioPath = await extractAudio(
      videoStore.currentVideo!.filePath,
      videoStore.selectedAudioTrackId!
    );
    loading.value.extract = false;
    
    // 开始识别阶段
    ProgressMonitor.updateProgress(progressTaskId, 30, '音频提取完成，开始语音识别...');
    
    recognitionStatus.value = 'recognizing';
    loading.value.recognize = true;
    
    // Whisper本地识别，无需API密钥
    const apiKeys = undefined;
    
    const task = await startRecognition(
      audioPath,
      recognitionSettings.value.engine,
      recognitionSettings.value.language,
      videoStore.currentVideo!,
      apiKeys
    );
    
    currentTaskId.value = task.id;
    videoStore.addRecognitionTask(task);
    
    // 定时检查识别状态
    await monitorRecognitionProgress(task.id, progressTaskId);
    
  } catch (error) {
    // 处理错误时的清理工作
    recognitionStatus.value = 'failed';
    errorMessage.value = error instanceof Error ? error.message : String(error);
    loading.value.extract = false;
    loading.value.recognize = false;
    
    if (currentProgressTaskId.value) {
      ProgressMonitor.failTask(currentProgressTaskId.value, error instanceof Error ? error.message : String(error));
      currentProgressTaskId.value = null;
    }
    
    ElMessage.error(error instanceof Error ? error.message : String(error));
  }
}

/**
 * 监控识别进度
 * @param taskId 任务ID
 * @param progressTaskId 进度任务ID
 */
async function monitorRecognitionProgress(taskId: string, progressTaskId: string) {
  const checkInterval = setInterval(async () => {
    try {
      if (recognitionStatus.value !== 'recognizing') {
        clearInterval(checkInterval);
        return;
      }
      
      const status = await getRecognitionStatus(taskId);
      console.log('获取到识别状态:', status);
      
      // 将 0.0-1.0 转换为 0-100 的百分比
      const progressPercent = status.progress * 100;
      recognitionProgress.value = progressPercent;
      
      console.log('更新前端进度:', {
        originalProgress: status.progress,
        progressPercent: progressPercent,
        status: status.status
      });

      // 更新进度监控任务
      const adjustedProgress = 30 + (status.progress * 70); // 30% 基础进度 + 70% 识别进度
      ProgressMonitor.updateProgress(progressTaskId, adjustedProgress, `识别进度: ${Math.round(progressPercent)}%`);

      // 更新任务状态
      videoStore.updateRecognitionTask(taskId, {
        status: status.status as any,
        progress: progressPercent,
        subtitles: status.subtitles,
        error: status.error,
        updatedAt: new Date()
      });

      if (status.status === 'completed') {
        // 识别完成
        clearInterval(checkInterval);

        // 重置所有状态，允许开始新的识别任务
        recognitionStatus.value = 'idle';
        loading.value.recognize = false;
        currentTaskId.value = null;

        // 设置字幕
        if (status.subtitles && status.subtitles.length > 0) {
          // 转换后端数据格式（start_time -> startTime, end_time -> endTime）
          const convertedSubtitles = status.subtitles.map((subtitle: any) => ({
            id: subtitle.id,
            startTime: subtitle.start_time,
            endTime: subtitle.end_time,
            text: subtitle.text
          }));

          videoStore.setSubtitles(convertedSubtitles);

          // 完成进度任务
          ProgressMonitor.completeTask(progressTaskId, `识别完成，共生成${status.subtitles.length}条字幕`);
          currentProgressTaskId.value = null;

          ElMessage.success(`识别完成，共生成${status.subtitles.length}条字幕`);
        } else {
          ProgressMonitor.completeTask(progressTaskId, '识别完成，但未生成字幕');
          currentProgressTaskId.value = null;
          ElMessage.warning('识别完成，但未生成字幕');
        }
      } else if (status.status === 'failed') {
        // 识别失败
        recognitionStatus.value = 'failed';
        errorMessage.value = status.error || '未知错误';
        clearInterval(checkInterval);

        // 重置加载状态和任务ID
        loading.value.recognize = false;
        currentTaskId.value = null;

        // 记录错误并失败进度任务
        const errorMsg = status.error || '未知错误';
        ErrorHandler.handle(
          new Error(errorMsg),
          'RECOGNITION_ERROR' as any,
          'HIGH' as any,
          {
            component: 'RecognitionPanel',
            action: 'monitorProgress',
            taskId,
            progressTaskId
          }
        );

        ProgressMonitor.failTask(progressTaskId, errorMsg);
        currentProgressTaskId.value = null;

        ElMessage.error(`识别失败: ${errorMsg}`);
      }
    } catch (error) {
      console.error('获取识别状态失败:', error);
      clearInterval(checkInterval);
      recognitionStatus.value = 'failed';

      // 重置加载状态和任务ID
      loading.value.recognize = false;
      currentTaskId.value = null;

      const errorMsg = `获取识别状态失败: ${error}`;
      errorMessage.value = errorMsg;

      // 记录错误并失败进度任务
      ErrorHandler.handle(
        error instanceof Error ? error : new Error(String(error)),
        'NETWORK_ERROR' as any,
        'HIGH' as any,
        {
          component: 'RecognitionPanel',
          action: 'monitorProgress',
          taskId,
          progressTaskId
        }
      );

      ProgressMonitor.failTask(progressTaskId, errorMsg);
      currentProgressTaskId.value = null;
      
      ElMessage.error(errorMsg);
    }
  }, 2000); // 每2秒检查一次
}

/**
 * 取消识别
 */
async function cancelRecognitionProcess() {
  console.log('=== 取消识别按钮点击 ===');
  console.log('当前任务ID:', currentTaskId.value);
  
  // 先显示一个简单的提示，确认按钮点击有效
  ElMessage.info('取消按钮点击成功！');
  
  if (!currentTaskId.value) {
    ElMessage.warning('没有正在运行的识别任务');
    return;
  }
  
  try {
    await ElMessageBox.confirm('确定要取消当前识别任务吗？', '取消识别', {
      confirmButtonText: '确定',
      cancelButtonText: '继续识别',
      type: 'warning'
    });
    
    await cancelRecognition(currentTaskId.value);
    
    // 取消进度任务
    if (currentProgressTaskId.value) {
      ProgressMonitor.cancelTask(currentProgressTaskId.value, '用户取消识别任务');
      currentProgressTaskId.value = null;
    }
    
    recognitionStatus.value = 'idle';
    currentTaskId.value = null;
    loading.value.extract = false;
    loading.value.recognize = false;
    
    ElMessage.info('已取消识别任务');
  } catch {
    // 用户取消操作
  }
}
</script>

<template>
  <div class="recognition-panel">
    <div v-if="!videoStore.isVideoLoaded" class="no-video">
      <el-empty description="请先导入视频" />
    </div>
    
    <div v-else class="recognition-content">
      <el-form label-width="80px" class="recognition-form">
        <el-form-item label="识别引擎">
          <el-select v-model="recognitionSettings.engine" class="form-select">
            <el-option label="Whisper (本地)" value="whisper" />
          </el-select>
        </el-form-item>

        <el-form-item label="识别语言">
          <el-select v-model="recognitionSettings.language" class="form-select">
            <el-option
              v-for="lang in supportedLanguages"
              :key="lang.code"
              :label="lang.name"
              :value="lang.code"
            />
          </el-select>
        </el-form-item>
        
        <el-form-item>
          <el-button
            type="primary"
            :disabled="!canStartRecognition"
            :loading="loading.extract || loading.recognize"
            @click="startRecognitionProcess"
          >
            <el-icon><Microphone /></el-icon> 开始识别
          </el-button>
          
          <el-button
            type="danger"
            :disabled="!canCancelRecognition"
            @click="cancelRecognitionProcess"
          >
            <el-icon><Close /></el-icon> 取消识别
          </el-button>
        </el-form-item>
      </el-form>
      
      <!-- 识别状态和进度 -->
      <div v-if="recognitionStatus !== 'idle'" class="recognition-status">
        <div class="status-header">
          <span class="status-label">
            {{
              recognitionStatus === 'extracting' ? '正在提取音频...' :
              recognitionStatus === 'recognizing' ? '正在识别...' :
              recognitionStatus === 'completed' ? '识别完成' :
              '识别失败'
            }}
          </span>
        </div>
        
        <el-progress
          :percentage="recognitionProgress"
          :status="
            recognitionStatus === 'completed' ? 'success' :
            recognitionStatus === 'failed' ? 'exception' :
            ''
          "
        />
        
        <div v-if="recognitionStatus === 'failed'" class="error-message">
          {{ errorMessage }}
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.recognition-panel {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 20px;
  overflow-y: auto;
  box-sizing: border-box;
  max-width: 100%;
}

.no-video {
  display: flex;
  justify-content: center;
  align-items: center;
  flex: 1;
  min-height: 200px;
}

.recognition-content {
  flex: 1;
  width: 100%;
  overflow: hidden;
}

.recognition-form {
  width: 100%;
  max-width: 100%;
  box-sizing: border-box;
}

/* 确保表单项正确布局 */
:deep(.el-form-item) {
  margin-bottom: 18px;
  width: 100%;
  box-sizing: border-box;
}

:deep(.el-form-item__label) {
  width: 80px !important;
  flex-shrink: 0;
}

:deep(.el-form-item__content) {
  flex: 1;
  width: calc(100% - 80px);
  max-width: calc(100% - 80px);
  margin-left: 0 !important;
}

:deep(.el-select) {
  width: 100% !important;
  max-width: 100% !important;
}

:deep(.el-select .el-input) {
  width: 100% !important;
}

:deep(.el-input__wrapper) {
  width: 100% !important;
  box-sizing: border-box;
}

.recognition-status {
  margin-top: 20px;
  padding: 16px;
  border: 2px solid #0fdc78;
  border-radius: 0;
}

.status-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
}

.status-label {
  font-weight: bold;
}

.progress-text {
  color: #409eff;
}

.error-message {
  margin-top: 8px;
  color: #f56c6c;
  font-size: 14px;
}
</style>