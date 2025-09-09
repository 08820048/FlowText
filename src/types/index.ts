// 视频文件信息接口
export interface VideoInfo {
  filePath: string;
  fileName: string;
  duration: number; // 视频时长（秒）
  resolution: {
    width: number;
    height: number;
  };
  frameRate: number; // 帧率
  codecInfo: string; // 编码格式
  audioTracks: AudioTrack[];
}

// 音频轨道信息
export interface AudioTrack {
  id: number;
  language?: string;
  codecInfo: string;
  channels: number;
  sampleRate: number;
}

// 字幕信息接口
export interface Subtitle {
  id: string;
  startTime: number; // 开始时间（秒）
  endTime: number; // 结束时间（秒）
  text: string; // 字幕文本
  style?: SubtitleStyle; // 字幕样式
}

// 字幕样式接口
export interface SubtitleStyle {
  fontFamily?: string;
  fontSize?: number;
  color?: string;
  backgroundColor?: string;
  position?: 'top' | 'middle' | 'bottom';
  alignment?: 'left' | 'center' | 'right';
  outline?: boolean;
  outlineColor?: string;
  outlineWidth?: number;
}

// 识别引擎类型
export type RecognitionEngine = 'whisper' | 'faster-whisper' | 'sensevoice' | 'baidu' | 'tencent' | 'aliyun';

// 模型提供商类型
export type ModelProvider = 'openai' | 'alibaba' | 'baidu' | 'tencent' | 'aliyun';

// 模型配置接口
export interface ModelConfig {
  provider: ModelProvider;
  name: string;
  displayName: string;
  description: string;
  sizes: ModelSize[];
  languages: string[];
  features: ModelFeature[];
  requirements: ModelRequirements;
  performance: ModelPerformance;
}

// 模型大小配置
export interface ModelSize {
  id: string;
  name: string;
  displayName: string;
  description: string;
  fileSize: string;
  memoryUsage: string;
  speed: 'very-fast' | 'fast' | 'medium' | 'slow' | 'very-slow';
  accuracy: 'basic' | 'good' | 'high' | 'very-high' | 'excellent';
  downloadUrl?: string;
  localPath?: string;
}

// 模型特性
export interface ModelFeature {
  id: string;
  name: string;
  description: string;
  supported: boolean;
}

// 模型系统要求
export interface ModelRequirements {
  python?: string;
  packages: string[];
  minMemory: string;
  minDisk: string;
  gpu?: boolean;
  cuda?: string;
}

// 模型性能指标
export interface ModelPerformance {
  wer?: number; // Word Error Rate
  rtf?: number; // Real Time Factor
  latency?: number; // ms
  throughput?: number; // words/second
}

// 识别任务状态
export type RecognitionStatus = 'pending' | 'processing' | 'completed' | 'failed';

// 识别任务接口
export interface RecognitionTask {
  id: string;
  videoInfo: VideoInfo;
  status: RecognitionStatus;
  progress: number; // 0-100
  engine: RecognitionEngine;
  language: string;
  subtitles?: Subtitle[];
  error?: string;
  createdAt: Date;
  updatedAt: Date;
}

// 字幕格式类型
export type SubtitleFormat = 'srt' | 'ass' | 'vtt' | 'txt' | 'json';

// 应用设置接口
export interface AppSettings {
  defaultEngine: RecognitionEngine;
  defaultLanguage: string;
  defaultSubtitleFormat: SubtitleFormat;
  apiKeys: {
    baidu?: {
      appId: string;
      apiKey: string;
      secretKey: string;
    };
    tencent?: {
      secretId: string;
      secretKey: string;
      // COS配置（可选，用于大文件上传）
      cos?: {
        bucket: string;
        region: string;
        domain?: string; // 自定义域名（可选）
      };
    };
    aliyun?: {
      accessKeyId: string;
      accessKeySecret: string;
    };
  };
  // 模型配置
  modelConfigs: {
    whisper?: {
      size: 'tiny' | 'base' | 'small' | 'medium' | 'large' | 'large-v2' | 'large-v3';
      device: 'cpu' | 'gpu';
      computeType?: 'int8' | 'int16' | 'float16' | 'float32';
    };
    fasterWhisper?: {
      size: 'tiny' | 'base' | 'small' | 'medium' | 'large' | 'large-v2' | 'large-v3';
      device: 'cpu' | 'gpu';
      computeType: 'int8' | 'int16' | 'float16' | 'float32';
      beamSize?: number;
      temperature?: number;
    };
    sensevoice?: {
      size: 'small' | 'large';
      device: 'cpu' | 'gpu';
      language?: 'auto' | 'zh' | 'en' | 'ja' | 'ko';
      enableEmotionRecognition?: boolean;
      enableEventDetection?: boolean;
    };
  };
  useGPU: boolean;
  maxConcurrentTasks: number;
  autoSave: boolean;
  autoSaveInterval: number; // 秒
  exportPath?: string; // 字幕导出路径
}