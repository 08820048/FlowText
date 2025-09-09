<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { ElMessage } from 'element-plus';
import { FolderOpened } from '@element-plus/icons-vue';
import { useSettingsStore } from '../stores';
import { validateApiKeys } from '../utils/recognitionUtils';
import { ProgressMonitor } from '../utils/progressMonitor';
import { ErrorHandler, ErrorType, ErrorSeverity } from '../utils/errorHandler';
import { themeManager } from '../utils/themeManager';
import { getDefaultExportPath } from '../utils/videoUtils';
import type { RecognitionEngine } from '../types';

// 引入设置存储
const settingsStore = useSettingsStore();

// 加载状态
const loading = ref({
  validate: false
});

// 多模型语音识别设置

// 当前默认引擎
const defaultEngine = ref(settingsStore.settings.defaultEngine || 'faster-whisper');



// 强制清除旧数据并重新初始化
function forceResetSettings() {
  console.log('强制重置设置...');
  localStorage.removeItem('flowtext-settings');

  // 重新初始化设置存储
  settingsStore.resetSettings();

  // 强制更新本地状态
  defaultEngine.value = 'faster-whisper';
  defaultLanguage.value = settingsStore.settings.defaultLanguage;
  defaultSubtitleFormat.value = settingsStore.settings.defaultSubtitleFormat;
  useGPU.value = settingsStore.settings.useGPU;
  maxConcurrentTasks.value = settingsStore.settings.maxConcurrentTasks;
  autoSave.value = settingsStore.settings.autoSave;
  autoSaveInterval.value = settingsStore.settings.autoSaveInterval;
  exportPath.value = settingsStore.settings.exportPath || '';

  console.log('设置已重置，新的默认引擎:', defaultEngine.value);
  ElMessage.success('设置已修复！默认引擎已设置为 Faster Whisper');
}

// 页面加载时检查并修复设置
if (settingsStore.settings.defaultEngine === 'tencent' || !settingsStore.settings.defaultEngine) {
  console.log('检测到无效的默认引擎，正在修复...');
  forceResetSettings();
}

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

// 当前默认模型大小
const defaultModelSize = ref(settingsStore.settings.defaultModelSize || 'base');

// 支持的模型大小
const supportedModelSizes = ref([
  { value: 'tiny', label: 'Tiny', description: '最快速度，基础精度' },
  { value: 'base', label: 'Base', description: '平衡速度和精度' },
  { value: 'small', label: 'Small', description: '较慢速度，较高精度' },
  { value: 'medium', label: 'Medium', description: '中等速度，高精度' },
  { value: 'large', label: 'Large', description: '最慢速度，最高精度' }
]);

// 当前默认语言
const defaultLanguage = ref(settingsStore.settings.defaultLanguage);

// 当前默认字幕格式
const defaultSubtitleFormat = ref(settingsStore.settings.defaultSubtitleFormat);

// 是否使用GPU
const useGPU = ref(settingsStore.settings.useGPU);

// 最大并发任务数
const maxConcurrentTasks = ref(settingsStore.settings.maxConcurrentTasks);

// 自动保存
const autoSave = ref(settingsStore.settings.autoSave);

// 自动保存间隔（秒）
const autoSaveInterval = ref(settingsStore.settings.autoSaveInterval);

// 字幕导出路径
const exportPath = ref(settingsStore.settings.exportPath || '');

// 默认导出路径
const defaultExportPath = ref('');

// 云服务API密钥相关功能已移除，只保留本地Whisper识别

// API密钥验证功能已移除（仅使用本地Whisper）

// 主题更新方法已移除

/**
 * 更新默认引擎（固定为whisper）
 */
function updateDefaultEngine() {
  // 固定使用whisper，无需更新
  ElMessage.success('默认引擎已设置为Whisper本地识别');
}

/**
 * 更新默认语言
 */
function updateDefaultLanguage() {
  settingsStore.setDefaultLanguage(defaultLanguage.value);
  ElMessage.success('默认语言已更新');
}

/**
 * 更新默认字幕格式
 */
function updateDefaultSubtitleFormat() {
  settingsStore.setDefaultSubtitleFormat(defaultSubtitleFormat.value);
  ElMessage.success('默认字幕格式已更新');
}

/**
 * 更新通用设置
 */
function updateGeneralSettings() {
  settingsStore.updateSettings({
    useGPU: useGPU.value,
    maxConcurrentTasks: maxConcurrentTasks.value,
    autoSave: autoSave.value,
    autoSaveInterval: autoSaveInterval.value,
    exportPath: exportPath.value
  });
  ElMessage.success('设置已更新');
}

