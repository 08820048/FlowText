import type { ModelConfig, ModelSize, ModelFeature, ModelRequirements, ModelPerformance } from '../types';

/**
 * 模型管理器
 * 负责管理所有语音识别模型的配置、安装状态和性能信息
 */
export class ModelManager {
  private static instance: ModelManager;
  private models: Map<string, ModelConfig> = new Map();

  private constructor() {
    this.initializeModels();
  }

  public static getInstance(): ModelManager {
    if (!ModelManager.instance) {
      ModelManager.instance = new ModelManager();
    }
    return ModelManager.instance;
  }

  /**
   * 初始化所有支持的模型配置
   */
  private initializeModels() {
    // Whisper 模型配置
    this.models.set('whisper', {
      provider: 'openai',
      name: 'whisper',
      displayName: 'OpenAI Whisper',
      description: 'OpenAI开源的多语言语音识别模型，支持99种语言',
      sizes: this.getWhisperSizes(),
      languages: this.getWhisperLanguages(),
      features: this.getWhisperFeatures(),
      requirements: {
        python: '>=3.8',
        packages: ['openai-whisper', 'torch', 'torchaudio'],
        minMemory: '4GB',
        minDisk: '10GB',
        gpu: false,
        cuda: '>=11.0'
      },
      performance: {
        wer: 0.15,
        rtf: 0.3,
        latency: 2000,
        throughput: 150
      }
    });

    // Faster-Whisper 模型配置
    this.models.set('faster-whisper', {
      provider: 'openai',
      name: 'faster-whisper',
      displayName: 'Faster Whisper',
      description: '基于CTranslate2优化的Whisper实现，速度提升4-5倍',
      sizes: this.getFasterWhisperSizes(),
      languages: this.getWhisperLanguages(),
      features: this.getFasterWhisperFeatures(),
      requirements: {
        python: '>=3.8',
        packages: ['faster-whisper', 'ctranslate2'],
        minMemory: '2GB',
        minDisk: '8GB',
        gpu: false,
        cuda: '>=11.2'
      },
      performance: {
        wer: 0.15,
        rtf: 0.06,
        latency: 500,
        throughput: 600
      }
    });

    // SenseVoice 模型配置
    this.models.set('sensevoice', {
      provider: 'alibaba',
      name: 'sensevoice',
      displayName: 'SenseVoice',
      description: '阿里巴巴开源的多语言语音理解模型，支持情感识别和事件检测',
      sizes: this.getSenseVoiceSizes(),
      languages: this.getSenseVoiceLanguages(),
      features: this.getSenseVoiceFeatures(),
      requirements: {
        python: '>=3.8',
        packages: ['funasr', 'modelscope', 'torch', 'torchaudio'],
        minMemory: '4GB',
        minDisk: '6GB',
        gpu: false,
        cuda: '>=11.0'
      },
      performance: {
        wer: 0.12,
        rtf: 0.1,
        latency: 800,
        throughput: 400
      }
    });
  }

  /**
   * 获取 Whisper 模型大小配置
   */
  private getWhisperSizes(): ModelSize[] {
    return [
      {
        id: 'tiny',
        name: 'tiny',
        displayName: 'Tiny',
        description: '最小模型，速度最快但精度较低',
        fileSize: '39MB',
        memoryUsage: '~1GB',
        speed: 'very-fast',
        accuracy: 'basic'
      },
      {
        id: 'base',
        name: 'base',
        displayName: 'Base',
        description: '基础模型，平衡速度和精度',
        fileSize: '142MB',
        memoryUsage: '~1GB',
        speed: 'fast',
        accuracy: 'good'
      },
      {
        id: 'small',
        name: 'small',
        displayName: 'Small',
        description: '小型模型，较好的精度',
        fileSize: '461MB',
        memoryUsage: '~2GB',
        speed: 'medium',
        accuracy: 'high'
      },
      {
        id: 'medium',
        name: 'medium',
        displayName: 'Medium',
        description: '中型模型，高精度',
        fileSize: '1.42GB',
        memoryUsage: '~5GB',
        speed: 'slow',
        accuracy: 'very-high'
      },
      {
        id: 'large',
        name: 'large',
        displayName: 'Large',
        description: '大型模型，最高精度',
        fileSize: '2.87GB',
        memoryUsage: '~10GB',
        speed: 'very-slow',
        accuracy: 'excellent'
      },
      {
        id: 'large-v2',
        name: 'large-v2',
        displayName: 'Large v2',
        description: '大型模型v2版本，改进的精度',
        fileSize: '2.87GB',
        memoryUsage: '~10GB',
        speed: 'very-slow',
        accuracy: 'excellent'
      },
      {
        id: 'large-v3',
        name: 'large-v3',
        displayName: 'Large v3',
        description: '最新的大型模型，最佳精度',
        fileSize: '2.87GB',
        memoryUsage: '~10GB',
        speed: 'very-slow',
        accuracy: 'excellent'
      }
    ];
  }

  /**
   * 获取 Faster-Whisper 模型大小配置
   */
  private getFasterWhisperSizes(): ModelSize[] {
    return this.getWhisperSizes().map(size => ({
      ...size,
      description: size.description + ' (优化版本，速度提升4-5倍)',
      speed: this.improveSpeed(size.speed),
      memoryUsage: this.reduceMemoryUsage(size.memoryUsage)
    }));
  }

