<script setup lang="ts">
import { ref, computed } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { useVideoStore, useSettingsStore } from '../stores';
import { extractAudio } from '../utils/videoUtils';
import { startRecognition, getRecognitionStatus, cancelRecognition } from '../utils/recognitionUtils';
import { ProgressMonitor } from '../utils/progressMonitor';
import { ErrorHandler, ErrorType, ErrorSeverity } from '../utils/errorHandler';
import type { RecognitionEngine } from '../types';

// 引入存储
const videoStore = useVideoStore();
const settingsStore = useSettingsStore();

// 识别设置
const recognitionSettings = ref({
  engine: settingsStore.settings.defaultEngine as RecognitionEngine,
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
  return (
    videoStore.isVideoLoaded &&
    videoStore.selectedAudioTrackId !== null &&
    recognitionStatus.value !== 'extracting' &&
    recognitionStatus.value !== 'recognizing'
  );
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
  if (!videoStore.currentVideo || videoStore.selectedAudioTrackId === null) {
    ElMessage.warning('请先导入视频并选择音频轨道');
    return;
  }
  
  const progressTaskId = await ErrorHandler.withErrorHandling(async () => {
    // 检查API密钥是否配置
    const engine = recognitionSettings.value.engine;
    if (engine !== 'whisper' && !settingsStore.settings.apiKeys[engine]) {
      throw new Error(`请先在设置中配置${engine}的API密钥`);
    }
    
    // 确认开始识别
    await ElMessageBox.confirm(
      `将使用${engine === 'baidu' ? '百度智能云' : engine === 'tencent' ? '腾讯云' : engine === 'aliyun' ? '阿里云' : 'Whisper'}进行语音识别，是否继续？`,
      '开始识别',
      {
        confirmButtonText: '开始',
        cancelButtonText: '取消',
        type: 'info'
      }
    );
    
    // 创建进度任务
    const engineName = engine === 'baidu' ? '百度智能云' : engine === 'tencent' ? '腾讯云' : engine === 'aliyun' ? '阿里云' : 'Whisper';
    const progressTaskId = ProgressMonitor.createTask(
      `语音识别 - ${engineName}`,
      `正在识别视频: ${videoStore.currentVideo!.name}`,
      120000 // 预估2分钟
    );
    
    currentProgressTaskId.value = progressTaskId;
    
    // 重置状态
    recognitionStatus.value = 'extracting';
    recognitionProgress.value = 0;
    errorMessage.value = '';
    
    // 提取音频阶段
    ProgressMonitor.updateTask(progressTaskId, {
      progress: 10,
      message: '正在提取音频...'
    });
    
    loading.value.extract = true;
    const audioPath = await extractAudio(
      videoStore.currentVideo!.filePath,
      videoStore.selectedAudioTrackId
    );
    loading.value.extract = false;
    
    // 开始识别阶段
    ProgressMonitor.updateTask(progressTaskId, {
      progress: 30,
      message: '音频提取完成，开始语音识别...'
    });
    
    recognitionStatus.value = 'recognizing';
    loading.value.recognize = true;
    
    // 获取当前引擎的API密钥
    const apiKeys = settingsStore.settings.apiKeys[recognitionSettings.value.engine];
    
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
    
    return progressTaskId;
  }, {
    context: {
      component: 'RecognitionPanel',
      action: 'startRecognition',
      engine: recognitionSettings.value.engine,
      language: recognitionSettings.value.language
    },
    onError: (error) => {
      // 处理错误时的清理工作
      recognitionStatus.value = 'failed';
      errorMessage.value = error.message;
      loading.value.extract = false;
      loading.value.recognize = false;
      
      if (currentProgressTaskId.value) {
        ProgressMonitor.failTask(currentProgressTaskId.value, error.message);
        currentProgressTaskId.value = null;
      }
      
      ElMessage.error(error.message);
    }
  });
  
  if (!progressTaskId) {
    // 如果创建任务失败，重置状态
    recognitionStatus.value = 'idle';
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
      recognitionProgress.value = status.progress;
      
      // 更新进度监控任务
      const adjustedProgress = 30 + (status.progress * 0.7); // 30% 基础进度 + 70% 识别进度
      ProgressMonitor.updateTask(progressTaskId, {
        progress: adjustedProgress,
        message: `识别进度: ${Math.round(status.progress)}%`
      });
      
      // 更新任务状态
      videoStore.updateRecognitionTask(taskId, {
        status: status.status,
        progress: status.progress,
        subtitles: status.subtitles,
        error: status.error,
        updatedAt: new Date()
      });
      
      if (status.status === 'completed') {
        // 识别完成
        recognitionStatus.value = 'completed';
        clearInterval(checkInterval);
        
        // 设置字幕
        if (status.subtitles && status.subtitles.length > 0) {
          videoStore.setSubtitles(status.subtitles);
          
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
        
        // 记录错误并失败进度任务
        const errorMsg = status.error || '未知错误';
        ErrorHandler.handleError(
          new Error(errorMsg),
          ErrorType.RECOGNITION_ERROR,
          ErrorSeverity.HIGH,
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
      
      const errorMsg = `获取识别状态失败: ${error}`;
      errorMessage.value = errorMsg;
      
      // 记录错误并失败进度任务
      ErrorHandler.handleError(
        error instanceof Error ? error : new Error(String(error)),
        ErrorType.NETWORK_ERROR,
        ErrorSeverity.HIGH,
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
  if (!currentTaskId.value) return;
  
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
    <div class="panel-header">
      <h3>语音识别</h3>
    </div>
    
    <div v-if="!videoStore.isVideoLoaded" class="no-video">
      <el-empty description="请先导入视频" />
    </div>
    
    <div v-else class="recognition-content">
      <el-form label-width="100px">
        <el-form-item label="识别引擎">
          <el-select v-model="recognitionSettings.engine" style="width: 100%">
            <el-option label="百度智能云" value="baidu" />
            <el-option label="腾讯云" value="tencent" />
            <el-option label="阿里云" value="aliyun" />
            <el-option label="Whisper" value="whisper" />
          </el-select>
        </el-form-item>
        
        <el-form-item label="识别语言">
          <el-select v-model="recognitionSettings.language" style="width: 100%">
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
          
          <span v-if="recognitionStatus === 'recognizing'" class="progress-text">
            {{ Math.round(recognitionProgress) }}%
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
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid #ebeef5;
}

.panel-header h3 {
  margin: 0;
}

.no-video {
  display: flex;
  justify-content: center;
  align-items: center;
  flex: 1;
  min-height: 200px;
}

.recognition-content {
  padding: 16px;
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