/**
 * 选择导出文件夹
 */
async function selectExportFolder() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择字幕导出文件夹'
    });

    if (selected && typeof selected === 'string') {
      exportPath.value = selected;
      updateGeneralSettings();
      ElMessage.success('导出路径已设置');
    }
  } catch (error) {
    console.error('选择文件夹失败:', error);
    ElMessage.error('选择文件夹失败');
  }
}

/**
 * 打开导出文件夹
 */
async function openExportFolder() {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    // 使用自定义路径或默认路径
    const pathToOpen = exportPath.value || defaultExportPath.value;
    await invoke('open_folder', { path: pathToOpen });
    ElMessage.success('已打开导出文件夹');
  } catch (error) {
    console.error('打开文件夹失败:', error);
    ElMessage.error('打开文件夹失败');
  }
}

/**
 * 重置导出路径
 */
function resetExportPath() {
  exportPath.value = '';
  updateGeneralSettings();
  ElMessage.success('已重置为默认路径');
}

/**
 * 获取默认导出路径
 */
async function loadDefaultExportPath() {
  try {
    defaultExportPath.value = await getDefaultExportPath();
  } catch (error) {
    console.error('获取默认导出路径失败:', error);
    defaultExportPath.value = '应用程序目录';
  }
}

// 组件挂载时获取默认路径
onMounted(() => {
  loadDefaultExportPath();
});

/**
 * 重置所有设置
 */
function resetAllSettings() {
  // 清除本地存储中的旧数据
  localStorage.removeItem('flowtext-settings');

  settingsStore.resetSettings();

  // 更新本地状态
  defaultEngine.value = settingsStore.settings.defaultEngine;
  defaultLanguage.value = settingsStore.settings.defaultLanguage;
  defaultSubtitleFormat.value = settingsStore.settings.defaultSubtitleFormat;
  useGPU.value = settingsStore.settings.useGPU;
  maxConcurrentTasks.value = settingsStore.settings.maxConcurrentTasks;
  autoSave.value = settingsStore.settings.autoSave;
  autoSaveInterval.value = settingsStore.settings.autoSaveInterval;
  exportPath.value = settingsStore.settings.exportPath || '';
  
  apiKeys.value = {
    baidu: {
      appId: '',
      apiKey: '',
      secretKey: ''
    },
    tencent: {
      secretId: '',
      secretKey: ''
    },
    aliyun: {
      accessKeyId: '',
      accessKeySecret: ''
    }
  };
  
  ElMessage.success('所有设置已重置为默认值');
}
</script>

