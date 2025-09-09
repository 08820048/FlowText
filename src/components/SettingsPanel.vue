<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import { FolderOpened } from '@element-plus/icons-vue';
import { useSettingsStore } from '../stores';
import { validateApiKeys } from '../utils/recognitionUtils';
import { ProgressMonitor } from '../utils/progressMonitor';
import { ErrorHandler, ErrorType, ErrorSeverity } from '../utils/errorHandler';
import { themeManager } from '../utils/themeManager';
import type { RecognitionEngine } from '../types';

// 引入设置存储
const settingsStore = useSettingsStore();

// 加载状态
const loading = ref({
  validate: false
});

// Whisper设置（本地识别，无需API密钥）

// 主题切换功能已移除

// 当前默认引擎（固定为whisper）
const defaultEngine = ref('whisper');

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
  if (!exportPath.value) {
    ElMessage.warning('请先设置导出路径');
    return;
  }

  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('open_folder', { path: exportPath.value });
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
 * 重置所有设置
 */
function resetAllSettings() {
  settingsStore.resetSettings();
  
  // 更新本地状态（固定为whisper）
  defaultEngine.value = 'whisper';
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
                    v-model="exportPath"
                    :placeholder="exportPath ? '' : '默认路径（留空使用系统默认位置）'"
                    readonly
                    style="flex: 1; margin-right: 8px;"
                  />
                  <el-button @click="selectExportFolder" size="small">选择文件夹</el-button>
                </div>
                <div class="export-path-actions" v-if="exportPath">
                  <el-button @click="openExportFolder" size="small" type="success">
                    <el-icon><FolderOpened /></el-icon>
                    打开
                  </el-button>
                  <el-button @click="resetExportPath" size="small">重置</el-button>
                </div>
              </div>
              <div class="setting-hint">
                设置字幕文件的默认导出位置，留空则使用应用程序目录
              </div>
            </el-form-item>

            <el-form-item>
              <el-button type="primary" @click="updateGeneralSettings">保存设置</el-button>
              <el-button @click="resetAllSettings">重置所有设置</el-button>
            </el-form-item>
          </el-form>
        </el-tab-pane>
        
        <!-- Whisper设置 -->
        <el-tab-pane label="Whisper">
          <div class="whisper-settings">
            <el-alert
              title="Whisper本地语音识别"
              type="success"
              :closable="false"
              show-icon
            >
              <template #default>
                <div>
                  <p><strong>优势：</strong></p>
                  <p>• 完全本地处理，无需网络连接</p>
                  <p>• 支持任意大小的音频文件</p>
                  <p>• 识别准确率高，支持多种语言</p>
                  <p>• 无API调用限制和费用</p>
                  <p>• 保护隐私，数据不会上传到云端</p>
                  <br>
                  <p><strong>支持的语言：</strong></p>
                  <p>中文、英文、日文、韩文、法文、德文、西班牙文等100+种语言</p>
                </div>
              </template>
            </el-alert>

            <el-form label-width="100px" style="margin-top: 20px;">
              <el-form-item label="模型大小">
                <el-radio-group v-model="settingsStore.settings.whisperModel" @change="updateGeneralSettings">
                  <el-radio-button label="tiny">Tiny</el-radio-button>
                  <el-radio-button label="base">Base</el-radio-button>
                  <el-radio-button label="small">Small</el-radio-button>
                </el-radio-group>
              </el-form-item>

              <el-alert
                title="关于Whisper"
                type="info"
                description="Whisper是OpenAI开发的开源语音识别模型，可以在本地运行，无需API密钥。模型大小影响识别精度和速度，Tiny最快但精度较低，Small最慢但精度最高。"
                :closable="false"
                show-icon
              />
            </el-form>
          </div>
        </el-tab-pane>
        
        <!-- 关于 -->
        <el-tab-pane label="关于">
          <div class="about-content">
            <h2>FlowText - 智能视频字幕提取工具</h2>
            <p>版本: 1.0.0</p>
            <p>FlowText是一款桌面应用，用于从视频中提取音频并生成字幕。使用本地Whisper模型进行语音识别，完全离线处理，保护您的隐私。</p>
            
            <h3>主要功能</h3>
            <ul>
              <li>视频导入和信息展示</li>
              <li>音频提取</li>
              <li>Whisper本地语音识别</li>
              <li>字幕编辑和管理</li>
              <li>多格式字幕导出</li>
            </ul>
            
            <h3>技术栈</h3>
            <ul>
              <li>前端: Vue 3 + TypeScript + Element Plus</li>
              <li>桌面框架: Tauri 2.0</li>
              <li>后端: Rust</li>
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
  background: #f8fafc;
  padding: 20px;
  box-sizing: border-box;
}

.settings-content {
  flex: 1;
  overflow: hidden;
  background: #ffffff;
  border: 1px solid rgba(99, 102, 241, 0.1);
  display: flex;
  flex-direction: column;
  width: 100%;
  box-sizing: border-box;
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
  background: #667eea;
  padding: 0 16px;
  flex-shrink: 0;
  width: 100%;
  box-sizing: border-box;
}

:deep(.el-tabs__nav-wrap) {
  padding: 0;
}

:deep(.el-tabs__item) {
  color: rgba(255, 255, 255, 0.8);
  font-weight: 500;
  padding: 12px 16px;
  border: none;
  transition: all 0.3s ease;
}

:deep(.el-tabs__item:hover) {
  color: white;
}

:deep(.el-tabs__item.is-active) {
  color: white;
  background: rgba(255, 255, 255, 0.1);
}

:deep(.el-tabs__active-bar) {
  background: white;
  height: 3px;
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