  /**
   * 获取 SenseVoice 模型大小配置
   */
  private getSenseVoiceSizes(): ModelSize[] {
    return [
      {
        id: 'small',
        name: 'small',
        displayName: 'Small',
        description: '小型模型，支持多语言识别和情感分析',
        fileSize: '800MB',
        memoryUsage: '~3GB',
        speed: 'fast',
        accuracy: 'very-high'
      },
      {
        id: 'large',
        name: 'large',
        displayName: 'Large',
        description: '大型模型，最佳的多语言识别和情感分析效果',
        fileSize: '2.1GB',
        memoryUsage: '~6GB',
        speed: 'medium',
        accuracy: 'excellent'
      }
    ];
  }

  /**
   * 获取支持的语言列表
   */
  private getWhisperLanguages(): string[] {
    return [
      'zh', 'en', 'ja', 'ko', 'fr', 'de', 'es', 'ru', 'it', 'pt',
      'ar', 'hi', 'th', 'vi', 'tr', 'pl', 'nl', 'sv', 'da', 'no'
    ];
  }

  private getSenseVoiceLanguages(): string[] {
    return [
      'zh', 'en', 'ja', 'ko', 'fr', 'de', 'es', 'ru', 'it', 'pt',
      'ar', 'hi', 'th', 'vi', 'tr', 'pl', 'nl', 'sv', 'da', 'no',
      'fi', 'hu', 'cs', 'sk', 'bg', 'hr', 'sl', 'et', 'lv', 'lt'
    ];
  }

  /**
   * 获取模型特性
   */
  private getWhisperFeatures(): ModelFeature[] {
    return [
      { id: 'multilingual', name: '多语言支持', description: '支持99种语言', supported: true },
      { id: 'timestamps', name: '时间戳', description: '提供词级时间戳', supported: true },
      { id: 'translation', name: '翻译', description: '支持翻译为英语', supported: true },
      { id: 'emotion', name: '情感识别', description: '识别语音情感', supported: false },
      { id: 'speaker', name: '说话人识别', description: '区分不同说话人', supported: false },
      { id: 'noise', name: '噪声处理', description: '处理背景噪声', supported: true }
    ];
  }

  private getFasterWhisperFeatures(): ModelFeature[] {
    return [
      ...this.getWhisperFeatures(),
      { id: 'optimization', name: '性能优化', description: 'CTranslate2优化，速度提升4-5倍', supported: true },
      { id: 'quantization', name: '模型量化', description: '支持INT8量化减少内存使用', supported: true }
    ];
  }

  private getSenseVoiceFeatures(): ModelFeature[] {
    return [
      { id: 'multilingual', name: '多语言支持', description: '支持50+种语言', supported: true },
      { id: 'timestamps', name: '时间戳', description: '提供精确时间戳', supported: true },
      { id: 'emotion', name: '情感识别', description: '识别语音情感（开心、悲伤、愤怒等）', supported: true },
      { id: 'event', name: '事件检测', description: '检测音频事件（掌声、笑声等）', supported: true },
      { id: 'language_id', name: '语言识别', description: '自动识别语言类型', supported: true },
      { id: 'noise', name: '噪声处理', description: '优秀的噪声处理能力', supported: true }
    ];
  }

  /**
   * 辅助方法：提升速度等级
   */
  private improveSpeed(speed: string): 'very-fast' | 'fast' | 'medium' | 'slow' | 'very-slow' {
    const speedMap: Record<string, 'very-fast' | 'fast' | 'medium' | 'slow' | 'very-slow'> = {
      'very-slow': 'slow',
      'slow': 'medium',
      'medium': 'fast',
      'fast': 'very-fast',
      'very-fast': 'very-fast'
    };
    return speedMap[speed] || 'fast';
  }

  /**
   * 辅助方法：减少内存使用
   */
  private reduceMemoryUsage(memory: string): string {
    const match = memory.match(/~(\d+)GB/);
    if (match) {
      const gb = parseInt(match[1]);
      const reducedGb = Math.max(1, Math.floor(gb * 0.7));
      return `~${reducedGb}GB`;
    }
    return memory;
  }

  /**
   * 获取所有模型配置
   */
  public getAllModels(): ModelConfig[] {
    return Array.from(this.models.values());
  }

  /**
   * 根据名称获取模型配置
   */
  public getModel(name: string): ModelConfig | undefined {
    return this.models.get(name);
  }

  /**
   * 获取模型的可用大小
   */
  public getModelSizes(modelName: string): ModelSize[] {
    const model = this.models.get(modelName);
    return model?.sizes || [];
  }

  /**
   * 检查模型是否支持某个特性
   */
  public hasFeature(modelName: string, featureId: string): boolean {
    const model = this.models.get(modelName);
    if (!model) return false;
    
    const feature = model.features.find(f => f.id === featureId);
    return feature?.supported || false;
  }

  /**
   * 获取推荐的模型配置
   */
  public getRecommendedModel(requirements: {
    speed?: 'fast' | 'balanced' | 'accurate';
    memory?: 'low' | 'medium' | 'high';
    features?: string[];
  }): { model: string; size: string } {
    // 根据需求推荐最适合的模型和大小
    if (requirements.features?.includes('emotion')) {
      return { model: 'sensevoice', size: 'small' };
    }
    
    if (requirements.speed === 'fast') {
      return { model: 'faster-whisper', size: 'base' };
    }
    
    if (requirements.memory === 'low') {
      return { model: 'faster-whisper', size: 'tiny' };
    }
    
    if (requirements.speed === 'accurate') {
      return { model: 'sensevoice', size: 'large' };
    }
    
    // 默认推荐
    return { model: 'faster-whisper', size: 'base' };
  }
}

// 导出单例实例
export const modelManager = ModelManager.getInstance();
