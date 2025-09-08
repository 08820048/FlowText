# FlowText API 参考文档

## 📋 概述

本文档详细描述了 FlowText 应用的 API 接口规范，包括前端与后端（Tauri）之间的通信接口，以及与第三方语音识别服务的集成接口。

## 🏗️ 架构概览

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│                 │    │                 │    │                 │
│   Vue Frontend  │◄──►│  Tauri Backend  │◄──►│  External APIs  │
│                 │    │                 │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
        │                        │                        │
        │                        │                        │
    Web APIs                Rust Commands           Speech APIs
   (JavaScript)              (Rust)                (HTTP/gRPC)
```

## 🔧 Tauri Commands API

### 文件操作

#### `select_video_file`

选择视频文件

**调用方式**:
```javascript
import { invoke } from '@tauri-apps/api/tauri'

const result = await invoke('select_video_file')
```

**返回值**:
```typescript
interface VideoFileInfo {
  path: string;           // 文件路径
  name: string;           // 文件名
  size: number;           // 文件大小（字节）
  duration: number;       // 视频时长（秒）
  width: number;          // 视频宽度
  height: number;         // 视频高度
  fps: number;            // 帧率
  format: string;         // 视频格式
  codec: string;          // 视频编码
  bitrate: number;        // 比特率
  audio_info: {
    codec: string;        // 音频编码
    sample_rate: number;  // 采样率
    channels: number;     // 声道数
    bitrate: number;      // 音频比特率
  };
}
```

**错误处理**:
```typescript
try {
  const fileInfo = await invoke('select_video_file');
  console.log('文件信息:', fileInfo);
} catch (error) {
  console.error('文件选择失败:', error);
}
```

#### `extract_audio`

从视频中提取音频

**参数**:
```typescript
interface ExtractAudioParams {
  video_path: string;     // 视频文件路径
  output_path: string;    // 输出音频路径
  quality: 'low' | 'medium' | 'high';  // 音频质量
  format: 'wav' | 'mp3' | 'flac';      // 音频格式
}
```

**调用方式**:
```javascript
const result = await invoke('extract_audio', {
  video_path: '/path/to/video.mp4',
  output_path: '/path/to/audio.wav',
  quality: 'medium',
  format: 'wav'
});
```

**返回值**:
```typescript
interface ExtractAudioResult {
  success: boolean;
  audio_path: string;     // 提取的音频文件路径
  duration: number;       // 音频时长
  size: number;           // 文件大小
  error?: string;         // 错误信息（如果失败）
}
```

### 语音识别

#### `start_recognition`

开始语音识别任务

**参数**:
```typescript
interface RecognitionParams {
  audio_path: string;           // 音频文件路径
  engine: 'whisper' | 'baidu' | 'tencent' | 'aliyun';  // 识别引擎
  language: string;             // 语言代码 (zh-CN, en-US, etc.)
  config: {
    // Whisper 配置
    whisper?: {
      model: 'tiny' | 'base' | 'small' | 'medium' | 'large';
      temperature: number;      // 0.0 - 1.0
      beam_size: number;        // 1 - 10
    };
    // 百度云配置
    baidu?: {
      app_id: string;
      api_key: string;
      secret_key: string;
      dev_pid: number;          // 语言模型ID
    };
    // 腾讯云配置
    tencent?: {
      secret_id: string;
      secret_key: string;
      region: string;
      engine_model_type: string;
    };
    // 阿里云配置
    aliyun?: {
      access_key_id: string;
      access_key_secret: string;
      app_key: string;
      region: string;
    };
  };
}
```

**调用方式**:
```javascript
const taskId = await invoke('start_recognition', {
  audio_path: '/path/to/audio.wav',
  engine: 'whisper',
  language: 'zh-CN',
  config: {
    whisper: {
      model: 'base',
      temperature: 0.0,
      beam_size: 5
    }
  }
});
```

**返回值**:
```typescript
interface RecognitionTask {
  task_id: string;        // 任务ID
  status: 'pending' | 'running' | 'completed' | 'failed';
  created_at: string;     // 创建时间 (ISO 8601)
}
```

#### `get_recognition_status`

获取识别任务状态

**参数**:
```typescript
interface GetStatusParams {
  task_id: string;        // 任务ID
}
```

**返回值**:
```typescript
interface RecognitionStatus {
  task_id: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  progress: number;       // 进度百分比 (0-100)
  current_step: string;   // 当前步骤描述
  estimated_time: number; // 预估剩余时间（秒）
  error?: string;         // 错误信息
  result?: SubtitleItem[]; // 识别结果（完成时）
}

