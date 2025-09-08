# FlowText 开发者文档

## 📋 目录

- [项目概述](#项目概述)
- [技术栈详解](#技术栈详解)
- [项目架构](#项目架构)
- [开发环境搭建](#开发环境搭建)
- [核心模块说明](#核心模块说明)
- [API 接口文档](#api-接口文档)
- [数据流设计](#数据流设计)
- [性能优化](#性能优化)
- [测试策略](#测试策略)
- [部署指南](#部署指南)
- [常见问题](#常见问题)

## 项目概述

FlowText 是一个基于 Tauri 2.0 的跨平台桌面应用，主要功能是从视频文件中提取音频并通过语音识别生成字幕。项目采用前后端分离的架构，前端使用 Vue 3 + TypeScript，后端使用 Rust。

### 核心功能模块

1. **视频处理模块** - 视频文件导入、信息解析、音频提取
2. **语音识别模块** - 多引擎语音识别（百度、腾讯、阿里、Whisper）
3. **字幕编辑模块** - 字幕时间轴编辑、内容修改、格式转换
4. **任务管理模块** - 异步任务调度、进度监控、错误处理
5. **设置管理模块** - 用户配置、主题切换、API 密钥管理

## 技术栈详解

### 前端技术栈

#### Vue 3 + Composition API
- **响应式系统**: 使用 `ref`、`reactive`、`computed` 等 API
- **生命周期**: `onMounted`、`onUnmounted` 等钩子函数
- **组件通信**: Props、Emit、Provide/Inject

#### TypeScript
- **类型定义**: 接口定义、泛型使用、类型守卫
- **编译配置**: 严格模式、路径映射、装饰器支持

#### Element Plus
- **组件库**: 表单、表格、对话框、消息提示等
- **主题定制**: CSS 变量、SCSS 变量覆盖
- **国际化**: 多语言支持

#### Pinia 状态管理
- **Store 设计**: 模块化状态管理
- **持久化**: 本地存储集成
- **开发工具**: Vue DevTools 集成

### 后端技术栈

#### Tauri 2.0
- **命令系统**: Rust 函数暴露给前端
- **事件系统**: 前后端双向通信
- **文件系统**: 安全的文件操作 API
- **系统集成**: 原生系统功能调用

#### Rust 核心库
- **异步运行时**: Tokio 异步编程
- **HTTP 客户端**: Reqwest 网络请求
- **JSON 处理**: Serde 序列化/反序列化
- **错误处理**: Result 类型和自定义错误

#### FFmpeg 集成
- **音频提取**: 从视频文件中提取音频轨道
- **格式转换**: 支持多种音频格式输出
- **质量控制**: 可配置的音频质量参数

## 项目架构

### 目录结构

```
FlowText/
├── src/                          # 前端源码
│   ├── components/               # Vue 组件
│   │   ├── VideoPlayer.vue       # 视频播放器组件
│   │   ├── SubtitleEditor.vue    # 字幕编辑器组件
│   │   ├── RecognitionPanel.vue  # 语音识别面板
│   │   ├── SettingsPanel.vue     # 设置面板
│   │   ├── TaskStatusBar.vue     # 任务状态栏
│   │   └── MainLayout.vue        # 主布局组件
│   ├── stores/                   # Pinia 状态管理
│   │   ├── settings.ts           # 设置状态
│   │   ├── video.ts              # 视频状态
│   │   ├── subtitle.ts           # 字幕状态
│   │   └── task.ts               # 任务状态
│   ├── utils/                    # 工具函数
│   │   ├── api.ts                # API 调用封装
│   │   ├── format.ts             # 格式化工具
│   │   ├── validation.ts         # 验证工具
│   │   └── themeManager.ts       # 主题管理器
│   ├── types/                    # TypeScript 类型定义
│   │   ├── video.ts              # 视频相关类型
│   │   ├── subtitle.ts           # 字幕相关类型
│   │   ├── recognition.ts        # 识别相关类型
│   │   └── common.ts             # 通用类型
│   ├── styles/                   # 样式文件
│   │   ├── main.css              # 主样式
│   │   ├── variables.css         # CSS 变量
│   │   └── themes.css            # 主题样式
│   ├── App.vue                   # 根组件
│   └── main.ts                   # 应用入口
├── src-tauri/                    # Tauri 后端源码
│   ├── src/                      # Rust 源码
│   │   ├── commands/             # Tauri 命令
│   │   │   ├── video.rs          # 视频处理命令
│   │   │   ├── recognition.rs    # 语音识别命令
│   │   │   ├── subtitle.rs       # 字幕处理命令
│   │   │   └── settings.rs       # 设置管理命令
│   │   ├── services/             # 业务服务
│   │   │   ├── video_service.rs  # 视频处理服务
│   │   │   ├── audio_service.rs  # 音频处理服务
│   │   │   ├── recognition_service.rs # 语音识别服务
│   │   │   └── subtitle_service.rs    # 字幕处理服务
│   │   ├── models/               # 数据模型
│   │   │   ├── video.rs          # 视频模型
│   │   │   ├── subtitle.rs       # 字幕模型
│   │   │   └── task.rs           # 任务模型
│   │   ├── utils/                # 工具模块
│   │   │   ├── ffmpeg.rs         # FFmpeg 工具
│   │   │   ├── file.rs           # 文件操作工具
│   │   │   └── error.rs          # 错误处理
│   │   ├── config.rs             # 配置管理
│   │   ├── lib.rs                # 库入口
│   │   └── main.rs               # 应用入口
│   ├── Cargo.toml                # Rust 依赖配置
│   ├── tauri.conf.json           # Tauri 配置
│   └── build.rs                  # 构建脚本
├── public/                       # 静态资源
├── dist/                         # 构建输出
├── package.json                  # Node.js 依赖配置
├── vite.config.ts                # Vite 构建配置
├── tsconfig.json                 # TypeScript 配置
└── README.md                     # 项目说明
```

### 架构设计原则

1. **模块化设计**: 功能模块独立，低耦合高内聚
2. **类型安全**: 全面使用 TypeScript 类型系统
3. **异步优先**: 所有 I/O 操作使用异步模式
4. **错误处理**: 完善的错误捕获和用户反馈
5. **性能优化**: 懒加载、虚拟滚动、缓存策略

## 开发环境搭建

### 系统要求

- **Node.js**: >= 18.0.0
- **Rust**: >= 1.70.0
- **操作系统**: Windows 10+, macOS 10.15+, Linux (Ubuntu 18.04+)

### 开发工具推荐

- **IDE**: VS Code
- **插件**:
  - Vue - Official
  - Tauri
  - rust-analyzer
  - TypeScript Importer
  - ESLint
  - Prettier

### 环境配置

1. **安装 Rust**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **安装 Node.js**
   ```bash
   # 使用 nvm 管理 Node.js 版本
   curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
   nvm install 18
   nvm use 18
   ```

3. **安装项目依赖**
   ```bash
   npm install
   ```

4. **安装 Tauri CLI**
   ```bash
   npm install -g @tauri-apps/cli
   ```

### 开发命令

```bash
# 开发模式
npm run tauri dev

# 构建应用
npm run tauri build

# 前端开发服务器
npm run dev

# 类型检查
npm run type-check

# 代码格式化
npm run format

# 代码检查
npm run lint
```

## 核心模块说明

### 1. 视频处理模块

#### 前端组件: VideoPlayer.vue

**功能特性**:
- 视频文件拖拽导入
- 视频信息展示（时长、分辨率、编码格式）
- 视频播放控制（播放/暂停、进度条、音量控制）
- 字幕同步显示
- 全屏播放支持

**关键代码**:
```typescript
// 视频文件导入
const handleFileSelect = async (file: File) => {
  try {
    const videoInfo = await invoke('get_video_info', { filePath: file.path });
    videoStore.setVideoInfo(videoInfo);
  } catch (error) {
    ElMessage.error('视频文件解析失败');
  }
};

// 播放进度同步
const handleTimeUpdate = () => {
  if (videoRef.value) {
    const currentTime = videoRef.value.currentTime;
    videoStore.setCurrentTime(currentTime);
    // 同步字幕显示
    subtitleStore.updateCurrentSubtitle(currentTime);
  }
};
```

#### 后端服务: video_service.rs

**功能实现**:
- FFmpeg 集成，提取视频元信息
- 音频轨道提取和格式转换
- 视频文件格式验证

**关键代码**:
```rust
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoInfo {
    pub duration: f64,
    pub width: u32,
    pub height: u32,
    pub codec: String,
    pub bitrate: u64,
}

/// 获取视频信息
pub async fn get_video_info(file_path: &str) -> Result<VideoInfo, Box<dyn std::error::Error>> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            file_path,
        ])
        .output()?;

    let json_str = String::from_utf8(output.stdout)?;
    let probe_data: serde_json::Value = serde_json::from_str(&json_str)?;
    
    // 解析视频流信息
    let video_stream = probe_data["streams"]
        .as_array()
        .unwrap()
        .iter()
        .find(|stream| stream["codec_type"] == "video")
        .ok_or("No video stream found")?;

    Ok(VideoInfo {
        duration: probe_data["format"]["duration"].as_str().unwrap().parse()?,
        width: video_stream["width"].as_u64().unwrap() as u32,
        height: video_stream["height"].as_u64().unwrap() as u32,
        codec: video_stream["codec_name"].as_str().unwrap().to_string(),
        bitrate: probe_data["format"]["bit_rate"].as_str().unwrap().parse()?,
    })
}

/// 提取音频
pub async fn extract_audio(
    video_path: &str,
    output_path: &str,
    quality: AudioQuality,
) -> Result<(), Box<dyn std::error::Error>> {
    let (bitrate, sample_rate) = match quality {
        AudioQuality::Low => ("64k", "16000"),
        AudioQuality::Medium => ("128k", "22050"),
        AudioQuality::High => ("192k", "44100"),
    };

    let status = Command::new("ffmpeg")
        .args([
            "-i", video_path,
            "-vn",  // 不包含视频
            "-acodec", "pcm_s16le",
            "-ar", sample_rate,
            "-ab", bitrate,
            "-y",  // 覆盖输出文件
            output_path,
        ])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err("Audio extraction failed".into())
    }
}
```

### 2. 语音识别模块

#### 前端组件: RecognitionPanel.vue

**功能特性**:
- 识别引擎选择（百度、腾讯、阿里、Whisper）
- 识别语言配置
- 音频质量设置
- 实时进度显示
- 错误处理和重试机制

#### 后端服务: recognition_service.rs

**多引擎支持**:
```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait RecognitionEngine {
    async fn recognize(&self, audio_path: &str, language: &str) -> Result<Vec<SubtitleItem>, RecognitionError>;
}

/// 百度语音识别引擎
pub struct BaiduEngine {
    app_id: String,
    api_key: String,
    secret_key: String,
}

#[async_trait]
impl RecognitionEngine for BaiduEngine {
    async fn recognize(&self, audio_path: &str, language: &str) -> Result<Vec<SubtitleItem>, RecognitionError> {
        // 1. 获取访问令牌
        let token = self.get_access_token().await?;
        
        // 2. 读取音频文件
        let audio_data = tokio::fs::read(audio_path).await?;
        let audio_base64 = base64::encode(&audio_data);
        
        // 3. 构建请求
        let request_body = serde_json::json!({
            "format": "wav",
            "rate": 16000,
            "channel": 1,
            "cuid": "rust_client",
            "token": token,
            "speech": audio_base64,
            "len": audio_data.len()
        });
        
        // 4. 发送识别请求
        let client = reqwest::Client::new();
        let response = client
            .post("https://vop.baidu.com/server_api")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;
            
        // 5. 解析响应
        let result: BaiduResponse = response.json().await?;
        
        if result.err_no == 0 {
            Ok(self.parse_result(result.result))
        } else {
            Err(RecognitionError::ApiError(result.err_msg))
        }
    }
}

/// Whisper 本地识别引擎
pub struct WhisperEngine {
    model_path: String,
}

#[async_trait]
impl RecognitionEngine for WhisperEngine {
    async fn recognize(&self, audio_path: &str, language: &str) -> Result<Vec<SubtitleItem>, RecognitionError> {
        // 使用 whisper.cpp 或 Python whisper 库
        let output = tokio::process::Command::new("whisper")
            .args([
                audio_path,
                "--model", &self.model_path,
                "--language", language,
                "--output_format", "json",
                "--word_timestamps", "True",
            ])
            .output()
            .await?;
            
        if output.status.success() {
            let json_str = String::from_utf8(output.stdout)?;
            let whisper_result: WhisperResult = serde_json::from_str(&json_str)?;
            Ok(self.parse_whisper_result(whisper_result))
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(RecognitionError::ProcessError(error_msg.to_string()))
        }
    }
}
```

### 3. 字幕编辑模块

#### 前端组件: SubtitleEditor.vue

**功能特性**:
- 字幕列表展示和编辑
- 时间轴可视化编辑
- 字幕内容实时修改
- 字幕项添加/删除
- 批量操作支持
- 撤销/重做功能

**关键实现**:
```typescript
// 字幕编辑状态管理
const subtitleStore = useSubtitleStore();
const editingIndex = ref(-1);
const editingText = ref('');

// 开始编辑字幕
const startEdit = (index: number) => {
  editingIndex.value = index;
  editingText.value = subtitleStore.subtitles[index].text;
};

// 保存编辑
const saveEdit = () => {
  if (editingIndex.value >= 0) {
    subtitleStore.updateSubtitle(editingIndex.value, {
      text: editingText.value
    });
    editingIndex.value = -1;
  }
};

// 时间轴调整
const adjustTiming = (index: number, field: 'start' | 'end', value: number) => {
  subtitleStore.updateSubtitle(index, {
    [field]: value
  });
};

// 跳转到指定时间
const jumpToTime = (time: number) => {
  const videoStore = useVideoStore();
  videoStore.setCurrentTime(time);
};
```

#### 后端服务: subtitle_service.rs

**字幕格式支持**:
```rust
use chrono::{Duration, NaiveTime};
use std::fmt::Write;

/// 字幕项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleItem {
    pub index: usize,
    pub start_time: f64,
    pub end_time: f64,
    pub text: String,
}

/// 字幕格式枚举
#[derive(Debug, Clone)]
pub enum SubtitleFormat {
    Srt,
    Ass,
    Vtt,
    Txt,
    Json,
}

/// 字幕导出服务
pub struct SubtitleExporter;

impl SubtitleExporter {
    /// 导出为 SRT 格式
    pub fn export_srt(subtitles: &[SubtitleItem]) -> String {
        let mut output = String::new();
        
        for (i, subtitle) in subtitles.iter().enumerate() {
            writeln!(output, "{}", i + 1).unwrap();
            writeln!(
                output,
                "{} --> {}",
                Self::format_srt_time(subtitle.start_time),
                Self::format_srt_time(subtitle.end_time)
            ).unwrap();
            writeln!(output, "{}", subtitle.text).unwrap();
            writeln!(output).unwrap();
        }
        
        output
    }
    
    /// 导出为 ASS 格式
    pub fn export_ass(subtitles: &[SubtitleItem]) -> String {
        let mut output = String::new();
        
        // ASS 文件头
        writeln!(output, "[Script Info]").unwrap();
        writeln!(output, "Title: FlowText Generated Subtitles").unwrap();
        writeln!(output, "ScriptType: v4.00+").unwrap();
        writeln!(output).unwrap();
        
        writeln!(output, "[V4+ Styles]").unwrap();
        writeln!(output, "Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding").unwrap();
        writeln!(output, "Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1").unwrap();
        writeln!(output).unwrap();
        
        writeln!(output, "[Events]").unwrap();
        writeln!(output, "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text").unwrap();
        
        for subtitle in subtitles {
            writeln!(
                output,
                "Dialogue: 0,{},{},Default,,0,0,0,,{}",
                Self::format_ass_time(subtitle.start_time),
                Self::format_ass_time(subtitle.end_time),
                subtitle.text
            ).unwrap();
        }
        
        output
    }
    
    /// 格式化 SRT 时间
    fn format_srt_time(seconds: f64) -> String {
        let total_ms = (seconds * 1000.0) as u64;
        let ms = total_ms % 1000;
        let total_seconds = total_ms / 1000;
        let s = total_seconds % 60;
        let total_minutes = total_seconds / 60;
        let m = total_minutes % 60;
        let h = total_minutes / 60;
        
        format!("{:02}:{:02}:{:02},{:03}", h, m, s, ms)
    }
    
    /// 格式化 ASS 时间
    fn format_ass_time(seconds: f64) -> String {
        let total_cs = (seconds * 100.0) as u64;
        let cs = total_cs % 100;
        let total_seconds = total_cs / 100;
        let s = total_seconds % 60;
        let total_minutes = total_seconds / 60;
        let m = total_minutes % 60;
        let h = total_minutes / 60;
        
        format!("{:01}:{:02}:{:02}.{:02}", h, m, s, cs)
    }
}
```

## API 接口文档

### Tauri 命令接口

#### 视频处理相关

```rust
/// 获取视频信息
#[tauri::command]
pub async fn get_video_info(file_path: String) -> Result<VideoInfo, String> {
    video_service::get_video_info(&file_path)
        .await
        .map_err(|e| e.to_string())
}

/// 提取音频
#[tauri::command]
pub async fn extract_audio(
    video_path: String,
    output_path: String,
    quality: String,
) -> Result<(), String> {
    let quality = match quality.as_str() {
        "low" => AudioQuality::Low,
        "medium" => AudioQuality::Medium,
        "high" => AudioQuality::High,
        _ => AudioQuality::Medium,
    };
    
    video_service::extract_audio(&video_path, &output_path, quality)
        .await
        .map_err(|e| e.to_string())
}
```

#### 语音识别相关

```rust
/// 开始语音识别
#[tauri::command]
pub async fn start_recognition(
    audio_path: String,
    engine: String,
    language: String,
    config: RecognitionConfig,
) -> Result<String, String> {
    let task_id = uuid::Uuid::new_v4().to_string();
    
    // 异步执行识别任务
    tokio::spawn(async move {
        let result = recognition_service::recognize(
            &audio_path,
            &engine,
            &language,
            config,
        ).await;
        
        // 发送进度事件
        match result {
            Ok(subtitles) => {
                emit_event("recognition_complete", RecognitionResult {
                    task_id: task_id.clone(),
                    subtitles,
                    success: true,
                    error: None,
                });
            }
            Err(error) => {
                emit_event("recognition_error", RecognitionResult {
                    task_id: task_id.clone(),
                    subtitles: vec![],
                    success: false,
                    error: Some(error.to_string()),
                });
            }
        }
    });
    
    Ok(task_id)
}
```

#### 字幕处理相关

```rust
/// 导出字幕
#[tauri::command]
pub async fn export_subtitles(
    subtitles: Vec<SubtitleItem>,
    format: String,
    output_path: String,
) -> Result<(), String> {
    subtitle_service::export_subtitles(subtitles, &format, &output_path)
        .await
        .map_err(|e| e.to_string())
}

/// 导入字幕
#[tauri::command]
pub async fn import_subtitles(file_path: String) -> Result<Vec<SubtitleItem>, String> {
    subtitle_service::import_subtitles(&file_path)
        .await
        .map_err(|e| e.to_string())
}
```

### 事件系统

#### 进度事件

```typescript
// 监听识别进度
import { listen } from '@tauri-apps/api/event';

// 识别进度更新
listen('recognition_progress', (event) => {
  const { taskId, progress, stage } = event.payload;
  taskStore.updateProgress(taskId, progress, stage);
});

// 识别完成
listen('recognition_complete', (event) => {
  const { taskId, subtitles } = event.payload;
  subtitleStore.setSubtitles(subtitles);
  taskStore.completeTask(taskId);
});

// 识别错误
listen('recognition_error', (event) => {
  const { taskId, error } = event.payload;
  taskStore.setError(taskId, error);
});
```

## 数据流设计

### 状态管理架构

```typescript
// stores/video.ts
export const useVideoStore = defineStore('video', () => {
  const videoInfo = ref<VideoInfo | null>(null);
  const currentTime = ref(0);
  const duration = ref(0);
  const isPlaying = ref(false);
  
  const setVideoInfo = (info: VideoInfo) => {
    videoInfo.value = info;
    duration.value = info.duration;
  };
  
  const setCurrentTime = (time: number) => {
    currentTime.value = time;
  };
  
  const togglePlay = () => {
    isPlaying.value = !isPlaying.value;
  };
  
  return {
    videoInfo,
    currentTime,
    duration,
    isPlaying,
    setVideoInfo,
    setCurrentTime,
    togglePlay,
  };
});

// stores/subtitle.ts
export const useSubtitleStore = defineStore('subtitle', () => {
  const subtitles = ref<SubtitleItem[]>([]);
  const currentSubtitle = ref<SubtitleItem | null>(null);
  const editHistory = ref<SubtitleItem[][]>([]);
  const historyIndex = ref(-1);
  
  const setSubtitles = (newSubtitles: SubtitleItem[]) => {
    subtitles.value = newSubtitles;
    saveToHistory();
  };
  
  const updateSubtitle = (index: number, updates: Partial<SubtitleItem>) => {
    if (index >= 0 && index < subtitles.value.length) {
      subtitles.value[index] = { ...subtitles.value[index], ...updates };
      saveToHistory();
    }
  };
  
  const addSubtitle = (subtitle: SubtitleItem) => {
    subtitles.value.push(subtitle);
    subtitles.value.sort((a, b) => a.start_time - b.start_time);
    saveToHistory();
  };
  
  const removeSubtitle = (index: number) => {
    subtitles.value.splice(index, 1);
    saveToHistory();
  };
  
  const updateCurrentSubtitle = (currentTime: number) => {
    const current = subtitles.value.find(
      sub => currentTime >= sub.start_time && currentTime <= sub.end_time
    );
    currentSubtitle.value = current || null;
  };
  
  const saveToHistory = () => {
    editHistory.value = editHistory.value.slice(0, historyIndex.value + 1);
    editHistory.value.push([...subtitles.value]);
    historyIndex.value = editHistory.value.length - 1;
  };
  
  const undo = () => {
    if (historyIndex.value > 0) {
      historyIndex.value--;
      subtitles.value = [...editHistory.value[historyIndex.value]];
    }
  };
  
  const redo = () => {
    if (historyIndex.value < editHistory.value.length - 1) {
      historyIndex.value++;
      subtitles.value = [...editHistory.value[historyIndex.value]];
    }
  };
  
  return {
    subtitles,
    currentSubtitle,
    setSubtitles,
    updateSubtitle,
    addSubtitle,
    removeSubtitle,
    updateCurrentSubtitle,
    undo,
    redo,
  };
});
```

### 数据流向

1. **视频导入流程**:
   ```
   用户选择文件 → VideoPlayer.vue → get_video_info 命令 → video_service.rs → 返回视频信息 → videoStore
   ```

2. **音频提取流程**:
   ```
   用户点击识别 → RecognitionPanel.vue → extract_audio 命令 → video_service.rs → FFmpeg 处理 → 音频文件
   ```

3. **语音识别流程**:
   ```
   音频文件 → start_recognition 命令 → recognition_service.rs → 识别引擎 API → 识别结果 → 事件通知 → subtitleStore
   ```

4. **字幕编辑流程**:
   ```
   用户编辑 → SubtitleEditor.vue → subtitleStore → 状态更新 → 界面重渲染
   ```

5. **字幕导出流程**:
   ```
   用户导出 → export_subtitles 命令 → subtitle_service.rs → 格式转换 → 文件保存
   ```

## 性能优化

### 前端优化策略

1. **虚拟滚动**: 字幕列表使用虚拟滚动处理大量数据
2. **懒加载**: 组件按需加载，减少初始包大小
3. **缓存策略**: 视频信息和字幕数据本地缓存
4. **防抖节流**: 用户输入和滚动事件优化
5. **内存管理**: 及时清理不需要的数据和事件监听器

```typescript
// 虚拟滚动实现
import { FixedSizeList as List } from 'react-window';

const SubtitleList = () => {
  const subtitles = useSubtitleStore().subtitles;
  
  const Row = ({ index, style }) => (
    <div style={style}>
      <SubtitleItem subtitle={subtitles[index]} index={index} />
    </div>
  );
  
  return (
    <List
      height={600}
      itemCount={subtitles.length}
      itemSize={80}
      itemData={subtitles}
    >
      {Row}
    </List>
  );
};

// 防抖搜索
const searchText = ref('');
const debouncedSearch = debounce((text: string) => {
  // 执行搜索逻辑
  filterSubtitles(text);
}, 300);

watch(searchText, (newText) => {
  debouncedSearch(newText);
});
```

### 后端优化策略

1. **异步处理**: 所有 I/O 操作使用异步模式
2. **任务队列**: 语音识别任务队列化处理
3. **缓存机制**: 识别结果和音频文件缓存
4. **资源池**: 连接池和线程池管理
5. **内存优化**: 及时释放大文件占用的内存

```rust
// 任务队列实现
use tokio::sync::mpsc;
use std::collections::HashMap;

pub struct TaskManager {
    sender: mpsc::UnboundedSender<RecognitionTask>,
    tasks: Arc<Mutex<HashMap<String, TaskStatus>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let tasks = Arc::new(Mutex::new(HashMap::new()));
        let tasks_clone = tasks.clone();
        
        // 任务处理器
        tokio::spawn(async move {
            while let Some(task) = receiver.recv().await {
                let tasks = tasks_clone.clone();
                tokio::spawn(async move {
                    Self::process_task(task, tasks).await;
                });
            }
        });
        
        Self { sender, tasks }
    }
    
    pub fn submit_task(&self, task: RecognitionTask) -> String {
        let task_id = uuid::Uuid::new_v4().to_string();
        
        {
            let mut tasks = self.tasks.lock().unwrap();
            tasks.insert(task_id.clone(), TaskStatus::Pending);
        }
        
        self.sender.send(task).unwrap();
        task_id
    }
    
    async fn process_task(
        task: RecognitionTask,
        tasks: Arc<Mutex<HashMap<String, TaskStatus>>>,
    ) {
        // 更新任务状态为处理中
        {
            let mut tasks = tasks.lock().unwrap();
            tasks.insert(task.id.clone(), TaskStatus::Processing);
        }
        
        // 执行识别任务
        let result = recognition_service::recognize(
            &task.audio_path,
            &task.engine,
            &task.language,
            task.config,
        ).await;
        
        // 更新任务状态
        {
            let mut tasks = tasks.lock().unwrap();
            match result {
                Ok(_) => tasks.insert(task.id.clone(), TaskStatus::Completed),
                Err(_) => tasks.insert(task.id.clone(), TaskStatus::Failed),
            };
        }
    }
}
```

## 测试策略

### 单元测试

```typescript
// 前端单元测试 (Vitest)
import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useSubtitleStore } from '@/stores/subtitle';

describe('SubtitleStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });
  
  it('should add subtitle correctly', () => {
    const store = useSubtitleStore();
    const subtitle = {
      index: 0,
      start_time: 0,
      end_time: 5,
      text: 'Test subtitle'
    };
    
    store.addSubtitle(subtitle);
    
    expect(store.subtitles).toHaveLength(1);
    expect(store.subtitles[0]).toEqual(subtitle);
  });
  
  it('should update current subtitle based on time', () => {
    const store = useSubtitleStore();
    store.setSubtitles([
      { index: 0, start_time: 0, end_time: 5, text: 'First' },
      { index: 1, start_time: 5, end_time: 10, text: 'Second' },
    ]);
    
    store.updateCurrentSubtitle(3);
    expect(store.currentSubtitle?.text).toBe('First');
    
    store.updateCurrentSubtitle(7);
    expect(store.currentSubtitle?.text).toBe('Second');
    
    store.updateCurrentSubtitle(15);
    expect(store.currentSubtitle).toBeNull();
  });
});
```

```rust
// 后端单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    
    #[tokio::test]
    async fn test_video_info_extraction() {
        let video_path = "test_assets/sample.mp4";
        let result = get_video_info(video_path).await;
        
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(info.duration > 0.0);
        assert!(info.width > 0);
        assert!(info.height > 0);
    }
    
    #[tokio::test]
    async fn test_subtitle_export_srt() {
        let subtitles = vec![
            SubtitleItem {
                index: 0,
                start_time: 0.0,
                end_time: 5.0,
                text: "Hello world".to_string(),
            },
            SubtitleItem {
                index: 1,
                start_time: 5.0,
                end_time: 10.0,
                text: "Second subtitle".to_string(),
            },
        ];
        
        let srt_content = SubtitleExporter::export_srt(&subtitles);
        
        assert!(srt_content.contains("1"));
        assert!(srt_content.contains("00:00:00,000 --> 00:00:05,000"));
        assert!(srt_content.contains("Hello world"));
    }
}
```

### 集成测试

```typescript
// E2E 测试 (Playwright)
import { test, expect } from '@playwright/test';

test('complete subtitle generation workflow', async ({ page }) => {
  // 启动应用
  await page.goto('/');
  
  // 导入视频文件
  await page.setInputFiles('input[type="file"]', 'test_assets/sample.mp4');
  
  // 等待视频信息加载
  await expect(page.locator('.video-info')).toBeVisible();
  
  // 配置识别参数
  await page.selectOption('.engine-select', 'whisper');
  await page.selectOption('.language-select', 'zh');
  
  // 开始识别
  await page.click('.start-recognition');
  
  // 等待识别完成
  await expect(page.locator('.subtitle-list')).toBeVisible({ timeout: 30000 });
  
  // 验证字幕生成
  const subtitleItems = page.locator('.subtitle-item');
  await expect(subtitleItems).toHaveCountGreaterThan(0);
  
  // 编辑字幕
  await subtitleItems.first().dblclick();
  await page.fill('.subtitle-text-input', 'Edited subtitle');
  await page.press('.subtitle-text-input', 'Enter');
  
  // 导出字幕
  await page.click('.export-button');
  await page.selectOption('.format-select', 'srt');
  await page.click('.confirm-export');
  
  // 验证导出成功
  await expect(page.locator('.export-success')).toBeVisible();
});
```

## 部署指南

### 开发环境部署

```bash
# 克隆项目
git clone https://github.com/your-username/FlowText.git
cd FlowText

# 安装依赖
npm install

# 启动开发服务器
npm run tauri dev
```

### 生产环境构建

```bash
# 构建应用
npm run tauri build

# 构建产物位置
# macOS: src-tauri/target/release/bundle/dmg/
# Windows: src-tauri/target/release/bundle/msi/
# Linux: src-tauri/target/release/bundle/deb/
```

### CI/CD 配置

```yaml
# .github/workflows/build.yml
name: Build and Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: 18
          
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Install dependencies
        run: npm install
        
      - name: Build application
        run: npm run tauri build
        
      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: ${{ matrix.os }}-build
          path: src-tauri/target/release/bundle/
```

## 常见问题

### Q: FFmpeg 未找到错误

**问题**: 运行时提示 "ffmpeg not found" 或 "ffprobe not found"

**解决方案**:
1. **macOS**: `brew install ffmpeg`
2. **Windows**: 下载 FFmpeg 并添加到 PATH
3. **Linux**: `sudo apt install ffmpeg`

### Q: 语音识别 API 调用失败

**问题**: 识别任务失败，提示 API 错误

**解决方案**:
1. 检查 API 密钥配置是否正确
2. 确认网络连接正常
3. 检查音频文件格式是否支持
4. 查看 API 配额是否用完

### Q: 应用启动缓慢

**问题**: 应用启动时间过长

**解决方案**:
1. 检查是否有大量缓存文件
2. 优化启动时的初始化逻辑
3. 使用 `cargo build --release` 构建优化版本
4. 检查系统资源使用情况

### Q: 字幕时间轴不准确

**问题**: 生成的字幕时间与视频不同步

**解决方案**:
1. 检查音频提取质量设置
2. 尝试不同的识别引擎
3. 手动调整字幕时间轴
4. 确认视频文件没有损坏

### Q: 内存使用过高

**问题**: 处理大文件时内存占用过多

**解决方案**:
1. 分段处理大文件
2. 及时清理临时文件
3. 优化音频质量设置
4. 增加系统虚拟内存

---

## 贡献指南

欢迎提交 Issue 和 Pull Request！请确保：

1. 代码符合项目规范
2. 添加适当的测试
3. 更新相关文档
4. 提交信息清晰明确

## 许可证

本项目采用 MIT 许可证，详见 [LICENSE](LICENSE) 文件。