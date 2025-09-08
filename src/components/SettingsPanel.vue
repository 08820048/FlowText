<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
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

// 当前编辑的API密钥
const apiKeys = ref({
  baidu: {
    appId: settingsStore.settings.apiKeys.baidu?.appId || '',
    apiKey: settingsStore.settings.apiKeys.baidu?.apiKey || '',
    secretKey: settingsStore.settings.apiKeys.baidu?.secretKey || ''
  },
  tencent: {
    secretId: settingsStore.settings.apiKeys.tencent?.secretId || '',
    secretKey: settingsStore.settings.apiKeys.tencent?.secretKey || ''
  },
  aliyun: {
    accessKeyId: settingsStore.settings.apiKeys.aliyun?.accessKeyId || '',
    accessKeySecret: settingsStore.settings.apiKeys.aliyun?.accessKeySecret || ''
  }
});

// 当前主题
const currentTheme = ref(themeManager.getCurrentTheme());

// 当前默认引擎
const defaultEngine = ref(settingsStore.settings.defaultEngine);

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

/**
 * 保存百度API密钥
 */
async function saveBaiduApiKeys() {
  settingsStore.setApiKeys('baidu', apiKeys.value.baidu);
  ElMessage.success('百度API密钥已保存');
}

/**
 * 保存腾讯API密钥
 */
async function saveTencentApiKeys() {
  settingsStore.setApiKeys('tencent', apiKeys.value.tencent);
  ElMessage.success('腾讯API密钥已保存');
}

/**
 * 保存阿里API密钥
 */
async function saveAliyunApiKeys() {
  settingsStore.setApiKeys('aliyun', apiKeys.value.aliyun);
  ElMessage.success('阿里API密钥已保存');
}

/**
 * 验证API密钥
 * @param engine 识别引擎
 */
async function validateApiKey(engine: RecognitionEngine) {
  await ErrorHandler.withErrorHandling(async () => {
    loading.value.validate = true;
    
    // 创建进度任务
    const progressTaskId = ProgressMonitor.createTask(
      `验证${engine}API密钥`,
      `正在验证${engine}API密钥...`,
      5000 // 预估5秒
    );
    
    try {
      const keys = apiKeys.value[engine];
      
      ProgressMonitor.updateTask(progressTaskId, {
        progress: 50,
        message: `正在连接${engine}服务器...`
      });
      
      const isValid = await validateApiKeys(engine, keys);
      
      if (isValid) {
        ProgressMonitor.completeTask(progressTaskId, `${engine}API密钥验证成功`);
        ElMessage.success(`${engine}API密钥验证成功`);
      } else {
        ProgressMonitor.failTask(progressTaskId, `${engine}API密钥验证失败`);
        ElMessage.error(`${engine}API密钥验证失败`);
      }
    } catch (error) {
      ProgressMonitor.failTask(progressTaskId, `验证失败: ${error}`);
      throw error;
    } finally {
      loading.value.validate = false;
    }
  }, {
    context: {
      component: 'SettingsPanel',
      action: 'validateApiKey',
      engine
    },
    onError: (error) => {
      loading.value.validate = false;
      ElMessage.error(`验证${engine}API密钥失败: ${error.message}`);
    }
  });
}

/**
 * 更新主题
 */
function updateTheme() {
  themeManager.setTheme(currentTheme.value);
  settingsStore.setTheme(currentTheme.value);
  ElMessage.success('主题已更新');
}

/**
 * 更新默认引擎
 */
function updateDefaultEngine() {
  settingsStore.setDefaultEngine(defaultEngine.value);
  ElMessage.success('默认引擎已更新');
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
    autoSaveInterval: autoSaveInterval.value
  });
  ElMessage.success('设置已更新');
}

/**
 * 重置所有设置
 */
