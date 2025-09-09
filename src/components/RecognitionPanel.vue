<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Microphone, Close, InfoFilled, Setting } from '@element-plus/icons-vue';
import { useVideoStore, useSettingsStore } from '../stores';
import { extractAudio } from '../utils/videoUtils';
import { startRecognition, startRecognitionWithConfig, getRecognitionStatus, cancelRecognition } from '../utils/recognitionUtils';
import { ProgressMonitor } from '../utils/progressMonitor';
import { ErrorHandler, ErrorType, ErrorSeverity, withErrorHandling } from '../utils/errorHandler';
import { modelManager } from '../utils/modelManager';
import { ModelApi } from '../utils/modelApi';
import type { RecognitionEngine, ModelConfig, ModelSize, ExtendedRecognitionParams } from '../types';

// 引入存储
const videoStore = useVideoStore();
const settingsStore = useSettingsStore();

// 识别设置
const recognitionSettings = ref({
  engine: 'faster-whisper' as RecognitionEngine, // 默认使用faster-whisper
  language: settingsStore.settings.defaultLanguage,
  modelSize: 'base',
  advancedSettings: {
    device: 'cpu' as 'cpu' | 'gpu',
    computeType: 'int8' as 'int8' | 'int16' | 'float16' | 'float32',
    beamSize: 5,
    temperature: 0.0,
    enableEmotionRecognition: false,
    enableEventDetection: false
  }
});

// 当前选中的模型配置
const currentModel = ref<ModelConfig | null>(null);
const availableModelSizes = ref<ModelSize[]>([]);
const showAdvancedSettings = ref(false);

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

// 支持的引擎列表
const supportedEngines = ref([
  {
    value: 'faster-whisper',
    label: 'Faster Whisper',
    description: '优化版Whisper，速度提升4-5倍',
    icon: '🚀'
  },
  {
    value: 'whisper',
    label: 'OpenAI Whisper',
    description: '原版Whisper，稳定可靠',
    icon: '🎯'
  },
  {
    value: 'sensevoice',
    label: 'SenseVoice',
    description: '阿里巴巴模型，支持情感识别',
    icon: '🧠'
  }
]);

// 支持的语言列表（动态根据模型更新）
const supportedLanguages = computed(() => {
  if (!currentModel.value) return [];

  const languageNames: Record<string, string> = {
    'zh': '中文', 'en': '英语', 'ja': '日语', 'ko': '韩语',
    'fr': '法语', 'de': '德语', 'es': '西班牙语', 'ru': '俄语',
    'it': '意大利语', 'pt': '葡萄牙语', 'ar': '阿拉伯语', 'hi': '印地语',
    'th': '泰语', 'vi': '越南语', 'tr': '土耳其语', 'pl': '波兰语',
    'nl': '荷兰语', 'sv': '瑞典语', 'da': '丹麦语', 'no': '挪威语'
  };

  return currentModel.value.languages.map(code => ({
    code,
    name: languageNames[code] || code.toUpperCase()
  }));
});

// 监听引擎变化，更新模型配置
watch(() => recognitionSettings.value.engine, (newEngine) => {
  updateModelConfig(newEngine);
}, { immediate: true });

// 监听模型大小变化，更新高级设置的可用选项
watch(() => recognitionSettings.value.modelSize, (newSize) => {
  updateAdvancedSettingsForSize(newSize);
});

/**
 * 更新模型配置
 */
function updateModelConfig(engine: RecognitionEngine) {
  const model = modelManager.getModel(engine);
  if (model) {
    currentModel.value = model;
    availableModelSizes.value = model.sizes;

    // 如果当前选择的大小不在新模型中，选择第一个可用的
    const currentSize = recognitionSettings.value.modelSize;
    if (!model.sizes.find(s => s.id === currentSize)) {
      recognitionSettings.value.modelSize = model.sizes[0]?.id || 'base';
    }

    // 根据模型特性更新高级设置的可用性
    updateAdvancedSettingsAvailability(model);
  }
}

/**
 * 更新高级设置的可用性
 */