interface SubtitleItem {
  index: number;          // 字幕序号
  start_time: number;     // 开始时间（秒）
  end_time: number;       // 结束时间（秒）
  text: string;           // 字幕文本
  confidence: number;     // 置信度 (0.0-1.0)
  speaker?: string;       // 说话人（如果支持）
}
```

#### `cancel_recognition`

取消识别任务

**参数**:
```typescript
interface CancelTaskParams {
  task_id: string;        // 任务ID
}
```

**返回值**:
```typescript
interface CancelResult {
  success: boolean;
  message: string;
}
```

### 字幕操作

#### `save_subtitles`

保存字幕文件

**参数**:
```typescript
interface SaveSubtitlesParams {
  subtitles: SubtitleItem[];  // 字幕数据
  output_path: string;        // 输出文件路径
  format: 'srt' | 'ass' | 'vtt' | 'txt' | 'json';  // 输出格式
  encoding: 'utf-8' | 'gbk' | 'ascii';  // 文件编码
  options?: {
    // SRT 选项
    srt?: {
      time_format: 'standard' | 'frame';  // 时间格式
    };
    // ASS 选项
    ass?: {
      style: {
        font_name: string;
        font_size: number;
        primary_color: string;
        secondary_color: string;
        outline_color: string;
        shadow_color: string;
        bold: boolean;
        italic: boolean;
        underline: boolean;
        strike_out: boolean;
        scale_x: number;
        scale_y: number;
        spacing: number;
        angle: number;
        border_style: number;
        outline: number;
        shadow: number;
        alignment: number;
        margin_l: number;
        margin_r: number;
        margin_v: number;
      };
    };
    // VTT 选项
    vtt?: {
      include_styles: boolean;
    };
  };
}
```

**调用方式**:
```javascript
const result = await invoke('save_subtitles', {
  subtitles: subtitleData,
  output_path: '/path/to/output.srt',
  format: 'srt',
  encoding: 'utf-8'
});
```

**返回值**:
```typescript
interface SaveResult {
  success: boolean;
  file_path: string;      // 保存的文件路径
  file_size: number;      // 文件大小
  subtitle_count: number; // 字幕条数
  error?: string;         // 错误信息
}
```

#### `load_subtitles`

加载字幕文件

**参数**:
```typescript
interface LoadSubtitlesParams {
  file_path: string;      // 字幕文件路径
  encoding?: string;      // 文件编码（自动检测）
}
```

**返回值**:
```typescript
interface LoadResult {
  success: boolean;
  subtitles: SubtitleItem[];
  format: string;         // 检测到的格式
  encoding: string;       // 检测到的编码
  error?: string;
}
```

### 应用设置

#### `get_app_config`

获取应用配置

**返回值**:
```typescript
interface AppConfig {
  version: string;        // 应用版本
  theme: 'light' | 'dark' | 'auto';  // 主题设置
  language: string;       // 界面语言
  default_engine: string; // 默认识别引擎
  default_language: string; // 默认识别语言
  audio_quality: string;  // 默认音频质量
  max_concurrent_tasks: number; // 最大并发任务数
  gpu_acceleration: boolean;    // GPU加速
  auto_save: boolean;     // 自动保存
  backup_enabled: boolean; // 备份功能
  api_configs: {
    baidu?: {
      app_id: string;
      api_key: string;
      secret_key: string;
    };
    tencent?: {
      secret_id: string;
      secret_key: string;
      region: string;
    };
    aliyun?: {
      access_key_id: string;
      access_key_secret: string;
      app_key: string;
      region: string;
    };
  };
  paths: {
    default_import: string;   // 默认导入路径
    default_export: string;   // 默认导出路径
    temp_directory: string;   // 临时文件目录
    cache_directory: string;  // 缓存目录
  };
  performance: {
    memory_limit: number;     // 内存限制（MB）
    disk_cache_size: number;  // 磁盘缓存大小（MB）
    network_timeout: number;  // 网络超时（秒）
    retry_count: number;      // 重试次数
  };
}
```

#### `update_app_config`

更新应用配置

**参数**:
```typescript
interface UpdateConfigParams {
  config: Partial<AppConfig>;  // 部分配置更新
}
```

**返回值**:
```typescript
interface UpdateResult {
  success: boolean;
  message: string;
  error?: string;
}
```

### 系统信息

#### `get_system_info`

获取系统信息

**返回值**:
```typescript
interface SystemInfo {
  os: {
    name: string;           // 操作系统名称
    version: string;        // 系统版本
    arch: string;           // 系统架构
  };
  hardware: {
    cpu: {
      name: string;         // CPU名称
      cores: number;        // 核心数
      threads: number;      // 线程数
      frequency: number;    // 频率（MHz）
    };
    memory: {
      total: number;        // 总内存（MB）
      available: number;    // 可用内存（MB）
      used: number;         // 已用内存（MB）
    };
    gpu?: {
      name: string;         // GPU名称
      memory: number;       // 显存（MB）
      driver_version: string; // 驱动版本
      cuda_support: boolean;  // CUDA支持
      opencl_support: boolean; // OpenCL支持
    }[];
  };
  storage: {
    drives: {
      name: string;         // 驱动器名称
      total: number;        // 总容量（MB）
      free: number;         // 可用容量（MB）
      type: 'hdd' | 'ssd' | 'unknown'; // 存储类型
    }[];
  };
  network: {
    connected: boolean;     // 网络连接状态
    interfaces: {
      name: string;         // 网络接口名称
      ip: string;           // IP地址
      mac: string;          // MAC地址
      type: 'ethernet' | 'wifi' | 'other'; // 接口类型
    }[];
  };
  dependencies: {
    ffmpeg: {
      installed: boolean;
      version?: string;
      path?: string;
    };
    python: {
      installed: boolean;
      version?: string;
      path?: string;
    };
    whisper: {
      installed: boolean;
      models: string[];     // 已安装的模型
    };
  };
}
```

### 任务管理

#### `get_task_list`

获取任务列表

**参数**:
```typescript
interface GetTaskListParams {
  status?: 'all' | 'pending' | 'running' | 'completed' | 'failed';
  limit?: number;         // 返回数量限制
  offset?: number;        // 偏移量
}
```

**返回值**:
```typescript
interface TaskList {
  tasks: TaskInfo[];
  total: number;          // 总任务数
  has_more: boolean;      // 是否有更多
}