function resetAllSettings() {
  settingsStore.resetSettings();
  
  // 更新本地状态
  currentTheme.value = settingsStore.settings.theme;
  defaultEngine.value = settingsStore.settings.defaultEngine;
  defaultLanguage.value = settingsStore.settings.defaultLanguage;
  defaultSubtitleFormat.value = settingsStore.settings.defaultSubtitleFormat;
  useGPU.value = settingsStore.settings.useGPU;
  maxConcurrentTasks.value = settingsStore.settings.maxConcurrentTasks;
  autoSave.value = settingsStore.settings.autoSave;
  autoSaveInterval.value = settingsStore.settings.autoSaveInterval;
  
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
    <div class="panel-header">
      <h3>应用设置</h3>
    </div>
    
    <div class="settings-content">
      <el-tabs>
        <!-- 通用设置 -->
        <el-tab-pane label="通用设置">
          <el-form label-width="140px">
            <el-form-item label="主题">
              <el-radio-group v-model="currentTheme" @change="updateTheme">
                <el-radio-button label="light">浅色</el-radio-button>
                <el-radio-button label="dark">深色</el-radio-button>
              </el-radio-group>
              <div class="theme-preview" style="margin-top: 12px;">
                <div class="theme-card light" :class="{ active: currentTheme === 'light' }" @click="currentTheme = 'light'; updateTheme()">
                  <div class="theme-header"></div>
                  <div class="theme-content">
                    <div class="theme-sidebar"></div>
                    <div class="theme-main"></div>
                  </div>
                </div>
                <div class="theme-card dark" :class="{ active: currentTheme === 'dark' }" @click="currentTheme = 'dark'; updateTheme()">
                  <div class="theme-header"></div>
                  <div class="theme-content">
                    <div class="theme-sidebar"></div>
                    <div class="theme-main"></div>
                  </div>
                </div>
              </div>
            </el-form-item>
            
            <el-form-item label="默认识别引擎">
              <el-select v-model="defaultEngine" @change="updateDefaultEngine" style="width: 200px">
                <el-option label="百度智能云" value="baidu" />
                <el-option label="腾讯云" value="tencent" />
                <el-option label="阿里云" value="aliyun" />
                <el-option label="Whisper" value="whisper" />
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
            
            <el-form-item>
              <el-button type="primary" @click="updateGeneralSettings">保存设置</el-button>
              <el-button @click="resetAllSettings">重置所有设置</el-button>
            </el-form-item>
          </el-form>
        </el-tab-pane>
        
        <!-- API密钥设置 -->
        <el-tab-pane label="API密钥设置">
          <el-tabs tab-position="left">
            <!-- 百度智能云 -->
            <el-tab-pane label="百度智能云">
              <el-form label-width="120px">
                <el-form-item label="App ID">
                  <el-input v-model="apiKeys.baidu.appId" placeholder="请输入百度智能云App ID" />
                </el-form-item>
                
                <el-form-item label="API Key">
                  <el-input v-model="apiKeys.baidu.apiKey" placeholder="请输入百度智能云API Key" />
                </el-form-item>
                
                <el-form-item label="Secret Key">
                  <el-input v-model="apiKeys.baidu.secretKey" placeholder="请输入百度智能云Secret Key" show-password />
                </el-form-item>
                
                <el-form-item>
                  <el-button type="primary" @click="saveBaiduApiKeys">保存</el-button>
                  <el-button @click="validateApiKey('baidu')" :loading="loading.validate">验证</el-button>
                </el-form-item>
                
                <el-alert
                  title="如何获取百度智能云API密钥"
                  type="info"
                  description="1. 登录百度智能云控制台 2. 创建语音识别应用 3. 获取应用的App ID、API Key和Secret Key"
                  :closable="false"
                  show-icon
                />
              </el-form>
            </el-tab-pane>
            
            <!-- 腾讯云 -->
            <el-tab-pane label="腾讯云">
              <el-form label-width="120px">
                <el-form-item label="Secret ID">
                  <el-input v-model="apiKeys.tencent.secretId" placeholder="请输入腾讯云Secret ID" />
                </el-form-item>
                
                <el-form-item label="Secret Key">
                  <el-input v-model="apiKeys.tencent.secretKey" placeholder="请输入腾讯云Secret Key" show-password />
                </el-form-item>
                
                <el-form-item>
                  <el-button type="primary" @click="saveTencentApiKeys">保存</el-button>
                  <el-button @click="validateApiKey('tencent')" :loading="loading.validate">验证</el-button>
                </el-form-item>
                
                <el-alert
                  title="如何获取腾讯云API密钥"
                  type="info"
                  description="1. 登录腾讯云控制台 2. 访问'访问密钥'页面 3. 创建并获取Secret ID和Secret Key"
                  :closable="false"
                  show-icon
                />
              </el-form>
            </el-tab-pane>
            
            <!-- 阿里云 -->
            <el-tab-pane label="阿里云">
              <el-form label-width="140px">
                <el-form-item label="Access Key ID">
                  <el-input v-model="apiKeys.aliyun.accessKeyId" placeholder="请输入阿里云Access Key ID" />
                </el-form-item>
                
                <el-form-item label="Access Key Secret">
                  <el-input v-model="apiKeys.aliyun.accessKeySecret" placeholder="请输入阿里云Access Key Secret" show-password />
                </el-form-item>
                
                <el-form-item>
                  <el-button type="primary" @click="saveAliyunApiKeys">保存</el-button>
                  <el-button @click="validateApiKey('aliyun')" :loading="loading.validate">验证</el-button>
                </el-form-item>
                
                <el-alert
                  title="如何获取阿里云API密钥"
                  type="info"
                  description="1. 登录阿里云控制台 2. 访问'AccessKey管理'页面 3. 创建并获取Access Key ID和Access Key Secret"
                  :closable="false"
                  show-icon
                />
              </el-form>
            </el-tab-pane>
            
            <!-- Whisper设置 -->
            <el-tab-pane label="Whisper设置">
              <el-form label-width="120px">
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
            </el-tab-pane>
          </el-tabs>
        </el-tab-pane>
        
        <!-- 关于 -->
        <el-tab-pane label="关于">
          <div class="about-content">
            <h2>FlowText - 智能视频字幕提取工具</h2>
            <p>版本: 1.0.0</p>
            <p>FlowText是一款桌面应用，用于从视频中提取音频并生成字幕。支持多种语音识别引擎，包括百度智能云、腾讯云、阿里云和本地Whisper模型。</p>
            
            <h3>主要功能</h3>
            <ul>
              <li>视频导入和信息展示</li>
              <li>音频提取</li>
              <li>多引擎语音识别</li>
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
.settings-panel {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
  border-radius: 12px;
  padding: 20px;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  background: white;
  border-radius: 12px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.06);
  border: 1px solid rgba(99, 102, 241, 0.1);
  margin-bottom: 20px;
}