function updateAdvancedSettingsAvailability(model: ModelConfig) {
  // 重置高级设置
  if (model.name === 'sensevoice') {
    recognitionSettings.value.advancedSettings.enableEmotionRecognition = true;
    recognitionSettings.value.advancedSettings.enableEventDetection = true;
  } else {
    recognitionSettings.value.advancedSettings.enableEmotionRecognition = false;
    recognitionSettings.value.advancedSettings.enableEventDetection = false;
  }

  // 根据模型调整默认计算类型
  if (model.name === 'faster-whisper') {
    recognitionSettings.value.advancedSettings.computeType = 'int8';
  } else {
    recognitionSettings.value.advancedSettings.computeType = 'float32';
  }
}

/**
 * 根据模型大小更新高级设置
 */
function updateAdvancedSettingsForSize(size: string) {
  // 大模型建议使用更高精度的计算类型
  if (['large', 'large-v2', 'large-v3'].includes(size)) {
    if (recognitionSettings.value.engine === 'faster-whisper') {
      recognitionSettings.value.advancedSettings.computeType = 'float16';
    }
  }
}

/**
 * 获取模型性能信息
 */
const modelPerformanceInfo = computed(() => {
  if (!currentModel.value) return null;

  const size = availableModelSizes.value.find(s => s.id === recognitionSettings.value.modelSize);
  if (!size) return null;

  return {
    model: currentModel.value,
    size: size,
    performance: currentModel.value.performance,
    features: currentModel.value.features.filter(f => f.supported)
  };
});

/**
 * 检查模型是否已安装
 */
async function checkModelInstallation(engine: RecognitionEngine): Promise<boolean> {
  // 这里应该调用后端API检查模型是否已安装
  // 暂时返回true，实际实现需要检查Python包和模型文件
  return true;
}

/**
 * 获取推荐配置
 */
function getRecommendedConfig() {
  const recommendation = modelManager.getRecommendedModel({
    speed: 'balanced',
    memory: 'medium',
    features: []
  });

  recognitionSettings.value.engine = recommendation.model as RecognitionEngine;
  recognitionSettings.value.modelSize = recommendation.size;

  ElMessage.success('已应用推荐配置');
}

/**
 * 获取速度标签类型
 */
function getSpeedTagType(speed: string): string {
  const typeMap: Record<string, string> = {
    'very-fast': 'success',
    'fast': 'success',
    'medium': 'warning',
    'slow': 'danger',
    'very-slow': 'danger'
  };
  return typeMap[speed] || 'info';
}

/**
 * 获取速度文本
 */
function getSpeedText(speed: string): string {
  const textMap: Record<string, string> = {
    'very-fast': '极快',
    'fast': '快速',
    'medium': '中等',
    'slow': '较慢',
    'very-slow': '很慢'
  };
  return textMap[speed] || speed;
}

/**
 * 获取精度标签类型
 */
function getAccuracyTagType(accuracy: string): string {
  const typeMap: Record<string, string> = {
    'basic': 'info',
    'good': 'success',
    'high': 'success',
    'very-high': 'warning',
    'excellent': 'warning'
  };
  return typeMap[accuracy] || 'info';
}

/**
 * 获取精度文本
 */
function getAccuracyText(accuracy: string): string {
  const textMap: Record<string, string> = {
    'basic': '基础',
    'good': '良好',
    'high': '高精度',
    'very-high': '很高',
    'excellent': '极佳'
  };
  return textMap[accuracy] || accuracy;
}

/**
 * 开始识别流程
 */