interface TaskInfo {
  task_id: string;
  type: 'recognition' | 'export' | 'import';
  status: 'pending' | 'running' | 'completed' | 'failed';
  progress: number;
  created_at: string;
  updated_at: string;
  completed_at?: string;
  input_file: string;
  output_file?: string;
  engine?: string;
  language?: string;
  error?: string;
  metadata: {
    duration?: number;
    file_size?: number;
    subtitle_count?: number;
  };
}
```

#### `clear_completed_tasks`

清理已完成的任务

**返回值**:
```typescript
interface ClearResult {
  success: boolean;
  cleared_count: number;
  message: string;
}
```

## 🌐 外部 API 集成

### 百度智能云语音识别

#### 认证接口

**获取 Access Token**:
```http
POST https://aip.baidubce.com/oauth/2.0/token
Content-Type: application/x-www-form-urlencoded

grant_type=client_credentials&client_id={API_KEY}&client_secret={SECRET_KEY}
```

**响应**:
```json
{
  "access_token": "24.xxx.xxx",
  "expires_in": 2592000,
  "refresh_token": "25.xxx.xxx",
  "scope": "audio_voice_assistant_get",
  "session_key": "9mzdxxx",
  "session_secret": "xxx"
}
```

#### 语音识别接口

**短语音识别**:
```http
POST https://vop.baidu.com/server_api
Content-Type: application/json

{
  "format": "wav",
  "rate": 16000,
  "channel": 1,
  "cuid": "FlowText",
  "token": "{ACCESS_TOKEN}",
  "speech": "{BASE64_AUDIO_DATA}",
  "len": 4096,
  "dev_pid": 1537
}
```

**长语音识别**:
```http
POST https://vop.baidu.com/pro_api
Content-Type: application/json

{
  "format": "wav",
  "rate": 16000,
  "channel": 1,
  "cuid": "FlowText",
  "token": "{ACCESS_TOKEN}",
  "speech": "{BASE64_AUDIO_DATA}",
  "len": 4096,
  "dev_pid": 80001
}
```

**响应格式**:
```json
{
  "err_no": 0,
  "err_msg": "success.",
  "corpus_no": "15984125203285346378",
  "sn": "481D633F-73BA-726F-49EF-8659ACCC2F3D",
  "result": ["北京天气"]
}
```

### 腾讯云语音识别

#### 实时语音识别

**请求示例**:
```http
POST https://asr.tencentcloudapi.com/
Content-Type: application/json
X-TC-Action: SentenceRecognition
X-TC-Version: 2019-06-14
X-TC-Region: ap-beijing
X-TC-Timestamp: 1551113972
Authorization: TC3-HMAC-SHA256 Credential=...