<template>
  <div class="settings-panel">
    <div class="settings-content">
      <el-tabs>
        <!-- 通用设置 -->
        <el-tab-pane label="通用设置">
          <el-form label-width="140px">
            <!-- 主题切换功能已移除，默认使用白色主题 -->
            
            <el-form-item label="默认识别引擎">
              <el-select v-model="defaultEngine" @change="updateDefaultEngine" style="width: 200px">
                <el-option label="Whisper (本地)" value="whisper" />
              </el-select>
            </el-form-item>
            
            <el-form-item label="默认语言">
              <el-select v-model="defaultLanguage" @change="updateDefaultLanguage" style="width: 200px">
                <el-option label="中文" value="zh" />
                <el-option label="英语" value="en" />
                <el-option label="日语" value="ja" />
                <el-option label="韩语" value="ko" />
                <el-option label="法语" value="fr" />
                <el-option label="德语" value="de" />
                <el-option label="西班牙语" value="es" />
                <el-option label="俄语" value="ru" />
              </el-select>
            </el-form-item>
            
            <el-form-item label="默认字幕格式">
              <el-select v-model="defaultSubtitleFormat" @change="updateDefaultSubtitleFormat" style="width: 200px">
                <el-option label="SRT" value="srt" />
                <el-option label="ASS" value="ass" />
                <el-option label="VTT" value="vtt" />
                <el-option label="TXT" value="txt" />
                <el-option label="JSON" value="json" />
              </el-select>
            </el-form-item>
            
            <el-divider />
            
            <el-form-item label="使用GPU加速">
              <el-switch v-model="useGPU" />
              <span class="setting-hint">（仅对Whisper引擎有效）</span>
            </el-form-item>
            
            <el-form-item label="最大并发任务数">
              <el-input-number v-model="maxConcurrentTasks" :min="1" :max="5" />
            </el-form-item>
            
            <el-form-item label="自动保存">
              <el-switch v-model="autoSave" />
            </el-form-item>
            
            <el-form-item label="自动保存间隔（秒）" v-if="autoSave">
              <el-input-number v-model="autoSaveInterval" :min="10" :max="300" :step="10" />
            </el-form-item>

            <el-divider />

            <el-form-item label="字幕导出路径">
              <div class="export-path-container">
                <div class="export-path-input-row">
                  <el-input
                    :value="exportPath || defaultExportPath"
                    :placeholder="exportPath ? '' : `默认路径: ${defaultExportPath}`"
                    readonly
                    style="flex: 1; margin-right: 8px;"
                  />
                  <el-button @click="selectExportFolder" size="small">选择文件夹</el-button>
                </div>
                <div class="export-path-actions">
                  <el-button @click="openExportFolder" size="small" type="success">
                    <el-icon><FolderOpened /></el-icon>
                    打开
                  </el-button>
                  <el-button @click="resetExportPath" size="small" v-if="exportPath">重置</el-button>
                </div>
              </div>
            </el-form-item>

            <el-form-item>
              <el-button type="primary" @click="updateGeneralSettings">保存设置</el-button>
              <el-button @click="resetAllSettings">重置所有设置</el-button>
              <el-button type="warning" @click="forceResetSettings">修复设置</el-button>
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <!-- 关于 -->
        <el-tab-pane label="关于">
          <div class="about-content">
            <h2>FlowText - 智能视频字幕提取工具</h2>
            <p>版本: 2.0.0</p>
            <p>FlowText是一款现代化桌面应用，支持多种AI语音识别模型，为视频内容创作者提供高效的字幕生成解决方案。</p>

            <h3>核心特性</h3>
            <ul>
              <li>🎥 智能视频导入和信息展示</li>
              <li>🎵 高质量音频提取</li>
              <li>🤖 多模型AI语音识别（Faster Whisper、OpenAI Whisper、SenseVoice）</li>
              <li>✏️ 实时字幕编辑和管理</li>
              <li>📄 多格式字幕导出（SRT、ASS、VTT等）</li>
              <li>🔧 智能配置推荐系统</li>
              <li>🛡️ 完全本地处理，保护隐私</li>
            </ul>

            <h3>支持的AI模型</h3>
            <ul>
              <li><strong>Faster Whisper</strong> - 优化版Whisper，速度提升4-5倍</li>
              <li><strong>OpenAI Whisper</strong> - 原版Whisper，稳定可靠</li>
              <li><strong>SenseVoice</strong> - 阿里巴巴模型，支持情感识别</li>
            </ul>

            <h3>技术架构</h3>
            <ul>
              <li>前端: Vue 3 + TypeScript + Element Plus</li>
              <li>桌面框架: Tauri 2.0</li>
              <li>后端: Rust</li>
              <li>AI引擎: Python + PyTorch</li>
            </ul>
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>
  </div>
</template>

<style scoped>
/* 扁平化设计样式 */
.settings-panel {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #ffffff;
  padding: 20px;
  box-sizing: border-box;
  overflow: hidden;
}

.settings-content {
  flex: 1;
}

.setting-hint {
  margin-left: 12px;
  color: #64748b;
  font-size: 13px;
  font-style: italic;
}

.about-content {
  padding: 24px;
  line-height: 1.6;
}

.about-content h2 {
  margin-top: 0;
  color: #1e293b;
  font-weight: 700;
}

.about-content h3 {
  color: #334155;
  font-weight: 600;
  margin-top: 24px;
  margin-bottom: 12px;
}

.about-content ul {
  padding-left: 20px;
}

.about-content li {
  margin-bottom: 8px;
  color: #475569;
}

/* 深度选择器优化Element Plus组件样式 */
:deep(.el-tabs) {
  display: flex;
  flex-direction: column;
  flex: 1;
  width: 100%;
  box-sizing: border-box;
}

/* 左侧tabs特殊处理 - 覆盖默认的column布局 */
:deep(.el-tabs--left) {
  display: flex !important;
  flex-direction: row !important;
  flex: 1;
}

:deep(.el-tabs__header) {
  margin: 0;
  background: transparent;
  padding: 0 16px;
  flex-shrink: 0;
  width: 100%;
  box-sizing: border-box;
  border-bottom: 1px solid #e5e7eb;
}

:deep(.el-tabs__nav-wrap) {
  padding: 0;
}