async function startRecognitionProcess() {
  console.log('=== 开始识别按钮点击 ===');
  console.log('当前视频:', videoStore.currentVideo);
  console.log('选择的音频轨道ID:', videoStore.selectedAudioTrackId);
  console.log('识别设置:', recognitionSettings.value);

  if (!videoStore.currentVideo || videoStore.selectedAudioTrackId === null) {
    ElMessage.warning('请先导入视频并选择音频轨道');
    return;
  }

  try {
    const engine = recognitionSettings.value.engine;
    const modelSize = recognitionSettings.value.modelSize;
    const modelName = currentModel.value?.displayName || engine;

    console.log('使用引擎:', engine, '模型大小:', modelSize);

    // 确认开始识别
    await ElMessageBox.confirm(
      `将使用${modelName} (${modelSize})模型进行语音识别，是否继续？`,
      '开始识别',
      {
        confirmButtonText: '开始',
        cancelButtonText: '取消',
        type: 'info'
      }
    );
    
    // 创建进度任务
    const progressTaskId = ProgressMonitor.createTask(
      `语音识别 - ${modelName}`,
      120000, // 预估2分钟
      {
        video: videoStore.currentVideo!.fileName,
        engine,
        modelSize,
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
    
    // 构建扩展识别参数
    const recognitionParams: ExtendedRecognitionParams = {
      audio_path: audioPath,
      engine: engine,
      language: recognitionSettings.value.language,
      model_config: {
        engine: engine,
        size: modelSize,
        device: recognitionSettings.value.advancedSettings.device,
        compute_type: recognitionSettings.value.advancedSettings.computeType,
        beam_size: recognitionSettings.value.advancedSettings.beamSize,
        temperature: recognitionSettings.value.advancedSettings.temperature,
        enable_emotion_recognition: recognitionSettings.value.advancedSettings.enableEmotionRecognition,
        enable_event_detection: recognitionSettings.value.advancedSettings.enableEventDetection
      }
    };

    console.log('识别参数:', recognitionParams);

    // 使用扩展配置启动识别
    const task = await startRecognitionWithConfig(recognitionParams);
    currentTaskId.value = task.id;
    videoStore.addRecognitionTask(task);

    console.log('识别任务已启动，任务ID:', task.id);

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
        <!-- 模型选择 -->
        <el-form-item label="识别模型">
          <el-select v-model="recognitionSettings.engine" class="form-select">
            <el-option
              v-for="engine in supportedEngines"
              :key="engine.value"
              :label="engine.label"
              :value="engine.value"
            >
              <div class="engine-option">
                <span class="engine-icon">{{ engine.icon }}</span>
                <div class="engine-info">
                  <div class="engine-name">{{ engine.label }}</div>
                  <div class="engine-desc">{{ engine.description }}</div>
                </div>
              </div>
            </el-option>
          </el-select>
        </el-form-item>

        <!-- 模型大小选择 -->
        <el-form-item label="模型大小" v-if="availableModelSizes.length > 0">
          <el-select v-model="recognitionSettings.modelSize" class="form-select">
            <el-option
              v-for="size in availableModelSizes"
              :key="size.id"
              :label="size.displayName"
              :value="size.id"
            >
              <div class="size-option">
                <div class="size-info">
                  <div class="size-name">
                    {{ size.displayName }}
                    <el-tag :type="getSpeedTagType(size.speed)" size="small">
                      {{ getSpeedText(size.speed) }}
                    </el-tag>
                    <el-tag :type="getAccuracyTagType(size.accuracy)" size="small">
                      {{ getAccuracyText(size.accuracy) }}
                    </el-tag>
                  </div>
                  <div class="size-desc">{{ size.description }}</div>
                  <div class="size-stats">
                    <span>{{ size.fileSize }}</span> • <span>{{ size.memoryUsage }}</span>
                  </div>
                </div>
              </div>
            </el-option>
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

        <!-- 高级设置切换 -->
        <el-form-item>
          <el-button
            type="text"
            :icon="Setting"
            @click="showAdvancedSettings = !showAdvancedSettings"
            class="advanced-toggle"
          >
            {{ showAdvancedSettings ? '隐藏' : '显示' }}高级设置
          </el-button>
        </el-form-item>

        <!-- 高级设置面板 -->
        <div v-show="showAdvancedSettings" class="advanced-settings">
          <el-divider content-position="left">高级设置</el-divider>

          <!-- 设备选择 -->
          <el-form-item label="计算设备">
            <el-radio-group v-model="recognitionSettings.advancedSettings.device">
              <el-radio label="cpu">CPU</el-radio>
              <el-radio label="gpu">GPU</el-radio>
            </el-radio-group>
          </el-form-item>

          <!-- 计算精度 -->
          <el-form-item label="计算精度" v-if="recognitionSettings.engine === 'faster-whisper'">
            <el-select v-model="recognitionSettings.advancedSettings.computeType" class="form-select">
              <el-option label="INT8 (最快)" value="int8" />
              <el-option label="INT16 (平衡)" value="int16" />
              <el-option label="FLOAT16 (高精度)" value="float16" />
              <el-option label="FLOAT32 (最高精度)" value="float32" />
            </el-select>
          </el-form-item>

          <!-- Beam Size -->
          <el-form-item label="Beam Size" v-if="recognitionSettings.engine === 'faster-whisper'">
            <el-slider
              v-model="recognitionSettings.advancedSettings.beamSize"
              :min="1"
              :max="10"
              :step="1"
              show-input
              class="form-slider"
            />
          </el-form-item>

          <!-- Temperature -->
          <el-form-item label="Temperature" v-if="recognitionSettings.engine !== 'sensevoice'">
            <el-slider
              v-model="recognitionSettings.advancedSettings.temperature"
              :min="0"
              :max="1"
              :step="0.1"
              show-input
              class="form-slider"
            />
          </el-form-item>

          <!-- SenseVoice 特有设置 -->
          <template v-if="recognitionSettings.engine === 'sensevoice'">
            <el-form-item label="情感识别">
              <el-switch v-model="recognitionSettings.advancedSettings.enableEmotionRecognition" />
            </el-form-item>

            <el-form-item label="事件检测">
              <el-switch v-model="recognitionSettings.advancedSettings.enableEventDetection" />
            </el-form-item>
          </template>
        </div>

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

          <el-button
            type="info"
            @click="getRecommendedConfig"
            class="recommend-btn"
          >
            推荐配置
          </el-button>
        </el-form-item>
      </el-form>

      <!-- 模型性能信息 -->
      <div v-if="modelPerformanceInfo" class="model-info">
        <el-card shadow="never" class="info-card">
          <template #header>
            <div class="info-header">
              <el-icon><InfoFilled /></el-icon>
              <span>模型信息</span>
            </div>
          </template>

          <div class="model-details">
            <div class="detail-row">
              <span class="label">提供商:</span>
              <span class="value">{{ modelPerformanceInfo.model.displayName }}</span>
            </div>
            <div class="detail-row">
              <span class="label">当前配置:</span>
              <span class="value">{{ modelPerformanceInfo.size.displayName }}</span>
            </div>
            <div class="detail-row">
              <span class="label">预期精度:</span>
              <span class="value">WER ~{{ (modelPerformanceInfo.performance.wer! * 100).toFixed(1) }}%</span>
            </div>
            <div class="detail-row">
              <span class="label">处理速度:</span>
              <span class="value">{{ modelPerformanceInfo.performance.throughput }} 词/秒</span>
            </div>

            <!-- 特性标签 -->
            <div class="features">
              <el-tag
                v-for="feature in modelPerformanceInfo.features"
                :key="feature.id"
                size="small"
                class="feature-tag"
              >
                {{ feature.name }}
              </el-tag>
            </div>
          </div>
        </el-card>
      </div>
      
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
  overflow: hidden;
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

/* 引擎选项样式 */
.engine-option {
  display: flex;
  align-items: center;
  padding: 4px 0;
}

.engine-icon {
  font-size: 18px;
  margin-right: 8px;
}

.engine-info {
  flex: 1;
}

.engine-name {
  font-weight: 500;
  color: #303133;
}

.engine-desc {
  font-size: 12px;
  color: #909399;
  margin-top: 2px;
}

/* 模型大小选项样式 */
.size-option {
  padding: 4px 0;
}

.size-info {
  width: 100%;
}

.size-name {
  display: flex;
  align-items: center;
  gap: 6px;
  font-weight: 500;
  color: #303133;
  margin-bottom: 4px;
}

.size-desc {
  font-size: 12px;
  color: #606266;
  margin-bottom: 4px;
}

.size-stats {
  font-size: 11px;
  color: #909399;
}

/* 高级设置样式 */
.advanced-settings {
  background: #f8f9fa;
  border-radius: 6px;
  padding: 16px;
  margin: 16px 0;
}

.advanced-toggle {
  padding: 0;
  font-size: 13px;
}

/* 模型信息卡片样式 */
.model-info {
  margin-top: 20px;
}

.info-card {
  border: 1px solid #e4e7ed;
}

.info-header {
  display: flex;
  align-items: center;
  gap: 6px;
  font-weight: 500;
  color: #303133;
}

.model-details {
  font-size: 13px;
}

.detail-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.detail-row:last-child {
  margin-bottom: 12px;
}

.label {
  color: #606266;
  font-weight: 500;
}

.value {
  color: #303133;
}

.features {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 8px;
}

.feature-tag {
  font-size: 11px;
}

.recommend-btn {
  margin-left: 8px;
}
</style>