{
  "ProjectId": 0,
  "SubServiceType": 2,
  "EngSerViceType": "16k_zh",
  "SourceType": 1,
  "VoiceFormat": "wav",
  "UsrAudioKey": "session-123",
  "Data": "{BASE64_AUDIO_DATA}",
  "DataLen": 4096
}
```

**响应示例**:
```json
{
  "Response": {
    "Result": "北京天气怎么样",
    "AudioDuration": 2000,
    "WordList": [
      {
        "Word": "北京",
        "StartTime": 0,
        "EndTime": 500,
        "Stable": true
      }
    ],
    "RequestId": "6e8091e6-c2c4-4e1d-8223-2b5e8b5e8b5e"
  }
}
```

### 阿里云语音识别

#### 一句话识别

**请求示例**:
```http
POST https://nls-meta.cn-shanghai.aliyuncs.com/stream/v1/asr
Content-Type: application/json
Authorization: Bearer {ACCESS_TOKEN}

{
  "appkey": "{APP_KEY}",
  "file_link": "{AUDIO_FILE_URL}",
  "version": "4.0",
  "enable_words": true
}
```

**响应示例**:
```json
{
  "status": 20000000,
  "message": "SUCCESS",
  "request_id": "5f9a8b2c-1234-5678-9abc-def012345678",
  "result": "北京天气怎么样",
  "task_id": "task123456"
}
```

## 📡 WebSocket 实时通信

### 任务进度推送

**连接地址**: `ws://localhost:1420/ws/progress`

**消息格式**:
```typescript
interface ProgressMessage {
  type: 'progress' | 'status' | 'error' | 'complete';
  task_id: string;
  data: {
    progress?: number;      // 进度百分比
    status?: string;        // 状态描述
    current_step?: string;  // 当前步骤
    estimated_time?: number; // 预估时间
    error?: string;         // 错误信息
    result?: any;           // 结果数据
  };
  timestamp: string;        // 时间戳
}
```

**客户端示例**:
```javascript
const ws = new WebSocket('ws://localhost:1420/ws/progress');

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  
  switch (message.type) {
    case 'progress':
      updateProgressBar(message.task_id, message.data.progress);
      break;
    case 'status':
      updateStatusText(message.task_id, message.data.status);
      break;
    case 'error':
      showError(message.task_id, message.data.error);
      break;
    case 'complete':
      handleTaskComplete(message.task_id, message.data.result);
      break;
  }
};

ws.onerror = (error) => {
  console.error('WebSocket 连接错误:', error);
};

ws.onclose = () => {
  console.log('WebSocket 连接已关闭');
  // 实现重连逻辑
};
```

## 🔐 安全性

### API 密钥管理

1. **本地加密存储**
   - 使用 AES-256 加密算法
   - 密钥派生使用 PBKDF2
   - 盐值随机生成

2. **传输安全**
   - 所有 API 调用使用 HTTPS
   - 支持证书验证
   - 请求签名验证

3. **访问控制**
   - API 密钥权限最小化
   - 支持密钥轮换
   - 异常访问检测

### 数据隐私

1. **本地处理优先**
   - Whisper 引擎完全本地运行
   - 敏感数据不上传云端
   - 临时文件自动清理

2. **数据脱敏**
   - 日志中不记录敏感信息
   - API 调用参数过滤
   - 错误信息脱敏处理

## 🚀 性能优化

### 并发处理

1. **任务队列**
   - 支持多任务并发
   - 智能任务调度
   - 资源使用监控

2. **内存管理**
   - 流式音频处理
   - 内存使用限制
   - 垃圾回收优化

3. **缓存策略**
   - 识别结果缓存
   - 模型文件缓存
   - API 响应缓存

### 网络优化

1. **请求优化**
   - 连接池复用
   - 请求压缩
   - 超时重试机制

2. **带宽管理**
   - 分片上传大文件
   - 断点续传支持
   - 流量控制

## 🐛 错误处理

### 错误代码规范