:deep(.el-tabs__item) {
  color: #6b7280;
  font-weight: 500;
  padding: 12px 16px;
  border: none;
  background: transparent;
  transition: all 0.2s ease;
}

:deep(.el-tabs__item:hover) {
  color: #374151;
  background: transparent;
}

:deep(.el-tabs__item.is-active) {
  color: #0fdc78;
  background: transparent;
  font-weight: 600;
}

:deep(.el-tabs__active-bar) {
  background: #0fdc78;
  height: 2px;
}

:deep(.el-tabs__content) {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  width: 100%;
  box-sizing: border-box;
}

:deep(.el-tab-pane) {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  width: 100%;
  box-sizing: border-box;
}

/* 嵌套tabs的tab-pane样式 */
:deep(.el-tabs--left .el-tab-pane) {
  flex: 1;
  overflow-y: auto;
  padding: 0;
  width: 100%;
  box-sizing: border-box;
}

:deep(.el-form-item) {
  margin-bottom: 20px;
  width: 100%;
  box-sizing: border-box;
}

:deep(.el-form) {
  width: 100%;
  box-sizing: border-box;
}

:deep(.el-input) {
  max-width: 100%;
  box-sizing: border-box;
}

:deep(.el-select) {
  max-width: 100%;
  box-sizing: border-box;
}

:deep(.el-button) {
  box-sizing: border-box;
}

/* 嵌套tabs样式优化 - 合并到上面的样式中 */

:deep(.el-tabs--left .el-tabs__header) {
  width: 140px;
  margin-right: 0;
  background: #f8fafc;
  border-right: 2px solid #0fdc78;
}

:deep(.el-tabs--left .el-tabs__content) {
  flex: 1;
  padding: 16px;
  overflow-y: auto;
}

:deep(.el-tabs--left .el-tabs__nav) {
  width: 100%;
}

:deep(.el-tabs--left .el-tabs__item) {
  text-align: left;
  padding: 10px 12px;
  color: #374151;
  background: transparent;
}

:deep(.el-tabs--left .el-tabs__item:hover) {
  background: rgba(15, 220, 120, 0.1);
  color: #000000;
}

:deep(.el-tabs--left .el-tabs__item.is-active) {
  background: #0fdc78;
  color: #000000;
  font-weight: 600;
}

:deep(.el-form-item__label) {
  color: #374151;
  font-weight: 600;
  font-size: 14px;
}

:deep(.el-input__wrapper) {
  border-radius: 0;
  transition: all 0.3s ease;
  border: 1px solid #e2e8f0;
}

:deep(.el-input__wrapper:hover) {
  border-color: #0fdc78;
  border-radius: 0;
}

:deep(.el-input__wrapper.is-focus) {
  border-color: #0fdc78;
  border-radius: 0;
}

:deep(.el-select .el-input__wrapper) {
  cursor: pointer;
}

:deep(.el-switch) {
  --el-switch-on-color: #0fdc78;
}

:deep(.el-input-number) {
  width: 120px;
}

:deep(.el-input-number .el-input__wrapper) {
  border-radius: 0;
}

:deep(.el-button) {
  border-radius: 0;
  font-weight: 500;
  border: 2px solid #0fdc78;
}

:deep(.el-button--primary) {
  background: #0fdc78;
  border-color: #0fdc78;
  color: #ffffff;
}

:deep(.el-button--primary:hover) {
  background: #ffffff;
  border-color: #0fdc78;
  color: #0fdc78;
}

:deep(.el-button:not(.el-button--primary)) {
  background: #ffffff;
  color: #0fdc78;
}

:deep(.el-button:not(.el-button--primary):hover) {
  background: #0fdc78;
  color: #ffffff;
}

:deep(.el-divider) {
  margin: 32px 0;
  border-color: #e2e8f0;
}

/* 导出路径设置样式 */
.export-path-container {
  width: 100%;
}

.export-path-input-row {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  margin-bottom: 8px;
}

.export-path-input-row .el-input {
  flex: 1;
}

.export-path-input-row .el-button {
  flex-shrink: 0;
}

.export-path-actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}

.export-path-actions .el-button {
  flex-shrink: 0;
}

/* 扁平化滚动条样式 */
.settings-content::-webkit-scrollbar {
  width: 8px;
}

.settings-content::-webkit-scrollbar-track {
  background: #ffffff;
}

.settings-content::-webkit-scrollbar-thumb {
  background: #0fdc78;
}

.settings-content::-webkit-scrollbar-thumb:hover {
  background: #0bc96a;
}

/* 主题预览样式已移除 */
</style>