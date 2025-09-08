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
export type RecognitionEngine = 'baidu' | 'tencent' | 'aliyun' | 'whisper';

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
    };
    aliyun?: {
      accessKeyId: string;
      accessKeySecret: string;
    };
  };
  whisperModel?: 'tiny' | 'base' | 'small';
  useGPU: boolean;
  maxConcurrentTasks: number;
  autoSave: boolean;
  autoSaveInterval: number; // 秒
  theme: 'light' | 'dark' | 'system';
}