```typescript
enum ErrorCode {
  // 系统错误 (1000-1999)
  SYSTEM_ERROR = 1000,
  PERMISSION_DENIED = 1001,
  DISK_SPACE_INSUFFICIENT = 1002,
  MEMORY_INSUFFICIENT = 1003,
  
  // 文件错误 (2000-2999)
  FILE_NOT_FOUND = 2000,
  FILE_FORMAT_UNSUPPORTED = 2001,
  FILE_CORRUPTED = 2002,
  FILE_TOO_LARGE = 2003,
  
  // 网络错误 (3000-3999)
  NETWORK_ERROR = 3000,
  API_AUTHENTICATION_FAILED = 3001,
  API_QUOTA_EXCEEDED = 3002,
  API_REQUEST_TIMEOUT = 3003,
  
  // 识别错误 (4000-4999)
  RECOGNITION_FAILED = 4000,
  AUDIO_EXTRACTION_FAILED = 4001,
  MODEL_LOADING_FAILED = 4002,
  LANGUAGE_NOT_SUPPORTED = 4003,
  
  // 字幕错误 (5000-5999)
  SUBTITLE_PARSING_FAILED = 5000,
  SUBTITLE_EXPORT_FAILED = 5001,
  SUBTITLE_FORMAT_INVALID = 5002,
  
  // 配置错误 (6000-6999)
  CONFIG_INVALID = 6000,
  API_CONFIG_MISSING = 6001,
  PATH_INVALID = 6002
}
```

### 错误响应格式

```typescript
interface ErrorResponse {
  success: false;
  error: {
    code: ErrorCode;
    message: string;
    details?: any;
    timestamp: string;
    request_id?: string;
  };
}
```

### 错误处理最佳实践

1. **客户端处理**
```javascript
try {
  const result = await invoke('start_recognition', params);
  // 处理成功结果
} catch (error) {
  switch (error.code) {
    case ErrorCode.API_AUTHENTICATION_FAILED:
      showApiConfigDialog();
      break;
    case ErrorCode.DISK_SPACE_INSUFFICIENT:
      showDiskSpaceWarning();
      break;
    default:
      showGenericError(error.message);
  }
}
```

2. **重试机制**
```javascript
async function retryableInvoke(command, params, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await invoke(command, params);
    } catch (error) {
      if (i === maxRetries - 1 || !isRetryableError(error)) {
        throw error;
      }
      await delay(Math.pow(2, i) * 1000); // 指数退避
    }
  }
}
```

## 📊 监控和日志

### 性能监控

```typescript
interface PerformanceMetrics {
  cpu_usage: number;          // CPU使用率
  memory_usage: number;       // 内存使用量
  disk_io: {
    read_bytes: number;
    write_bytes: number;
  };
  network_io: {
    bytes_sent: number;
    bytes_received: number;
  };
  task_metrics: {
    active_tasks: number;
    completed_tasks: number;
    failed_tasks: number;
    average_duration: number;
  };
}
```

### 日志格式

```json
{
  "timestamp": "2024-01-15T10:30:00.000Z",
  "level": "INFO",
  "module": "recognition",
  "message": "Recognition task started",
  "task_id": "task_123456",
  "user_id": "user_789",
  "metadata": {
    "engine": "whisper",
    "language": "zh-CN",
    "file_size": 1024000
  }
}
```

## 🔄 版本控制

### API 版本管理

1. **版本号规范**
   - 主版本号：不兼容的 API 修改
   - 次版本号：向下兼容的功能性新增
   - 修订号：向下兼容的问题修正

2. **兼容性策略**
   - 保持向下兼容至少 2 个主版本
   - 废弃功能提前通知
   - 提供迁移指南

### 更新机制

```typescript
interface UpdateInfo {
  current_version: string;
  latest_version: string;
  update_available: boolean;
  update_required: boolean;
  release_notes: string;
  download_url: string;
  checksum: string;
}
```

---

## 📞 技术支持

如果您在 API 集成过程中遇到问题，请通过以下方式联系我们：

- **开发者文档**: [https://docs.flowtext.app](https://docs.flowtext.app)
- **API 支持**: api-support@flowtext.app
- **GitHub Issues**: [https://github.com/flowtext/flowtext/issues](https://github.com/flowtext/flowtext/issues)
- **开发者社区**: [https://community.flowtext.app](https://community.flowtext.app)

---

*本文档持续更新中，最新版本请访问官方文档网站。*