.panel-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 700;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.settings-content {
  flex: 1;
  overflow: auto;
  background: white;
  border-radius: 12px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.06);
  border: 1px solid rgba(99, 102, 241, 0.1);
}

.setting-hint {
  margin-left: 12px;
  color: #64748b;
  font-size: 13px;
  font-style: italic;
}

.about-content {
  max-width: 800px;
  margin: 0 auto;
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
  height: 100%;
}

:deep(.el-tabs__header) {
  margin: 0;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 12px 12px 0 0;
  padding: 0 20px;
}

:deep(.el-tabs__nav-wrap) {
  padding: 0;
}

:deep(.el-tabs__item) {
  color: rgba(255, 255, 255, 0.8);
  font-weight: 500;
  padding: 16px 20px;
  border: none;
  transition: all 0.3s ease;
}

:deep(.el-tabs__item:hover) {
  color: white;
}

:deep(.el-tabs__item.is-active) {
  color: white;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 8px 8px 0 0;
}

:deep(.el-tabs__active-bar) {
  background: white;
  height: 3px;
  border-radius: 2px;
}

:deep(.el-tabs__content) {
  padding: 24px;
  height: calc(100% - 60px);
  overflow-y: auto;
}

:deep(.el-tab-pane) {
  height: 100%;
}

:deep(.el-form-item) {
  margin-bottom: 24px;
}

:deep(.el-form-item__label) {
  color: #374151;
  font-weight: 600;
  font-size: 14px;
}

:deep(.el-input__wrapper) {
  border-radius: 8px;
  transition: all 0.3s ease;
  border: 1px solid #e2e8f0;
}

:deep(.el-input__wrapper:hover) {
  border-color: #667eea;
  box-shadow: 0 2px 8px rgba(102, 126, 234, 0.15);
}

:deep(.el-input__wrapper.is-focus) {
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
}

:deep(.el-select .el-input__wrapper) {
  cursor: pointer;
}

:deep(.el-switch) {
  --el-switch-on-color: #667eea;
}

:deep(.el-input-number) {
  width: 120px;
}

:deep(.el-input-number .el-input__wrapper) {
  border-radius: 8px;
}

:deep(.el-button) {
  border-radius: 8px;
  font-weight: 500;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

:deep(.el-button--primary) {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
}

:deep(.el-button--primary:hover) {
  background: linear-gradient(135deg, #5a67d8 0%, #6b46c1 100%);
  transform: translateY(-1px);
  box-shadow: 0 4px 15px rgba(102, 126, 234, 0.4);
}

:deep(.el-divider) {
  margin: 32px 0;
  border-color: #e2e8f0;
}

/* 滚动条样式 */
.settings-content::-webkit-scrollbar {
  width: 8px;
}

.settings-content::-webkit-scrollbar-track {
  background: #f1f5f9;
  border-radius: 6px;
  margin: 4px;
}

.settings-content::-webkit-scrollbar-thumb {
  background: linear-gradient(135deg, #cbd5e1 0%, #94a3b8 100%);
  border-radius: 6px;
  border: 2px solid #f1f5f9;
}

.settings-content::-webkit-scrollbar-thumb:hover {
  background: linear-gradient(135deg, #94a3b8 0%, #64748b 100%);
}

/* 主题预览样式 */
.theme-preview {
  display: flex;
  gap: 16px;
  margin-top: 12px;
}

.theme-card {
  width: 120px;
  height: 80px;
  border: 2px solid #e2e8f0;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.3s ease;
  overflow: hidden;
  position: relative;
}

.theme-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1);
}

.theme-card.active {
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.2);
}

.theme-card.light {
  background: #ffffff;
}

.theme-card.light .theme-header {
  height: 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.theme-card.light .theme-content {
  display: flex;
  height: 60px;
  padding: 4px;
  gap: 4px;
}

.theme-card.light .theme-sidebar {
  width: 30%;
  background: #f8fafc;
  border-radius: 4px;
}

.theme-card.light .theme-main {
  flex: 1;
  background: #f1f5f9;
  border-radius: 4px;
}

.theme-card.dark {
  background: #334155;
}

.theme-card.dark .theme-header {
  height: 20px;
  background: linear-gradient(135deg, #818cf8 0%, #a78bfa 100%);
}

.theme-card.dark .theme-content {
  display: flex;
  height: 60px;
  padding: 4px;
  gap: 4px;
}

.theme-card.dark .theme-sidebar {
  width: 30%;
  background: #475569;
  border-radius: 4px;
}

.theme-card.dark .theme-main {
  flex: 1;
  background: #1e293b;
  border-radius: 4px;
}
</style>