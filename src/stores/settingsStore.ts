import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { AppSettings, RecognitionEngine, SubtitleFormat } from '../types';

/**
 * 应用设置状态管理存储
 * 用于管理用户配置和应用设置
 */
export const useSettingsStore = defineStore('settings', () => {
  // 默认设置
  const defaultSettings: AppSettings = {
    defaultEngine: 'faster-whisper',
    defaultLanguage: 'zh',
    defaultSubtitleFormat: 'srt',
    apiKeys: {},
    modelConfigs: {
      whisper: {
        size: 'base',
        device: 'cpu',
        computeType: 'float32'
      },
      fasterWhisper: {
        size: 'base',
        device: 'cpu',
        computeType: 'int8',
        beamSize: 5,
        temperature: 0.0
      },
      sensevoice: {
        size: 'small',
        device: 'cpu',
        language: 'auto',
        enableEmotionRecognition: true,
        enableEventDetection: true
      }
    },
    useGPU: true,
    maxConcurrentTasks: 2,
    autoSave: true,
    autoSaveInterval: 60,
    exportPath: '' // 默认为空，使用系统默认路径
  };
  
  // 当前应用设置
  const settings = ref<AppSettings>(defaultSettings);
  
  /**
   * 初始化设置，从本地存储加载
   */
  async function initSettings() {
    try {
      // 从本地存储加载设置
      const savedSettings = localStorage.getItem('flowtext-settings');
      if (savedSettings) {
        const parsed = JSON.parse(savedSettings);

        // 检查是否有无效的引擎设置
        if (parsed.defaultEngine === 'tencent' || !parsed.defaultEngine ||
            !['whisper', 'faster-whisper', 'sensevoice'].includes(parsed.defaultEngine)) {
          console.log('检测到无效的引擎设置，使用默认设置');
          settings.value = { ...defaultSettings };
          saveSettings();
        } else {
          settings.value = { ...defaultSettings, ...parsed };
        }
      } else {
        settings.value = { ...defaultSettings };
      }
    } catch (error) {
      console.error('Failed to load settings:', error);
      // 如果加载失败，使用默认设置
      settings.value = { ...defaultSettings };
      saveSettings();
    }
  }
  
  /**
   * 保存设置到本地存储
   */
  function saveSettings() {
    try {
      localStorage.setItem('flowtext-settings', JSON.stringify(settings.value));
    } catch (error) {
      console.error('Failed to save settings:', error);
    }
  }
  
  /**
   * 更新设置
   * @param newSettings 新设置（部分）
   */
  function updateSettings(newSettings: Partial<AppSettings>) {
    settings.value = { ...settings.value, ...newSettings };
    saveSettings();
  }
  
  /**
   * 设置API密钥
   * @param engine 识别引擎
   * @param keys API密钥信息
   */
  function setApiKeys(engine: 'baidu' | 'tencent' | 'aliyun', keys: any) {
    settings.value.apiKeys = { ...settings.value.apiKeys, [engine]: keys };
    saveSettings();
  }
  
  /**
   * 设置默认识别引擎
   * @param engine 识别引擎
   */
  function setDefaultEngine(engine: RecognitionEngine) {
    settings.value.defaultEngine = engine;
    saveSettings();
  }
  
  /**
   * 设置默认语言
   * @param language 语言代码
   */
  function setDefaultLanguage(language: string) {
    settings.value.defaultLanguage = language;
    saveSettings();
  }
  
  /**
   * 设置默认字幕格式
   * @param format 字幕格式
   */
  function setDefaultSubtitleFormat(format: SubtitleFormat) {
    settings.value.defaultSubtitleFormat = format;
    saveSettings();
  }

  /**
   * 设置字幕导出路径
   * @param path 导出路径
   */
  function setExportPath(path: string) {
    settings.value.exportPath = path;
    saveSettings();
  }

  // 主题相关方法已移除

  /**
   * 重置设置为默认值
   */
  function resetSettings() {
    settings.value = defaultSettings;
    saveSettings();
  }
  
  return {
    settings,
    initSettings,
    updateSettings,
    setApiKeys,
    setDefaultEngine,
    setDefaultLanguage,
    setDefaultSubtitleFormat,
    setExportPath,
    resetSettings
  };
});