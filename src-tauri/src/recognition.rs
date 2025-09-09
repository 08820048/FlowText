use base64::Engine;
use chrono::Utc;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

// 模型配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub engine: String,
    pub size: String,
    pub device: String,
    pub compute_type: Option<String>,
    pub beam_size: Option<i32>,
    pub temperature: Option<f32>,
    pub enable_emotion_recognition: Option<bool>,
    pub enable_event_detection: Option<bool>,
}

// 扩展的识别参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedRecognitionParams {
    pub audio_path: String,
    pub engine: String,
    pub language: String,
    pub model_config: ModelConfig,
}

// 全局任务状态管理
lazy_static::lazy_static! {
    static ref RECOGNITION_TASKS: Arc<Mutex<HashMap<String, RecognitionTask>>> = {
        Arc::new(Mutex::new(HashMap::new()))
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognitionTask {
    pub task_id: String,
    pub audio_path: String,
    pub engine: String,
    pub language: String,
    pub status: RecognitionStatus,
    #[serde(skip)]
    pub cancel_sender: Option<mpsc::Sender<()>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognitionStatus {
    pub status: String, // "pending", "processing", "completed", "failed", "cancelled"
    pub progress: f32,  // 0.0 - 1.0
    #[serde(rename = "subtitles")]
    pub result: Option<Vec<crate::video::Subtitle>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Language {
    pub code: String,
    pub name: String,
}

/// 开始语音识别任务
pub fn start_recognition(
    task_id: String,
    audio_path: &str,
    engine: &str,
    language: &str,
    api_keys: Option<Value>,
) -> Result<(), String> {
    // 检查任务是否已存在
    let mut tasks = RECOGNITION_TASKS
        .lock()
        .map_err(|_| "无法获取任务锁".to_string())?;

    if tasks.contains_key(&task_id) {
        return Err(format!("任务ID已存在: {}", task_id));
    }

    // 创建取消通道
    let (cancel_tx, mut cancel_rx) = mpsc::channel::<()>(1);

    // 创建新任务
    let task = RecognitionTask {
        task_id: task_id.clone(),
        audio_path: audio_path.to_string(),
        engine: engine.to_string(),
        language: language.to_string(),
        status: RecognitionStatus {
            status: "pending".to_string(),
            progress: 0.0,
            result: None,
            error: None,
        },
        cancel_sender: Some(cancel_tx),
    };

    // 存储任务
    tasks.insert(task_id.clone(), task);

    // 启动异步任务
    let task_id_clone = task_id.clone();
    let audio_path = audio_path.to_string();
    let engine = engine.to_string();
    let language = language.to_string();

    tokio::spawn(async move {
        // 更新状态为处理中
        update_task_status(&task_id_clone, "processing".to_string(), 0.0, None, None);

        // 使用Whisper本地识别
        let result = {
            println!("使用Whisper引擎进行本地识别...");
            println!("音频文件路径: {}", audio_path);

            match call_whisper_api(&audio_path, &language, &task_id_clone, &mut cancel_rx).await {
                Ok(subtitles) => {
                    println!("Whisper识别成功，共生成{}条字幕", subtitles.len());
                    Ok(subtitles)
                }
                Err(e) => {
                    eprintln!("Whisper识别失败: {}", e);
                    // 如果Whisper未安装，提供安装指导和测试数据
                    if e.contains("未找到whisper") || e.contains("ModuleNotFoundError") {
                        println!("生成Whisper安装指导的测试数据...");
                        let installation_guide = generate_whisper_installation_guide(&audio_path);
                        Ok(installation_guide)
                    } else {
                        Err(format!("Whisper识别失败: {}", e))
                    }
                }
            }
        };

        // 处理结果
        match result {
            Ok(subtitles) => {
                update_task_status(
                    &task_id_clone,
                    "completed".to_string(),
                    1.0,
                    Some(subtitles),
                    None,
                );

                // 延迟清理已完成的任务（给前端足够时间获取结果）
                let cleanup_task_id = task_id_clone.clone();
                tokio::spawn(async move {
                    sleep(Duration::from_secs(1800)).await; // 30分钟后清理，给前端足够时间
                    cleanup_completed_task(&cleanup_task_id);
                });
            }
            Err(err) => {
                update_task_status(&task_id_clone, "failed".to_string(), 0.0, None, Some(err));

                // 延迟清理失败的任务
                let cleanup_task_id = task_id_clone.clone();
                tokio::spawn(async move {
                    sleep(Duration::from_secs(1800)).await; // 30分钟后清理，给前端足够时间
                    cleanup_completed_task(&cleanup_task_id);
                });
            }
        }
    });

    Ok(())
}

/// 使用扩展配置开始识别
pub fn start_recognition_with_config(
    task_id: String,
    params: ExtendedRecognitionParams,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("开始扩展配置识别任务: {}", task_id);
    println!("识别参数: {:?}", params);

    // 创建取消通道
    let (cancel_tx, mut cancel_rx) = mpsc::channel::<()>(1);

    // 创建识别任务
    let task = RecognitionTask {
        task_id: task_id.clone(),
        audio_path: params.audio_path.clone(),
        engine: params.engine.clone(),
        language: params.language.clone(),
        status: RecognitionStatus {
            status: "pending".to_string(),
            progress: 0.0,
            result: None,
            error: None,
        },
        cancel_sender: Some(cancel_tx),
    };

    // 存储任务
    {
        let mut tasks = RECOGNITION_TASKS.lock().unwrap();
        tasks.insert(task_id.clone(), task);
    }

    let task_id_clone = task_id.clone();

    tokio::spawn(async move {
        // 更新状态为处理中
        update_task_status(&task_id_clone, "processing".to_string(), 0.0, None, None);

        // 使用扩展配置进行识别
        let result = call_recognition_with_config(&params, &task_id_clone, &mut cancel_rx).await;

        match result {
            Ok(subtitles) => {
                println!("识别成功，共生成{}条字幕", subtitles.len());
                update_task_status(
                    &task_id_clone,
                    "completed".to_string(),
                    100.0,
                    Some(subtitles),
                    None,
                );
            }
            Err(e) => {
                eprintln!("识别失败: {}", e);
                update_task_status(&task_id_clone, "failed".to_string(), 0.0, None, Some(e));
            }
        }
    });

    Ok(())
}

/// 获取任务状态
pub fn get_recognition_status(task_id: &str) -> Result<RecognitionStatus, String> {
    let tasks = RECOGNITION_TASKS
        .lock()
        .map_err(|_| "无法获取任务锁".to_string())?;

    tasks
        .get(task_id)
        .map(|task| task.status.clone())
        .ok_or_else(|| format!("任务不存在: {}", task_id))
}

/// 取消识别任务
pub fn cancel_recognition(task_id: &str) -> Result<(), String> {
    let mut tasks = RECOGNITION_TASKS
        .lock()
        .map_err(|_| "无法获取任务锁".to_string())?;

    if let Some(task) = tasks.get_mut(task_id) {
        if let Some(sender) = task.cancel_sender.take() {
            // 发送取消信号
            let _ = sender.try_send(());
            return Ok(());
        }
    }

    Err(format!("无法取消任务: {}", task_id))
}

/// 获取支持的语言列表
pub fn get_supported_languages(engine: &str) -> Result<Vec<Language>, String> {
    match engine {
        "whisper" => {
            // Whisper支持的语言列表
            let languages = vec![
                Language {
                    code: "zh".to_string(),
                    name: "中文".to_string(),
                },
                Language {
                    code: "en".to_string(),
                    name: "英语".to_string(),
                },
                Language {
                    code: "ja".to_string(),
                    name: "日语".to_string(),
                },
                Language {
                    code: "ko".to_string(),
                    name: "韩语".to_string(),
                },
                Language {
                    code: "fr".to_string(),
                    name: "法语".to_string(),
                },
                Language {
                    code: "de".to_string(),
                    name: "德语".to_string(),
                },
                Language {
                    code: "es".to_string(),
                    name: "西班牙语".to_string(),
                },
                Language {
                    code: "ru".to_string(),
                    name: "俄语".to_string(),
                },
            ];
            Ok(languages)
        }
        "tencent" => {
            // 腾讯云支持的语言列表
            let languages = vec![
                Language {
                    code: "zh".to_string(),
                    name: "中文".to_string(),
                },
                Language {
                    code: "en".to_string(),
                    name: "英语".to_string(),
                },
                Language {
                    code: "ja".to_string(),
                    name: "日语".to_string(),
                },
                Language {
                    code: "ko".to_string(),
                    name: "韩语".to_string(),
                },
            ];
            Ok(languages)
        }
        "aliyun" => {
            // 阿里云支持的语言列表
            let languages = vec![
                Language {
                    code: "zh".to_string(),
                    name: "中文".to_string(),
                },
                Language {
                    code: "en".to_string(),
                    name: "英语".to_string(),
                },
                Language {
                    code: "ja".to_string(),
                    name: "日语".to_string(),
                },
                Language {
                    code: "ko".to_string(),
                    name: "韩语".to_string(),
                },
            ];
            Ok(languages)
        }
        "baidu" => {
            // 百度支持的语言列表
            let languages = vec![
                Language {
                    code: "zh".to_string(),
                    name: "中文".to_string(),
                },
                Language {
                    code: "en".to_string(),
                    name: "英语".to_string(),
                },
                Language {
                    code: "jp".to_string(),
                    name: "日语".to_string(),
                },
                Language {
                    code: "kor".to_string(),
                    name: "韩语".to_string(),
                },
            ];
            Ok(languages)
        }
        "google" => {
            // Google支持的语言列表
            let languages = vec![
                Language {
                    code: "zh-CN".to_string(),
                    name: "中文（简体）".to_string(),
                },
                Language {
                    code: "zh-TW".to_string(),
                    name: "中文（繁体）".to_string(),
                },
                Language {
                    code: "en-US".to_string(),
                    name: "英语（美国）".to_string(),
                },
                Language {
                    code: "en-GB".to_string(),
                    name: "英语（英国）".to_string(),
                },
                Language {
                    code: "ja-JP".to_string(),
                    name: "日语".to_string(),
                },
                Language {
                    code: "ko-KR".to_string(),
                    name: "韩语".to_string(),
                },
                Language {
                    code: "fr-FR".to_string(),
                    name: "法语".to_string(),
                },
                Language {
                    code: "de-DE".to_string(),
                    name: "德语".to_string(),
                },
            ];
            Ok(languages)
        }
        _ => Err(format!("不支持的识别引擎: {}", engine)),
    }
}

/// 获取可用模型列表
pub fn get_available_models(
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
    let models = vec![
        serde_json::json!({
            "engine": "whisper",
            "name": "OpenAI Whisper",
            "description": "OpenAI开源的多语言语音识别模型",
            "sizes": ["tiny", "base", "small", "medium", "large", "large-v2", "large-v3"],
            "features": ["multilingual", "timestamps", "translation"],
            "requirements": {
                "python": ">=3.8",
                "packages": ["openai-whisper", "torch", "torchaudio"]
            }
        }),
        serde_json::json!({
            "engine": "faster-whisper",
            "name": "Faster Whisper",
            "description": "基于CTranslate2优化的Whisper实现，速度提升4-5倍",
            "sizes": ["tiny", "base", "small", "medium", "large", "large-v2", "large-v3"],
            "features": ["multilingual", "timestamps", "optimization", "quantization"],
            "requirements": {
                "python": ">=3.8",
                "packages": ["faster-whisper", "ctranslate2"]
            }
        }),
        serde_json::json!({
            "engine": "sensevoice",
            "name": "SenseVoice",
            "description": "阿里巴巴开源的多语言语音理解模型，支持情感识别和事件检测",
            "sizes": ["small", "large"],
            "features": ["multilingual", "timestamps", "emotion", "event", "language_id"],
            "requirements": {
                "python": ">=3.8",
                "packages": ["funasr", "modelscope", "torch", "torchaudio"]
            }
        }),
    ];

    Ok(models)
}

/// 检查模型是否已安装
pub fn check_model_installation(
    engine: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    match engine {
        "whisper" => {
            // 检查whisper命令或Python包
            let whisper_cmd = std::process::Command::new("whisper").arg("--help").output();
            if whisper_cmd.is_ok() {
                return Ok(true);
            }

            // 检查Python whisper包
            let python_check = std::process::Command::new("python3")
                .arg("-c")
                .arg("import whisper; print('OK')")
                .output();

            Ok(python_check.is_ok() && python_check.unwrap().status.success())
        }
        "faster-whisper" => {
            // 检查faster-whisper Python包
            let python_check = std::process::Command::new("python3")
                .arg("-c")
                .arg("import faster_whisper; print('OK')")
                .output();

            Ok(python_check.is_ok() && python_check.unwrap().status.success())
        }
        "sensevoice" => {
            // 检查funasr Python包
            let python_check = std::process::Command::new("python3")
                .arg("-c")
                .arg("import funasr; print('OK')")
                .output();

            Ok(python_check.is_ok() && python_check.unwrap().status.success())
        }
        _ => Ok(false),
    }
}

/// 获取模型详细信息
pub fn get_model_info(
    engine: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let models = get_available_models()?;

    for model in models {
        if model["engine"].as_str() == Some(engine) {
            let mut info = model.clone();

            // 添加安装状态
            let installed = check_model_installation(engine).unwrap_or(false);
            info["installed"] = serde_json::Value::Bool(installed);

            // 添加性能信息
            match engine {
                "whisper" => {
                    info["performance"] = serde_json::json!({
                        "wer": 0.15,
                        "rtf": 0.3,
                        "latency": 2000,
                        "throughput": 150
                    });
                }
                "faster-whisper" => {
                    info["performance"] = serde_json::json!({
                        "wer": 0.15,
                        "rtf": 0.06,
                        "latency": 500,
                        "throughput": 600
                    });
                }
                "sensevoice" => {
                    info["performance"] = serde_json::json!({
                        "wer": 0.12,
                        "rtf": 0.1,
                        "latency": 800,
                        "throughput": 400
                    });
                }
                _ => {}
            }

            return Ok(info);
        }
    }

    Err(format!("未找到模型: {}", engine).into())
}

/// 验证API密钥
pub fn validate_api_keys(engine: &str, api_keys: Value) -> Result<bool, String> {
    // 在实际应用中，这里应该调用相应API进行验证
    // 目前简单返回true表示验证成功
    match engine {
        "whisper" => Ok(true), // Whisper不需要API密钥
        "tencent" => {
            // 检查必要的密钥
            if let Some(obj) = api_keys.as_object() {
                if obj.contains_key("secretId") && obj.contains_key("secretKey") {
                    return Ok(true);
                }
            }
            Err("腾讯云API需要提供secretId和secretKey".to_string())
        }
        "aliyun" => {
            // 检查必要的密钥
            if let Some(obj) = api_keys.as_object() {
                if obj.contains_key("accessKeyId") && obj.contains_key("accessKeySecret") {
                    return Ok(true);
                }
            }
            Err("阿里云API需要提供accessKeyId和accessKeySecret".to_string())
        }
        "baidu" => {
            // 检查必要的密钥
            if let Some(obj) = api_keys.as_object() {
                if obj.contains_key("api_key") && obj.contains_key("secret_key") {
                    return Ok(true);
                }
            }
            Err("百度API需要提供api_key和secret_key".to_string())
        }
        "google" => {
            // 检查必要的密钥
            if let Some(obj) = api_keys.as_object() {
                if obj.contains_key("api_key") {
                    return Ok(true);
                }
            }
            Err("Google API需要提供api_key".to_string())
        }
        _ => Err(format!("不支持的识别引擎: {}", engine)),
    }
}

/// 更新任务状态
fn update_task_status(
    task_id: &str,
    state: String,
    progress: f32,
    result: Option<Vec<crate::video::Subtitle>>,
    error: Option<String>,
) {
    println!("更新任务状态: {} -> {}, 进度: {}", task_id, state, progress);
    if let Ok(mut tasks) = RECOGNITION_TASKS.lock() {
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = RecognitionStatus {
                status: state.clone(),
                progress,
                result,
                error,
            };
            println!("任务状态已更新: {} -> {}", task_id, state);
        } else {
            println!("警告: 任务不存在: {}", task_id);
        }
    } else {
        println!("错误: 无法获取任务锁");
    }
}

/// 清理已完成的任务
fn cleanup_completed_task(task_id: &str) {
    if let Ok(mut tasks) = RECOGNITION_TASKS.lock() {
        if let Some(task) = tasks.get(task_id) {
            // 只清理已完成、失败或取消的任务
            match task.status.status.as_str() {
                "completed" | "failed" | "cancelled" => {
                    println!("清理已完成的任务: {}", task_id);
                    tasks.remove(task_id);
                }
                _ => {
                    // 任务仍在进行中，不清理
                }
            }
        }
    }
}

/// 使用配置参数进行识别（新的统一入口）
async fn call_recognition_with_config(
    params: &ExtendedRecognitionParams,
    task_id: &str,
    cancel_rx: &mut mpsc::Receiver<()>,
) -> Result<Vec<crate::video::Subtitle>, String> {
    use std::fs;
    use std::process::Command;

    // 检查音频文件是否存在
    if !std::path::Path::new(&params.audio_path).exists() {
        return Err(format!("音频文件不存在: {}", params.audio_path));
    }

    update_task_status(task_id, "processing".to_string(), 0.1, None, None);

    match params.engine.as_str() {
        "whisper" => call_whisper_with_config(params, task_id, cancel_rx).await,
        "faster-whisper" => call_faster_whisper_with_config(params, task_id, cancel_rx).await,
        "sensevoice" => call_sensevoice_with_config(params, task_id, cancel_rx).await,
        _ => {
            // 回退到原有的Whisper API
            call_whisper_api(&params.audio_path, &params.language, task_id, cancel_rx).await
        }
    }
}

/// 调用Whisper进行本地语音识别
async fn call_whisper_api(
    audio_path: &str,
    language: &str,
    task_id: &str,
    cancel_rx: &mut mpsc::Receiver<()>,
) -> Result<Vec<crate::video::Subtitle>, String> {
    // 检查音频文件是否存在
    if !std::path::Path::new(audio_path).exists() {
        return Err(format!("音频文件不存在: {}", audio_path));
    }

    update_task_status(task_id, "processing".to_string(), 0.1, None, None);

    // 检查whisper命令是否可用
    let whisper_check = Command::new("whisper").arg("--help").output();

    match whisper_check {
        Ok(_) => {
            println!("发现whisper命令，使用本地Whisper进行识别");
            call_local_whisper(audio_path, language, task_id, cancel_rx).await
        }
        Err(_) => {
            println!("未找到whisper命令，尝试使用Python whisper");
            call_python_whisper(audio_path, language, task_id, cancel_rx).await
        }
    }
}

/// 使用配置的Whisper进行识别
async fn call_whisper_with_config(
    params: &ExtendedRecognitionParams,
    task_id: &str,
    cancel_rx: &mut mpsc::Receiver<()>,
) -> Result<Vec<crate::video::Subtitle>, String> {
    println!("使用Whisper模型进行识别，配置: {:?}", params.model_config);

    // 检查whisper命令是否可用
    let whisper_check = Command::new("whisper").arg("--help").output();

    match whisper_check {
        Ok(_) => {
            println!("发现whisper命令，使用本地Whisper进行识别");
            call_local_whisper_with_config(params, task_id, cancel_rx).await
        }
        Err(_) => {
            println!("未找到whisper命令，尝试使用Python whisper");
            call_python_whisper_with_config(params, task_id, cancel_rx).await
        }
    }
}

/// 使用配置的Faster-Whisper进行识别
async fn call_faster_whisper_with_config(
    params: &ExtendedRecognitionParams,
    task_id: &str,
    cancel_rx: &mut mpsc::Receiver<()>,
) -> Result<Vec<crate::video::Subtitle>, String> {
    println!(
        "使用Faster-Whisper模型进行识别，配置: {:?}",
        params.model_config
    );

    update_task_status(task_id, "processing".to_string(), 0.3, None, None);

    // 构建Python脚本来调用faster-whisper
    let python_script = format!(
        r#"
import sys
import json
from faster_whisper import WhisperModel

try:
    # 初始化模型
    model = WhisperModel(
        "{model_size}",
        device="{device}",
        compute_type="{compute_type}"
    )

    # 设置参数
    beam_size = {beam_size}
    temperature = {temperature}

    # 进行识别
    segments, info = model.transcribe(
        "{audio_path}",
        language="{language}" if "{language}" != "auto" else None,
        beam_size=beam_size,
        temperature=temperature,
        word_timestamps=True
    )

    # 输出SRT格式
    for i, segment in enumerate(segments):
        start = segment.start
        end = segment.end
        text = segment.text.strip()

        start_time = f"{{:02d}}:{{:02d}}:{{:06.3f}}".format(
            int(start // 3600),
            int((start % 3600) // 60),
            start % 60
        )
        end_time = f"{{:02d}}:{{:02d}}:{{:06.3f}}".format(
            int(end // 3600),
            int((end % 3600) // 60),
            end % 60
        )

        print(f"{{i+1}}")
        print(f"{{start_time}} --> {{end_time}}")
        print(text)
        print()

except Exception as e:
    print(f"Error: {{e}}", file=sys.stderr)
    sys.exit(1)
"#,
        model_size = params.model_config.size,
        device = params.model_config.device,
        compute_type = params
            .model_config
            .compute_type
            .as_ref()
            .unwrap_or(&"int8".to_string()),
        beam_size = params.model_config.beam_size.unwrap_or(5),
        temperature = params.model_config.temperature.unwrap_or(0.0),
        audio_path = params.audio_path,
        language = if params.language == "zh" {
            "zh"
        } else {
            &params.language
        }
    );

    // 执行Python脚本
    let output = Command::new("python3")
        .arg("-c")
        .arg(&python_script)
        .output()
        .map_err(|e| format!("执行faster-whisper失败: {}", e))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Faster-Whisper识别失败: {}", error_msg));
    }

    // 解析输出
    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_srt_content(&output_str)
}

/// 使用配置的SenseVoice进行识别
async fn call_sensevoice_with_config(
    params: &ExtendedRecognitionParams,
    task_id: &str,
    cancel_rx: &mut mpsc::Receiver<()>,
) -> Result<Vec<crate::video::Subtitle>, String> {
    println!(
        "使用SenseVoice模型进行识别，配置: {:?}",
        params.model_config
    );

    update_task_status(task_id, "processing".to_string(), 0.3, None, None);

    // 构建Python脚本来调用SenseVoice
    let python_script = format!(
        r#"
import sys
from funasr import AutoModel

try:
    # 初始化SenseVoice模型
    model = AutoModel(
        model="iic/SenseVoiceSmall" if "{model_size}" == "small" else "iic/SenseVoiceLarge",
        device="{device}"
    )

    # 进行识别
    result = model.generate(
        input="{audio_path}",
        language="{language}" if "{language}" != "auto" else "auto",
        use_itn=True,
        batch_size_s=60
    )

    # 输出SRT格式
    if result and len(result) > 0:
        for i, item in enumerate(result):
            if 'text' in item:
                # SenseVoice通常返回整段文本，需要分段处理
                text = item['text'].strip()
                if text:
                    # 简单分段，实际应用中可能需要更复杂的分段逻辑
                    duration = 5.0  # 假设每段5秒
                    start_time = i * duration
                    end_time = start_time + duration

                    start_formatted = f"{{:02d}}:{{:02d}}:{{:06.3f}}".format(
                        int(start_time // 3600),
                        int((start_time % 3600) // 60),
                        start_time % 60
                    )
                    end_formatted = f"{{:02d}}:{{:02d}}:{{:06.3f}}".format(
                        int(end_time // 3600),
                        int((end_time % 3600) // 60),
                        end_time % 60
                    )

                    print(f"{{i+1}}")
                    print(f"{{start_formatted}} --> {{end_formatted}}")
                    print(text)
                    print()

                    # 情感和事件信息暂时跳过，避免格式问题

except Exception as e:
    print(f"Error: {{e}}", file=sys.stderr)
    sys.exit(1)
"#,
        model_size = params.model_config.size,
        device = params.model_config.device,
        audio_path = params.audio_path,
        language = if params.language == "zh" {
            "zh"
        } else {
            &params.language
        }
    );

    // 执行Python脚本
    let output = Command::new("python3")
        .arg("-c")
        .arg(&python_script)
        .output()
        .map_err(|e| format!("执行SenseVoice失败: {}", e))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("SenseVoice识别失败: {}", error_msg));
    }

    // 解析输出
    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_srt_content(&output_str)
}

/// 使用配置的本地whisper命令进行识别
async fn call_local_whisper_with_config(
    params: &ExtendedRecognitionParams,
    task_id: &str,
    cancel_rx: &mut mpsc::Receiver<()>,
) -> Result<Vec<crate::video::Subtitle>, String> {
    use std::path::Path;
    use std::process::Command;

    let audio_file = Path::new(&params.audio_path);
    let output_dir = audio_file.parent().unwrap_or(Path::new("."));
    let file_stem = audio_file.file_stem().unwrap().to_string_lossy();

    update_task_status(task_id, "processing".to_string(), 0.3, None, None);

    // 构建whisper命令
    let mut cmd = Command::new("whisper");
    cmd.arg(&params.audio_path)
        .arg("--model")
        .arg(&params.model_config.size) // 使用配置的模型大小
        .arg("--output_format")
        .arg("srt")
        .arg("--output_dir")
        .arg(output_dir)
        .arg("--verbose")
        .arg("False")
        .arg("--task")
        .arg("transcribe"); // 明确指定转写任务

    // 添加设备参数
    if params.model_config.device == "gpu" {
        cmd.arg("--device").arg("cuda");
    }

    // 添加温度参数
    if let Some(temperature) = params.model_config.temperature {
        cmd.arg("--temperature").arg(temperature.to_string());
    }

    // 设置语言
    if params.language == "zh" || params.language == "zh-CN" || params.language.is_empty() {
        cmd.arg("--language").arg("zh");
        cmd.arg("--initial_prompt").arg("以下是简体中文语音：");
    } else {
        let whisper_lang = match params.language.as_str() {
            "en" => "en",
            "ja" => "ja",
            "ko" => "ko",
            "fr" => "fr",
            "de" => "de",
            "es" => "es",
            "ru" => "ru",
            _ => "auto",
        };
        cmd.arg("--language").arg(whisper_lang);
    }

    println!("执行whisper命令: {:?}", cmd);

    // 执行命令
    let output = cmd
        .output()
        .map_err(|e| format!("执行whisper命令失败: {}", e))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Whisper命令执行失败: {}", error_msg));
    }

    update_task_status(task_id, "processing".to_string(), 0.8, None, None);

    // 读取生成的SRT文件
    let srt_file_path = output_dir.join(format!("{}.srt", file_stem));

    if !srt_file_path.exists() {
        return Err("未找到生成的SRT文件".to_string());
    }

    let srt_content =
        std::fs::read_to_string(&srt_file_path).map_err(|e| format!("读取SRT文件失败: {}", e))?;

    // 解析SRT内容
    parse_srt_content(&srt_content)
}

/// 使用配置的Python whisper进行识别
async fn call_python_whisper_with_config(
    params: &ExtendedRecognitionParams,
    task_id: &str,
    _cancel_rx: &mut mpsc::Receiver<()>,
) -> Result<Vec<crate::video::Subtitle>, String> {
    update_task_status(task_id, "processing".to_string(), 0.3, None, None);

    // 构建Python脚本
    let python_script = format!(
        r#"
import sys
import whisper

try:
    model = whisper.load_model("{model_size}")
    result = model.transcribe(
        "{audio_path}",
        language="{language}" if "{language}" != "auto" else None,
        temperature={temperature}
    )

    # 输出SRT格式
    for i, segment in enumerate(result['segments']):
        start = segment['start']
        end = segment['end']
        text = segment['text'].strip()

        start_time = f"{{:02d}}:{{:02d}}:{{:06.3f}}".format(
            int(start // 3600),
            int((start % 3600) // 60),
            start % 60
        )
        end_time = f"{{:02d}}:{{:02d}}:{{:06.3f}}".format(
            int(end // 3600),
            int((end % 3600) // 60),
            end % 60
        )

        print(f"{{i+1}}")
        print(f"{{start_time}} --> {{end_time}}")
        print(text)
        print()

except Exception as e:
    print(f"Error: {{e}}", file=sys.stderr)
    sys.exit(1)
"#,
        model_size = params.model_config.size,
        audio_path = params.audio_path,
        language = if params.language == "zh" {
            "zh"
        } else {
            &params.language
        },
        temperature = params.model_config.temperature.unwrap_or(0.0)
    );

    // 执行Python脚本
    let output = Command::new("python3")
        .arg("-c")
        .arg(&python_script)
        .output()
        .map_err(|e| format!("执行Python whisper失败: {}", e))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Python Whisper识别失败: {}", error_msg));
    }

    // 解析输出
    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_srt_content(&output_str)
}

/// 使用本地whisper命令进行识别
async fn call_local_whisper(
    audio_path: &str,
    language: &str,
    task_id: &str,
    _cancel_rx: &mut mpsc::Receiver<()>,
) -> Result<Vec<crate::video::Subtitle>, String> {
    use std::path::Path;
    use std::process::Command;

    let audio_file = Path::new(audio_path);
    let output_dir = audio_file.parent().unwrap_or(Path::new("."));
    let file_stem = audio_file.file_stem().unwrap().to_string_lossy();

    update_task_status(task_id, "processing".to_string(), 0.3, None, None);

    // 构建whisper命令
    let mut cmd = Command::new("whisper");
    cmd.arg(audio_path)
        .arg("--model")
        .arg("base") // 使用base模型，平衡速度和精度
        .arg("--output_format")
        .arg("srt")
        .arg("--output_dir")
        .arg(output_dir)
        .arg("--verbose")
        .arg("False")
        .arg("--task")
        .arg("transcribe"); // 明确指定转写任务

    // 设置语言（强制使用简体中文）
    if language == "zh" || language == "zh-CN" || language.is_empty() {
        cmd.arg("--language").arg("zh");
        // 添加简体中文输出参数
        cmd.arg("--initial_prompt").arg("以下是简体中文语音：");
    } else {
        let whisper_lang = match language {
            "en" | "en-US" => "en",
            "ja" | "ja-JP" => "ja",
            "ko" | "ko-KR" => "ko",
            _ => "zh",
        };
        cmd.arg("--language").arg(whisper_lang);
    }

    println!("执行Whisper命令: {:?}", cmd);

    // 执行命令
    update_task_status(task_id, "processing".to_string(), 0.5, None, None);

    let output = cmd
        .output()
        .map_err(|e| format!("执行whisper命令失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Whisper执行失败: {}", stderr));
    }

    update_task_status(task_id, "processing".to_string(), 0.8, None, None);

    // 读取生成的SRT文件
    let srt_path = output_dir.join(format!("{}.srt", file_stem));

    if !srt_path.exists() {
        return Err("Whisper未生成SRT文件".to_string());
    }

    let srt_content =
        std::fs::read_to_string(&srt_path).map_err(|e| format!("读取SRT文件失败: {}", e))?;

    // 解析SRT文件
    let subtitles = parse_srt_content(&srt_content)?;

    // 清理临时文件
    let _ = std::fs::remove_file(&srt_path);

    println!("Whisper识别完成，共解析到{}条字幕", subtitles.len());
    Ok(subtitles)
}

/// 使用Python whisper进行识别
async fn call_python_whisper(
    audio_path: &str,
    language: &str,
    task_id: &str,
    cancel_rx: &mut mpsc::Receiver<()>,
) -> Result<Vec<crate::video::Subtitle>, String> {
    use std::path::Path;
    use std::process::Command;

    // 检查是否安装了openai-whisper
    let python_check = Command::new("python3")
        .args(["-c", "import whisper; print('whisper available')"])
        .output();

    match python_check {
        Ok(output) if output.status.success() => {
            println!("发现Python whisper库");
        }
        _ => {
            return Err("未找到whisper。请安装: pip install openai-whisper".to_string());
        }
    }

    let audio_file = Path::new(audio_path);
    let output_dir = audio_file.parent().unwrap_or(Path::new("."));
    let file_stem = audio_file.file_stem().unwrap().to_string_lossy();

    update_task_status(task_id, "processing".to_string(), 0.3, None, None);

    // 创建Python脚本（强制简体中文输出）
    let python_script = format!(
        r#"
import whisper
import sys

try:
    import opencc
    converter = opencc.OpenCC('t2s')  # 繁体转简体
except ImportError:
    print("Warning: opencc not available, skipping traditional to simplified conversion", file=sys.stderr)
    converter = None

try:
    model = whisper.load_model("base")
    # 强制使用中文识别，并指定简体中文提示
    result = model.transcribe("{}", language="zh", initial_prompt="以下是简体中文语音：")
    
    # 输出SRT格式
    for i, segment in enumerate(result['segments']):
        start = segment['start']
        end = segment['end']
        text = segment['text'].strip()
        
        # 转换为简体中文
        if converter and text:
            try:
                text = converter.convert(text)
            except:
                pass  # 如果转换失败，保持原文
        
        start_time = f"{{:02d}}:{{:02d}}:{{:06.3f}}".format(
            int(start // 3600),
            int((start % 3600) // 60),
            start % 60
        )
        end_time = f"{{:02d}}:{{:02d}}:{{:06.3f}}".format(
            int(end // 3600),
            int((end % 3600) // 60),
            end % 60
        )
        
        print(f"{{i+1}}")
        print(f"{{start_time}} --> {{end_time}}")
        print(text)
        print()
except Exception as e:
    print(f"Error: {{e}}", file=sys.stderr)
    sys.exit(1)
"#,
        audio_path
    );

    // 写入临时Python文件
    let script_path = output_dir.join(format!("{}_whisper.py", file_stem));
    std::fs::write(&script_path, python_script)
        .map_err(|e| format!("写入Python脚本失败: {}", e))?;

    update_task_status(task_id, "processing".to_string(), 0.5, None, None);

    // 执行Python脚本
    println!("执行Python Whisper脚本...");
    let output = Command::new("python3")
        .arg(&script_path)
        .output()
        .map_err(|e| format!("执行Python脚本失败: {}", e))?;

    // 清理临时文件
    let _ = std::fs::remove_file(&script_path);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Python Whisper执行失败: {}", stderr));
    }

    update_task_status(task_id, "processing".to_string(), 0.8, None, None);

    // 解析输出的SRT内容
    let srt_content = String::from_utf8_lossy(&output.stdout);
    let subtitles = parse_srt_content(&srt_content)?;

    println!("Python Whisper识别完成，共解析到{}条字幕", subtitles.len());
    Ok(subtitles)
}

/// 解析SRT格式内容
fn parse_srt_content(content: &str) -> Result<Vec<crate::video::Subtitle>, String> {
    let mut subtitles = Vec::new();
    let blocks: Vec<&str> = content.split("\n\n").collect();

    for block in blocks {
        let lines: Vec<&str> = block.trim().split('\n').collect();
        if lines.len() < 3 {
            continue;
        }

        // 解析序号
        let id = lines[0].trim();

        // 解析时间
        let time_line = lines[1];
        if let Some((start_str, end_str)) = time_line.split_once(" --> ") {
            let start_time = parse_srt_time(start_str.trim())?;
            let end_time = parse_srt_time(end_str.trim())?;

            // 解析文本（可能有多行）
            let text = lines[2..].join("\n").trim().to_string();

            if !text.is_empty() {
                subtitles.push(crate::video::Subtitle {
                    id: id.to_string(),
                    start_time,
                    end_time,
                    text,
                });
            }
        }
    }

    if subtitles.is_empty() {
        return Err("未解析到任何字幕内容".to_string());
    }

    Ok(subtitles)
}

/// 解析SRT时间格式 (HH:MM:SS,mmm)
fn parse_srt_time(time_str: &str) -> Result<f64, String> {
    let time_str = time_str.replace(',', "."); // SRT使用逗号作为毫秒分隔符
    let parts: Vec<&str> = time_str.split(':').collect();

    if parts.len() != 3 {
        return Err(format!("无效的时间格式: {}", time_str));
    }

    let hours: f64 = parts[0]
        .parse()
        .map_err(|_| format!("无效的小时: {}", parts[0]))?;
    let minutes: f64 = parts[1]
        .parse()
        .map_err(|_| format!("无效的分钟: {}", parts[1]))?;
    let seconds: f64 = parts[2]
        .parse()
        .map_err(|_| format!("无效的秒数: {}", parts[2]))?;

    Ok(hours * 3600.0 + minutes * 60.0 + seconds)
}

/// 调用百度智能云语音识别API
async fn call_baidu_api(
    audio_path: &str,
    language: &str,
    task_id: &str,
    cancel_rx: &mut mpsc::Receiver<()>,
    api_key: &str,
    secret_key: &str,
) -> Result<Vec<crate::video::Subtitle>, String> {
    use std::fs;

    // 读取音频文件
    let audio_data = fs::read(audio_path).map_err(|e| format!("读取音频文件失败: {}", e))?;

    // 将音频数据转换为base64
    let audio_base64 = base64::engine::general_purpose::STANDARD.encode(&audio_data);

    // 获取访问令牌
    update_task_status(task_id, "processing".to_string(), 0.1, None, None);

    let access_token = get_baidu_access_token(api_key, secret_key)
        .await
        .map_err(|e| format!("获取百度访问令牌失败: {}", e))?;

    // 检查取消信号
    if cancel_rx.try_recv().is_ok() {
        return Err("任务已取消".to_string());
    }

    update_task_status(task_id, "processing".to_string(), 0.3, None, None);

    // 构建请求参数
    let mut params = HashMap::new();
    params.insert("format", "wav".to_string());
    params.insert("rate", "16000".to_string());
    params.insert("channel", "1".to_string());
    params.insert("cuid", "flow-text-app".to_string());
    params.insert("token", access_token);
    params.insert("speech", audio_base64);
    params.insert("len", audio_data.len().to_string());

    // 设置语言
    let dev_pid = match language {
        "zh-CN" => "1537", // 普通话(支持简单的英文识别)
        "en-US" => "1737", // 英语
        _ => "1537",       // 默认普通话
    };
    params.insert("dev_pid", dev_pid.to_string());

    // 发送请求
    update_task_status(task_id, "processing".to_string(), 0.5, None, None);

    let client = reqwest::Client::new();
    let response = client
        .post("https://vop.baidu.com/server_api")
        .header("Content-Type", "application/json")
        .json(&params)
        .send()
        .await
        .map_err(|e| format!("发送请求失败: {}", e))?;

    // 检查取消信号
    if cancel_rx.try_recv().is_ok() {
        return Err("任务已取消".to_string());
    }

    update_task_status(task_id, "processing".to_string(), 0.8, None, None);

    // 解析响应
    let response_text = response
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    let response_json: Value =
        serde_json::from_str(&response_text).map_err(|e| format!("解析响应JSON失败: {}", e))?;

    // 检查错误
    if let Some(err_no) = response_json["err_no"].as_i64() {
        if err_no != 0 {
            let err_msg = response_json["err_msg"].as_str().unwrap_or("未知错误");
            return Err(format!("百度API错误 {}: {}", err_no, err_msg));
        }
    }

    // 提取识别结果
    let result_text = response_json["result"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .unwrap_or("");

    update_task_status(task_id, "processing".to_string(), 0.9, None, None);

    // 将结果转换为字幕格式
    let subtitles = if result_text.is_empty() {
        vec![]
    } else {
        // 简单处理：将整个识别结果作为一个字幕段
        // 实际应用中可能需要更复杂的分段逻辑
        vec![crate::video::Subtitle {
            id: "1".to_string(),
            start_time: 0.0,
            end_time: 10.0, // 默认时长，实际应该根据音频长度计算
            text: result_text.to_string(),
        }]
    };

    Ok(subtitles)
}

/// 获取百度访问令牌
async fn get_baidu_access_token(api_key: &str, secret_key: &str) -> Result<String, String> {
    if api_key.is_empty() || secret_key.is_empty() {
        return Err("请在设置中配置百度API密钥".to_string());
    }

    let client = reqwest::Client::new();
    let url = format!(
        "https://aip.baidubce.com/oauth/2.0/token?grant_type=client_credentials&client_id={}&client_secret={}",
        api_key, secret_key
    );

    let response = client
        .post(&url)
        .send()
        .await
        .map_err(|e| format!("获取访问令牌请求失败: {}", e))?;

    let response_json: Value = response
        .json()
        .await
        .map_err(|e| format!("解析访问令牌响应失败: {}", e))?;

    if let Some(error) = response_json["error"].as_str() {
        return Err(format!("获取访问令牌失败: {}", error));
    }

    let access_token = response_json["access_token"]
        .as_str()
        .ok_or("响应中未找到访问令牌")?;

    Ok(access_token.to_string())
}

/// 调用腾讯云语音识别API
async fn call_tencent_api(
    audio_path: &str,
    _language: &str,
    task_id: &str,
    cancel_rx: &mut mpsc::Receiver<()>,
    secret_id: &str,
    secret_key: &str,
    cos_config: Option<crate::cos::CosConfig>,
) -> Result<Vec<crate::video::Subtitle>, String> {
    // 检查API密钥
    if secret_id.is_empty() || secret_key.is_empty() {
        return Err("腾讯云API密钥未配置".to_string());
    }

    println!("腾讯云API调用开始");
    println!(
        "Secret ID: {}",
        if secret_id.is_empty() {
            "[空]"
        } else {
            "[已配置]"
        }
    );
    println!(
        "Secret Key: {}",
        if secret_key.is_empty() {
            "[空]"
        } else {
            "[已配置]"
        }
    );

    // 更新进度：开始处理
    update_task_status(
        task_id,
        "processing".to_string(),
        0.1,
        None,
        Some("正在读取音频文件...".to_string()),
    );

    // 检查取消信号
    if cancel_rx.try_recv().is_ok() {
        return Err("任务已取消".to_string());
    }

    // 读取音频文件并转换为base64
    let audio_data = match std::fs::read(audio_path) {
        Ok(data) => data,
        Err(e) => {
            return Err(format!("读取音频文件失败: {}", e));
        }
    };

    // 更新进度：文件读取完成
    update_task_status(
        task_id,
        "processing".to_string(),
        0.3,
        None,
        Some("正在调用腾讯云API...".to_string()),
    );

    // 检查取消信号
    if cancel_rx.try_recv().is_ok() {
        return Err("任务已取消".to_string());
    }

    // 调用腾讯云录音文件识别极速版API
    match call_tencent_rapid_asr(
        secret_id,
        secret_key,
        &audio_data,
        task_id,
        cancel_rx,
        cos_config,
    )
    .await
    {
        Ok(result) => {
            println!("腾讯云极速版识别成功，共生成{}条字幕", result.len());
            Ok(result)
        }
        Err(e) => {
            eprintln!("腾讯云极速版API调用失败: {}", e);

            // 如果API调用失败，提供测试数据作为后备
            println!("API调用失败，返回测试数据");
            let mut subtitles = generate_test_data_result(audio_path, "腾讯云极速版");

            // 在测试数据中添加错误信息
            if !subtitles.is_empty() {
                subtitles[0].text = format!(
                    "[极速版API调用失败，显示测试数据]\n错误: {}\n原始文本: {}",
                    e, subtitles[0].text
                );
            }

            Ok(subtitles)
        }
    }
}

/// 生成Whisper安装指导
fn generate_whisper_installation_guide(audio_path: &str) -> Vec<crate::video::Subtitle> {
    use std::path::Path;

    let file_name = Path::new(audio_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    vec![
        crate::video::Subtitle {
            id: "1".to_string(),
            start_time: 0.0,
            end_time: 6.0,
            text: format!("正在处理文件: {} - Whisper未安装", file_name),
        },
        crate::video::Subtitle {
            id: "2".to_string(),
            start_time: 6.0,
            end_time: 12.0,
            text: "要使用真实Whisper识别，请安装: pip install openai-whisper".to_string(),
        },
        crate::video::Subtitle {
            id: "3".to_string(),
            start_time: 12.0,
            end_time: 18.0,
            text: "或者使用Homebrew安装: brew install whisper".to_string(),
        },
        crate::video::Subtitle {
            id: "4".to_string(),
            start_time: 18.0,
            end_time: 24.0,
            text: "安装后将能够进行真实的语音识别而不是模拟数据".to_string(),
        },
        crate::video::Subtitle {
            id: "5".to_string(),
            start_time: 24.0,
            end_time: 30.0,
            text: "当前显示的是安装指导信息，不是真实识别结果".to_string(),
        },
    ]
}

/// 生成测试数据结果（明确标示是测试数据）
fn generate_test_data_result(audio_path: &str, engine_name: &str) -> Vec<crate::video::Subtitle> {
    use std::path::Path;

    let file_name = Path::new(audio_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    vec![
        crate::video::Subtitle {
            id: "1".to_string(),
            start_time: 0.0,
            end_time: 5.0,
            text: format!("[测试数据] 使用{}引擎识别文件: {}", engine_name, file_name),
        },
        crate::video::Subtitle {
            id: "2".to_string(),
            start_time: 5.5,
            end_time: 10.0,
            text: format!("[测试数据] {}引擎当前处于测试模式", engine_name),
        },
        crate::video::Subtitle {
            id: "3".to_string(),
            start_time: 10.5,
            end_time: 15.0,
            text: "[测试数据] 请配置真实API密钥以获取真实识别结果".to_string(),
        },
        crate::video::Subtitle {
            id: "4".to_string(),
            start_time: 15.5,
            end_time: 20.0,
            text: "[测试数据] 这些是示例字幕，不是真实识别结果".to_string(),
        },
    ]
}

/// 模拟识别结果（用于演示）
fn simulate_recognition_result(_audio_path: &str) -> Vec<crate::video::Subtitle> {
    // 生成一些模拟的字幕数据
    let mut subtitles = Vec::new();

    // 添加一些示例字幕
    subtitles.push(crate::video::Subtitle {
        id: "1".to_string(),
        start_time: 0.0,
        end_time: 5.0,
        text: "欢迎使用FlowText视频字幕生成工具".to_string(),
    });

    subtitles.push(crate::video::Subtitle {
        id: "2".to_string(),
        start_time: 5.5,
        end_time: 10.0,
        text: "这是一个基于Tauri和Rust开发的应用".to_string(),
    });

    subtitles.push(crate::video::Subtitle {
        id: "3".to_string(),
        start_time: 10.5,
        end_time: 15.0,
        text: "它可以帮助您快速生成视频字幕".to_string(),
    });

    subtitles.push(crate::video::Subtitle {
        id: "4".to_string(),
        start_time: 15.5,
        end_time: 20.0,
        text: "支持多种语言和字幕格式".to_string(),
    });

    subtitles
}

/// 腾讯云API签名算法实现
type HmacSha256 = Hmac<Sha256>;

fn sha256_hex(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hex::encode(hasher.finalize())
}

fn hmac_sha256(key: &[u8], data: &str) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(data.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

/// 调用腾讯云录音文件识别API（支持大文件，异步识别）
async fn call_tencent_rapid_asr(
    secret_id: &str,
    secret_key: &str,
    audio_data: &[u8],
    task_id: &str,
    cancel_rx: &mut mpsc::Receiver<()>,
    cos_config: Option<crate::cos::CosConfig>,
) -> Result<Vec<crate::video::Subtitle>, String> {
    // 更新进度：开始调用录音文件识别API
    update_task_status(
        task_id,
        "processing".to_string(),
        0.3,
        None,
        Some("正在调用腾讯云录音文件识别API...".to_string()),
    );

    // 检查取消信号
    if cancel_rx.try_recv().is_ok() {
        return Err("任务已取消".to_string());
    }

    // 检查音频文件大小和处理方式选择
    const LOCAL_UPLOAD_LIMIT: usize = 5 * 1024 * 1024; // 5MB，腾讯云本地文件限制
    const BASE64_REQUEST_LIMIT: usize = 7 * 1024 * 1024; // 7MB，考虑base64编码后请求体限制

    println!(
        "音频文件大小: {:.1} MB",
        audio_data.len() as f64 / (1024.0 * 1024.0)
    );

    if audio_data.len() > LOCAL_UPLOAD_LIMIT {
        println!("音频文件超过5MB，需要使用URL方式上传");

        // 检查是否配置了COS
        if let Some(cos_cfg) = cos_config {
            println!("检测到COS配置，尝试上传到腾讯云对象存储");

            // 更新进度：开始上传到COS
            update_task_status(
                task_id,
                "processing".to_string(),
                0.4,
                None,
                Some("正在上传音频文件到腾讯云COS...".to_string()),
            );

            // 检查取消信号
            if cancel_rx.try_recv().is_ok() {
                return Err("任务已取消".to_string());
            }

            // 创建COS客户端并上传文件
            let cos_client = crate::cos::CosClient::new(cos_cfg);
            let file_name = format!("audio_{}.wav", chrono::Utc::now().timestamp());

            match cos_client
                .upload_file(audio_data, &file_name, Some("audio/wav"))
                .await
            {
                Ok(file_url) => {
                    println!("文件上传到COS成功: {}", file_url);

                    // 更新进度：COS上传完成，开始识别
                    update_task_status(
                        task_id,
                        "processing".to_string(),
                        0.6,
                        None,
                        Some("COS上传完成，正在调用识别API...".to_string()),
                    );

                    // 使用URL方式调用识别API
                    return call_tencent_rapid_api_with_url(
                        secret_id, secret_key, &file_url, task_id, cancel_rx,
                    )
                    .await;
                }
                Err(e) => {
                    return Err(format!("上传文件到COS失败: {}", e));
                }
            }
        } else {
            // 没有配置COS，提供详细说明
            return Err(format!(
                "音频文件较大 ({:.1} MB)，超过了直接上传的限制。\n\n\
                📋 **腾讯云录音文件识别限制说明**：\n\
                • 本地文件上传：≤ 5MB\n\
                • URL方式上传：≤ 1GB（需要先上传到COS）\n\
                • 请求体大小：≤ 10MB\n\n\
                💡 **推荐解决方案**：\n\n\
                1. 🏠 **使用Whisper本地识别**（推荐）\n\
                   • ✅ 支持任意大小音频文件\n\
                   • ✅ 无网络限制，识别速度快\n\
                   • ✅ 完全本地处理，隐私安全\n\
                   • ✅ 支持多种语言和模型\n\n\
                2. ☁️ **配置腾讯云COS存储**\n\
                   • 在设置中配置COS存储桶信息\n\
                   • 上传音频到腾讯云对象存储\n\
                   • 通过URL方式调用识别API\n\
                   • 支持最大1GB文件\n\n\
                3. 🎬 **视频分段处理**\n\
                   • 将长视频分成5分钟以内的片段\n\
                   • 分别识别后合并结果\n\
                   • 适合超长视频内容\n\n\
                当前文件: {:.1} MB，请配置COS或使用Whisper本地识别。",
                audio_data.len() as f64 / (1024.0 * 1024.0),
                audio_data.len() as f64 / (1024.0 * 1024.0)
            ));
        }
    }

    println!(
        "腾讯云录音文件识别API调用开始，音频大小: {} bytes",
        audio_data.len()
    );

    // 直接尝试上传（如果文件过大，API会返回相应错误）
    println!("使用腾讯云录音文件识别API（CreateRecTask）");

    // 调用录音文件识别API
    let response = call_tencent_rapid_api(secret_id, secret_key, audio_data).await?;

    // 解析任务创建响应，获取TaskId
    let task_response: Value =
        serde_json::from_str(&response).map_err(|e| format!("解析任务创建响应失败: {}", e))?;

    // 检查是否有错误
    if let Some(error) = task_response.get("Response").and_then(|r| r.get("Error")) {
        let error_code = error
            .get("Code")
            .and_then(|c| c.as_str())
            .unwrap_or("Unknown");
        let error_message = error
            .get("Message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");

        // 针对不同错误提供具体解决方案
        let detailed_error = match error_code {
            "RequestSizeLimitExceeded" => {
                format!(
                    "🚫 **请求体过大错误**\n\n\
                    错误详情: {}\n\n\
                    📋 **问题分析**：\n\
                    • 音频文件: {:.1} MB\n\
                    • Base64编码后: {:.1} MB\n\
                    • 腾讯云请求体限制: 10 MB\n\n\
                    💡 **解决方案**：\n\n\
                    1. 🏠 **使用Whisper本地识别**（强烈推荐）\n\
                       • ✅ 无文件大小限制\n\
                       • ✅ 识别准确度高\n\
                       • ✅ 完全本地处理\n\
                       • ✅ 一次性解决所有大文件问题\n\n\
                    2. ☁️ **配置腾讯云COS上传**\n\
                       • 先上传音频到腾讯云对象存储\n\
                       • 使用URL方式调用API\n\
                       • 支持最大1GB文件\n\n\
                    3. 🔧 **压缩音频文件**\n\
                       • 降低采样率到16kHz或8kHz\n\
                       • 转换为单声道\n\
                       • 使用更高压缩比的格式",
                    error_message,
                    audio_data.len() as f64 / (1024.0 * 1024.0),
                    (audio_data.len() as f64 * 1.37) / (1024.0 * 1024.0) // base64编码约增加37%
                )
            }
            "AudioTooLarge" => {
                format!(
                    "🚫 **音频文件过大**\n\n\
                    错误详情: {}\n\n\
                    💡 **解决方案**：\n\
                    1. 使用Whisper本地识别（推荐）\n\
                    2. 配置腾讯云COS存储上传\n\
                    3. 分割音频文件到5MB以下",
                    error_message
                )
            }
            _ => {
                format!(
                    "腾讯云录音文件识别API错误: {} - {}",
                    error_code, error_message
                )
            }
        };

        return Err(detailed_error);
    }

    let task_id_value = task_response
        .get("Response")
        .and_then(|r| r.get("Data"))
        .and_then(|d| d.get("TaskId"))
        .ok_or("无法获取TaskId")?;

    let recognition_task_id = task_id_value.as_u64().ok_or("TaskId格式错误")?;

    println!("录音文件识别任务已创建，TaskId: {}", recognition_task_id);

    // 更新进度：开始轮询结果
    update_task_status(
        task_id,
        "processing".to_string(),
        0.5,
        None,
        Some("正在等待识别完成...".to_string()),
    );

    // 轮询获取识别结果
    let subtitles = poll_tencent_recognition_result(
        secret_id,
        secret_key,
        recognition_task_id,
        task_id,
        cancel_rx,
    )
    .await?;

    println!("腾讯云录音文件识别完成，共生成{}条字幕", subtitles.len());
    Ok(subtitles)
}

/// 调用腾讯云录音文件识别API（支持大文件）
async fn call_tencent_rapid_api(
    secret_id: &str,
    secret_key: &str,
    audio_data: &[u8],
) -> Result<String, String> {
    let host = "asr.tencentcloudapi.com";
    let service = "asr";
    let version = "2019-06-14";
    let action = "CreateRecTask"; // 使用录音文件识别（支持大文件）
    let region = "ap-beijing";
    let algorithm = "TC3-HMAC-SHA256";

    // 获取当前时间戳
    let timestamp = Utc::now().timestamp();
    let date = Utc::now().format("%Y-%m-%d").to_string();

    // 将音频数据转换为base64
    let audio_base64 = base64::engine::general_purpose::STANDARD.encode(audio_data);

    // 构建请求参数（录音文件识别参数）
    let params = json!({
        "EngineModelType": "16k_zh",
        "ChannelNum": 1,
        "ResTextFormat": 0,
        "SourceType": 1,
        "Data": audio_base64,
        "DataLen": audio_data.len(),
        "WordInfo": 1,  // 返回词级别时间戳
        "FilterDirty": 0,
        "FilterModal": 0,
        "ConvertNumMode": 1,
        "SpeakerDiarization": 0,
        "SpeakerNumber": 0,
        "FilterPunc": 0,
        "OutputFileType": "txt"
    });

    let payload = params.to_string();

    // 构建签名
    let authorization = build_tencent_authorization(
        secret_id, secret_key, &payload, host, &action, &date, timestamp, service, &algorithm,
    )?;

    // 发送HTTP请求
    let client = reqwest::Client::new();
    let url = format!("https://{}", host);

    println!("调用腾讯云录音文件识别API: {}", action);
    println!("音频数据长度: {} bytes", audio_data.len());
    println!("请求负载大小: {} bytes", payload.len());

    let response = client
        .post(&url)
        .header("Authorization", authorization)
        .header("Content-Type", "application/json; charset=utf-8")
        .header("Host", host)
        .header("X-TC-Action", action)
        .header("X-TC-Timestamp", timestamp.to_string())
        .header("X-TC-Version", version)
        .header("X-TC-Region", region)
        .body(payload)
        .send()
        .await
        .map_err(|e| format!("HTTP请求失败: {}", e))?;

    let status = response.status();
    let response_text = response
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    println!("录音文件识别API响应状态: {}", status);
    println!("录音文件识别API响应内容: {}", response_text);

    if status.is_success() {
        Ok(response_text)
    } else {
        Err(format!(
            "录音文件识别API调用失败，状态码: {}, 响应: {}",
            status, response_text
        ))
    }
}

/// 查询腾讯云录音文件识别任务状态
async fn describe_tencent_task_status(
    secret_id: &str,
    secret_key: &str,
    task_id: &str,
) -> Result<String, String> {
    let host = "asr.tencentcloudapi.com";
    let service = "asr";
    let version = "2019-06-14";
    let action = "DescribeTaskStatus";
    let region = "ap-beijing";
    let algorithm = "TC3-HMAC-SHA256";

    // 获取当前时间戳
    let timestamp = Utc::now().timestamp();
    let date = Utc::now().format("%Y-%m-%d").to_string();

    // 构建请求参数
    let params = json!({
        "TaskId": task_id.parse::<u64>().map_err(|e| format!("TaskId格式错误: {}", e))?
    });

    let payload = params.to_string();

    // 构建签名
    let authorization = build_tencent_authorization(
        secret_id, secret_key, &payload, host, action, &date, timestamp, service, &algorithm,
    )?;

    // 发送HTTP请求
    let client = reqwest::Client::new();
    let url = format!("https://{}", host);

    let response = client
        .post(&url)
        .header("Authorization", authorization)
        .header("Content-Type", "application/json; charset=utf-8")
        .header("Host", host)
        .header("X-TC-Action", action)
        .header("X-TC-Timestamp", timestamp.to_string())
        .header("X-TC-Version", version)
        .header("X-TC-Region", region)
        .body(payload)
        .send()
        .await
        .map_err(|e| format!("HTTP请求失败: {}", e))?;

    let status = response.status();
    let response_text = response
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    if status.is_success() {
        Ok(response_text)
    } else {
        Err(format!(
            "查询任务状态失败，状态码: {}, 响应: {}",
            status, response_text
        ))
    }
}

/// 构建腾讯云API签名
fn build_tencent_authorization(
    secret_id: &str,
    secret_key: &str,
    payload: &str,
    host: &str,
    action: &str,
    date: &str,
    timestamp: i64,
    service: &str,
    algorithm: &str,
) -> Result<String, String> {
    // 步骤1：拼接规范请求串
    let http_request_method = "POST";
    let canonical_uri = "/";
    let canonical_query_string = "";
    let canonical_headers = format!(
        "content-type:application/json; charset=utf-8\nhost:{}\n",
        host
    );
    let signed_headers = "content-type;host";
    let hashed_request_payload = sha256_hex(payload);

    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        http_request_method,
        canonical_uri,
        canonical_query_string,
        canonical_headers,
        signed_headers,
        hashed_request_payload
    );

    // 步骤2：拼接待签名字符串
    let credential_scope = format!("{}/{}/tc3_request", date, service);
    let hashed_canonical_request = sha256_hex(&canonical_request);
    let string_to_sign = format!(
        "{}\n{}\n{}\n{}",
        algorithm, timestamp, credential_scope, hashed_canonical_request
    );

    // 步骤3：计算签名
    let secret_date = hmac_sha256(format!("TC3{}", secret_key).as_bytes(), date);
    let secret_service = hmac_sha256(&secret_date, service);
    let secret_signing = hmac_sha256(&secret_service, "tc3_request");
    let signature_bytes = hmac_sha256(&secret_signing, &string_to_sign);
    let signature = hex::encode(signature_bytes);

    // 步骤4：拼接Authorization
    let authorization = format!(
        "{} Credential={}/{}, SignedHeaders={}, Signature={}",
        algorithm, secret_id, credential_scope, signed_headers, signature
    );

    Ok(authorization)
}

/// 分片处理大音频文件
async fn process_large_audio_in_chunks(
    secret_id: &str,
    secret_key: &str,
    audio_data: &[u8],
    task_id: &str,
    cancel_rx: &mut mpsc::Receiver<()>,
) -> Result<Vec<crate::video::Subtitle>, String> {
    const CHUNK_SIZE: usize = 5 * 1024 * 1024; // 5MB per chunk
    const CHUNK_DURATION: f64 = 300.0; // 假设每个chunk约5分钟

    let total_chunks = (audio_data.len() + CHUNK_SIZE - 1) / CHUNK_SIZE;
    let mut all_subtitles = Vec::new();
    let mut current_time_offset = 0.0;

    println!("开始分片处理，总共 {} 个片段", total_chunks);

    for (chunk_index, chunk_data) in audio_data.chunks(CHUNK_SIZE).enumerate() {
        // 检查取消信号
        if cancel_rx.try_recv().is_ok() {
            return Err("任务已取消".to_string());
        }

        println!(
            "处理第 {}/{} 个片段，大小: {} bytes",
            chunk_index + 1,
            total_chunks,
            chunk_data.len()
        );

        // 更新进度
        let progress = 0.3 + (chunk_index as f32 / total_chunks as f32) * 0.6;
        update_task_status(
            task_id,
            "processing".to_string(),
            progress,
            None,
            Some(format!("处理片段 {}/{}", chunk_index + 1, total_chunks)),
        );

        // 处理当前片段
        match call_tencent_rapid_api(secret_id, secret_key, chunk_data).await {
            Ok(response) => {
                // 解析任务创建响应
                let task_response: Value = serde_json::from_str(&response)
                    .map_err(|e| format!("解析片段{}任务创建响应失败: {}", chunk_index + 1, e))?;

                // 检查错误
                if let Some(error) = task_response.get("Response").and_then(|r| r.get("Error")) {
                    let error_code = error
                        .get("Code")
                        .and_then(|c| c.as_str())
                        .unwrap_or("Unknown");
                    let error_message = error
                        .get("Message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown error");
                    println!(
                        "片段{}识别失败: {} - {}",
                        chunk_index + 1,
                        error_code,
                        error_message
                    );
                    continue; // 跳过失败的片段，继续处理下一个
                }

                let task_id_value = task_response
                    .get("Response")
                    .and_then(|r| r.get("Data"))
                    .and_then(|d| d.get("TaskId"))
                    .ok_or(format!("无法获取片段{}的TaskId", chunk_index + 1))?;

                let recognition_task_id = task_id_value
                    .as_u64()
                    .ok_or(format!("片段{}的TaskId格式错误", chunk_index + 1))?;

                // 轮询获取片段结果
                match poll_tencent_recognition_result(
                    secret_id,
                    secret_key,
                    recognition_task_id,
                    task_id,
                    cancel_rx,
                )
                .await
                {
                    Ok(mut chunk_subtitles) => {
                        // 调整时间戳
                        for subtitle in &mut chunk_subtitles {
                            subtitle.start_time += current_time_offset;
                            subtitle.end_time += current_time_offset;
                        }
                        all_subtitles.extend(chunk_subtitles);
                    }
                    Err(e) => {
                        println!("片段{}识别失败: {}", chunk_index + 1, e);
                        // 继续处理下一个片段
                    }
                }
            }
            Err(e) => {
                println!("片段{}API调用失败: {}", chunk_index + 1, e);
                // 继续处理下一个片段
            }
        }

        // 更新时间偏移
        current_time_offset += CHUNK_DURATION;

        // 短暂延迟，避免API调用过于频繁
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    // 重新编号字幕
    for (index, subtitle) in all_subtitles.iter_mut().enumerate() {
        subtitle.id = (index + 1).to_string();
    }

    if all_subtitles.is_empty() {
        Err("所有片段识别都失败了，请检查网络连接和API配置".to_string())
    } else {
        println!("分片处理完成，共生成{}条字幕", all_subtitles.len());
        Ok(all_subtitles)
    }
}

/// 轮询腾讯云录音文件识别结果
async fn poll_tencent_recognition_result(
    secret_id: &str,
    secret_key: &str,
    recognition_task_id: u64,
    task_id: &str,
    cancel_rx: &mut mpsc::Receiver<()>,
) -> Result<Vec<crate::video::Subtitle>, String> {
    let host = "asr.tencentcloudapi.com";
    let service = "asr";
    let version = "2019-06-14";
    let action = "DescribeTaskStatus";
    let region = "ap-beijing";
    let algorithm = "TC3-HMAC-SHA256";

    let client = reqwest::Client::new();
    let url = format!("https://{}", host);

    let mut attempts = 0;
    const MAX_ATTEMPTS: u32 = 60; // 最多等待5分钟（每5秒一次）

    loop {
        // 检查取消信号
        if cancel_rx.try_recv().is_ok() {
            return Err("任务已取消".to_string());
        }

        attempts += 1;
        if attempts > MAX_ATTEMPTS {
            return Err("识别超时，请稍后重试".to_string());
        }

        // 获取当前时间戳
        let timestamp = Utc::now().timestamp();
        let date = Utc::now().format("%Y-%m-%d").to_string();

        // 构建查询参数
        let params = json!({
            "TaskId": recognition_task_id
        });

        let payload = params.to_string();

        // 构建签名
        let authorization = build_tencent_authorization(
            secret_id, secret_key, &payload, host, &action, &date, timestamp, service, &algorithm,
        )?;

        // 发送查询请求
        let response = client
            .post(&url)
            .header("Authorization", authorization)
            .header("Content-Type", "application/json; charset=utf-8")
            .header("Host", host)
            .header("X-TC-Action", action)
            .header("X-TC-Timestamp", timestamp.to_string())
            .header("X-TC-Version", version)
            .header("X-TC-Region", region)
            .body(payload)
            .send()
            .await
            .map_err(|e| format!("查询识别状态失败: {}", e))?;

        let response_text = response
            .text()
            .await
            .map_err(|e| format!("读取查询响应失败: {}", e))?;

        println!("查询识别状态响应: {}", response_text);

        // 解析响应
        let status_response: Value =
            serde_json::from_str(&response_text).map_err(|e| format!("解析状态响应失败: {}", e))?;

        // 检查错误
        if let Some(error) = status_response.get("Response").and_then(|r| r.get("Error")) {
            let error_code = error
                .get("Code")
                .and_then(|c| c.as_str())
                .unwrap_or("Unknown");
            let error_message = error
                .get("Message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(format!(
                "查询识别状态错误: {} - {}",
                error_code, error_message
            ));
        }

        // 获取任务状态
        let data = status_response
            .get("Response")
            .and_then(|r| r.get("Data"))
            .ok_or("无法获取状态数据")?;

        let status = data
            .get("StatusStr")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown");

        println!("识别任务状态: {}", status);

        match status {
            "success" => {
                // 识别成功，解析结果
                let result = data
                    .get("Result")
                    .and_then(|r| r.as_str())
                    .ok_or("无法获取识别结果")?;

                return parse_tencent_file_recognition_result(result);
            }
            "failed" => {
                let error_msg = data
                    .get("ErrorMsg")
                    .and_then(|e| e.as_str())
                    .unwrap_or("识别失败");
                return Err(format!("录音文件识别失败: {}", error_msg));
            }
            "running" | "waiting" => {
                // 任务还在进行中，更新进度
                let progress = 0.5 + (attempts as f32 / MAX_ATTEMPTS as f32) * 0.3;
                update_task_status(
                    task_id,
                    "processing".to_string(),
                    progress,
                    None,
                    Some(format!("识别进行中... ({}/{})", attempts, MAX_ATTEMPTS)),
                );

                // 等待5秒后重试
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
            _ => {
                // 未知状态，继续等待
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
        }
    }
}

/// 生成音频处理建议
fn generate_audio_processing_suggestions(audio_size: usize, audio_path: &str) -> String {
    let size_mb = audio_size as f64 / (1024.0 * 1024.0);
    let estimated_duration = audio_size as f64 / (16000.0 * 2.0 * 60.0); // 估算时长（分钟）

    format!(
        "音频文件太大 ({:.1} MB，约 {:.1} 分钟)，超过腾讯云SentenceRecognition API限制。\n\n\
        💡 建议解决方案：\n\n\
        1. 📏 使用更短的音频片段\n\
           • SentenceRecognition适合60秒以内的短音频\n\
           • 可以手动剪切音频或使用视频编辑软件\n\n\
        2. 🔧 降低音频质量\n\
           • 重新提取音频时使用更低的采样率（如8kHz）\n\
           • 使用更低的比特率\n\n\
        3. 🏠 使用Whisper本地识别\n\
           • 支持任意长度的音频文件\n\
           • 无需网络连接，隐私更好\n\n\
        4. ☁️ 考虑其他腾讯云服务\n\
           • 录音文件识别（非极速版）支持更大文件\n\
           • 实时语音识别（适合流式处理）\n\n\
        当前文件: {}\n\
        建议：先尝试使用Whisper本地识别，或将音频分段处理。",
        size_mb, estimated_duration, audio_path
    )
}

/// 解析腾讯云录音文件识别结果
fn parse_tencent_file_recognition_result(
    result: &str,
) -> Result<Vec<crate::video::Subtitle>, String> {
    // 录音文件识别返回的是JSON格式的详细结果
    let result_data: Value =
        serde_json::from_str(result).map_err(|e| format!("解析识别结果JSON失败: {}", e))?;

    let mut subtitles = Vec::new();

    // 获取句子级别的结果
    if let Some(sentences) = result_data.get("Result").and_then(|r| r.as_array()) {
        for (index, sentence) in sentences.iter().enumerate() {
            if let Some(text) = sentence.get("Text").and_then(|t| t.as_str()) {
                let start_time = sentence
                    .get("StartTime")
                    .and_then(|t| t.as_u64())
                    .unwrap_or(0) as f64
                    / 1000.0; // 转换为秒

                let end_time = sentence
                    .get("EndTime")
                    .and_then(|t| t.as_u64())
                    .unwrap_or(0) as f64
                    / 1000.0; // 转换为秒

                // 如果没有时间信息，使用索引估算
                let (start, end) = if start_time == 0.0 && end_time == 0.0 {
                    let estimated_start = index as f64 * 3.0; // 假设每句3秒
                    let estimated_end = estimated_start + 3.0;
                    (estimated_start, estimated_end)
                } else {
                    (start_time, end_time)
                };

                subtitles.push(crate::video::Subtitle {
                    id: (index + 1).to_string(),
                    start_time: start,
                    end_time: end,
                    text: text.trim().to_string(),
                });
            }
        }
    } else {
        // 如果没有句子级别的结果，尝试解析整体文本
        if let Some(text) = result_data.get("Result").and_then(|r| r.as_str()) {
            // 简单分句处理
            let sentences: Vec<&str> = text
                .split(&['。', '！', '？', '.', '!', '?'])
                .filter(|s| !s.trim().is_empty())
                .collect();

            for (index, sentence) in sentences.iter().enumerate() {
                let start_time = index as f64 * 3.0; // 假设每句3秒
                let end_time = start_time + 3.0;

                subtitles.push(crate::video::Subtitle {
                    id: (index + 1).to_string(),
                    start_time,
                    end_time,
                    text: sentence.trim().to_string(),
                });
            }
        }
    }

    if subtitles.is_empty() {
        return Err("未能解析出有效的识别结果".to_string());
    }

    Ok(subtitles)
}

/// 解析腾讯云SentenceRecognition API响应结果（保留用于兼容性）
fn parse_tencent_rapid_result(response: &str) -> Result<Vec<crate::video::Subtitle>, String> {
    let response_json: Value =
        serde_json::from_str(response).map_err(|e| format!("解析JSON响应失败: {}", e))?;

    // 检查是否有错误（云API 3.0格式）
    if let Some(error) = response_json.get("Response").and_then(|r| r.get("Error")) {
        let error_code = error
            .get("Code")
            .and_then(|c| c.as_str())
            .unwrap_or("Unknown");
        let error_message = error
            .get("Message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");
        return Err(format!(
            "腾讯云SentenceRecognition API错误: {} - {}",
            error_code, error_message
        ));
    }

    // 提取识别结果（SentenceRecognition格式）
    let response_data = response_json
        .get("Response")
        .ok_or("响应中没有Response字段")?;

    // SentenceRecognition返回的是单个结果，不是数组
    let result = response_data
        .get("Result")
        .and_then(|r| r.as_str())
        .unwrap_or("")
        .to_string();

    if result.trim().is_empty() {
        return Err("识别结果为空".to_string());
    }

    // 由于SentenceRecognition是一句话识别，没有时间戳信息
    // 我们创建一个覆盖整个音频的字幕
    let subtitles = vec![crate::video::Subtitle {
        id: "tencent_sentence_1".to_string(),
        start_time: 0.0,
        end_time: 10.0, // 默认10秒，实际应该根据音频长度计算
        text: result,
    }];

    println!(
        "腾讯云SentenceRecognition识别完成，结果: {}",
        subtitles[0].text
    );
    Ok(subtitles)
}

/// 解析任务状态
fn parse_task_status(response: &str) -> Result<String, String> {
    let response_json: Value =
        serde_json::from_str(response).map_err(|e| format!("解析JSON响应失败: {}", e))?;

    // 检查是否有错误
    if let Some(error) = response_json.get("Response").and_then(|r| r.get("Error")) {
        let error_code = error
            .get("Code")
            .and_then(|c| c.as_str())
            .unwrap_or("Unknown");
        let error_message = error
            .get("Message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");
        return Err(format!("腾讯云API错误: {} - {}", error_code, error_message));
    }

    // 提取任务状态
    let status = response_json
        .get("Response")
        .and_then(|r| r.get("Data"))
        .and_then(|d| d.get("StatusStr"))
        .and_then(|s| s.as_str())
        .unwrap_or("unknown");

    Ok(status.to_string())
}

/// 提取错误信息
fn extract_error_message(response: &str) -> String {
    let response_json: Value = match serde_json::from_str(response) {
        Ok(json) => json,
        Err(_) => return "解析响应失败".to_string(),
    };

    response_json
        .get("Response")
        .and_then(|r| r.get("Data"))
        .and_then(|d| d.get("ErrorMsg"))
        .and_then(|e| e.as_str())
        .unwrap_or("未知错误")
        .to_string()
}

/// 解析腾讯云录音文件识别结果
fn parse_tencent_rec_result(response: &str) -> Result<Vec<crate::video::Subtitle>, String> {
    let response_json: Value =
        serde_json::from_str(response).map_err(|e| format!("解析JSON响应失败: {}", e))?;

    // 检查是否有错误
    if let Some(error) = response_json.get("Response").and_then(|r| r.get("Error")) {
        let error_code = error
            .get("Code")
            .and_then(|c| c.as_str())
            .unwrap_or("Unknown");
        let error_message = error
            .get("Message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");
        return Err(format!("腾讯云API错误: {} - {}", error_code, error_message));
    }

    // 提取识别结果
    let data = response_json
        .get("Response")
        .and_then(|r| r.get("Data"))
        .ok_or("响应中没有Data字段")?;

    let result_detail = data
        .get("ResultDetail")
        .and_then(|rd| rd.as_array())
        .ok_or("响应中没有ResultDetail字段")?;

    let mut subtitles = Vec::new();

    for (index, item) in result_detail.iter().enumerate() {
        let start_time = item.get("StartMs").and_then(|s| s.as_u64()).unwrap_or(0) as f64 / 1000.0;

        let end_time = item.get("EndMs").and_then(|e| e.as_u64()).unwrap_or(0) as f64 / 1000.0;

        let text = item
            .get("FinalSentence")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();

        if !text.trim().is_empty() {
            subtitles.push(crate::video::Subtitle {
                id: format!("tencent_{}", index + 1),
                start_time,
                end_time,
                text,
            });
        }
    }

    if subtitles.is_empty() {
        return Err("识别结果为空".to_string());
    }

    println!("腾讯云识别完成，共生成{}条字幕", subtitles.len());
    Ok(subtitles)
}

/// 通过URL调用腾讯云录音文件识别API
async fn call_tencent_rapid_api_with_url(
    secret_id: &str,
    secret_key: &str,
    audio_url: &str,
    task_id: &str,
    cancel_rx: &mut mpsc::Receiver<()>,
) -> Result<Vec<crate::video::Subtitle>, String> {
    use std::collections::HashMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    println!("使用URL方式调用腾讯云录音文件识别API");
    println!("音频URL: {}", audio_url);

    // 更新进度
    update_task_status(
        task_id,
        "processing".to_string(),
        0.7,
        None,
        Some("正在创建识别任务...".to_string()),
    );

    // 检查取消信号
    if cancel_rx.try_recv().is_ok() {
        return Err("任务已取消".to_string());
    }

    // 构建请求参数
    let mut params = HashMap::new();
    params.insert("Action".to_string(), "CreateRecTask".to_string());
    params.insert("Version".to_string(), "2019-06-14".to_string());
    params.insert("Region".to_string(), "ap-beijing".to_string());
    params.insert("EngineModelType".to_string(), "16k_zh".to_string());
    params.insert("ChannelNum".to_string(), "1".to_string());
    params.insert("ResTextFormat".to_string(), "0".to_string());
    params.insert("SourceType".to_string(), "0".to_string()); // 0表示URL
    params.insert("Url".to_string(), audio_url.to_string());

    // 生成签名
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let authorization = generate_tencent_signature_v3(
        secret_id,
        secret_key,
        "asr",
        "ap-beijing",
        &params,
        timestamp,
    )?;

    // 发送请求
    let client = reqwest::Client::new();
    let response = client
        .post("https://asr.tencentcloudapi.com/")
        .header("Authorization", authorization)
        .header("Content-Type", "application/json; charset=utf-8")
        .header("Host", "asr.tencentcloudapi.com")
        .header("X-TC-Action", "CreateRecTask")
        .header("X-TC-Timestamp", timestamp.to_string())
        .header("X-TC-Version", "2019-06-14")
        .header("X-TC-Region", "ap-beijing")
        .json(&params)
        .send()
        .await
        .map_err(|e| format!("发送请求失败: {}", e))?;

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    println!("腾讯云CreateRecTask响应: {}", response_text);

    // 解析响应
    let task_response: Value =
        serde_json::from_str(&response_text).map_err(|e| format!("解析任务创建响应失败: {}", e))?;

    // 检查是否有错误
    if let Some(error) = task_response.get("Response").and_then(|r| r.get("Error")) {
        let error_code = error
            .get("Code")
            .and_then(|c| c.as_str())
            .unwrap_or("Unknown");
        let error_message = error
            .get("Message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");
        return Err(format!(
            "腾讯云CreateRecTask API错误: {} - {}",
            error_code, error_message
        ));
    }

    // 获取TaskId
    let tencent_task_id = task_response
        .get("Response")
        .and_then(|r| r.get("Data"))
        .and_then(|d| d.get("TaskId"))
        .and_then(|t| t.as_u64())
        .ok_or("无法获取TaskId")?;

    println!("腾讯云任务创建成功，TaskId: {}", tencent_task_id);

    // 轮询任务状态
    let mut attempts = 0;
    const MAX_ATTEMPTS: u32 = 60; // 最多等待10分钟（每10秒查询一次）

    loop {
        // 检查取消信号
        if cancel_rx.try_recv().is_ok() {
            return Err("任务已取消".to_string());
        }

        attempts += 1;
        if attempts > MAX_ATTEMPTS {
            return Err("识别任务超时".to_string());
        }

        // 更新进度
        let progress = 0.7 + (attempts as f32 / MAX_ATTEMPTS as f32) * 0.3;
        update_task_status(
            task_id,
            "processing".to_string(),
            progress,
            None,
            Some(format!(
                "正在等待识别完成... ({}/{})",
                attempts, MAX_ATTEMPTS
            )),
        );

        // 等待10秒
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        // 查询任务状态
        match query_tencent_task_status(secret_id, secret_key, tencent_task_id).await {
            Ok(result) => {
                if let Some(subtitles) = result {
                    println!("腾讯云识别完成，共生成{}条字幕", subtitles.len());
                    return Ok(subtitles);
                }
                // 任务还在进行中，继续等待
            }
            Err(e) => {
                eprintln!("查询任务状态失败: {}", e);
                // 继续尝试，不立即返回错误
            }
        }
    }
}

/// 查询腾讯云任务状态
async fn query_tencent_task_status(
    secret_id: &str,
    secret_key: &str,
    tencent_task_id: u64,
) -> Result<Option<Vec<crate::video::Subtitle>>, String> {
    use std::collections::HashMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    // 构建请求参数
    let mut params = HashMap::new();
    params.insert("Action".to_string(), "DescribeTaskStatus".to_string());
    params.insert("Version".to_string(), "2018-05-22".to_string());
    params.insert("Region".to_string(), "ap-beijing".to_string());
    params.insert("TaskId".to_string(), tencent_task_id.to_string());

    // 生成签名
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let authorization = generate_tencent_signature_v3(
        secret_id,
        secret_key,
        "asr",
        "ap-beijing",
        &params,
        timestamp,
    )?;

    // 发送请求
    let client = reqwest::Client::new();
    let response = client
        .post("https://asr.tencentcloudapi.com/")
        .header("Authorization", authorization)
        .header("Content-Type", "application/json; charset=utf-8")
        .header("Host", "asr.tencentcloudapi.com")
        .header("X-TC-Action", "DescribeTaskStatus")
        .header("X-TC-Timestamp", timestamp.to_string())
        .header("X-TC-Version", "2018-05-22")
        .header("X-TC-Region", "ap-beijing")
        .json(&params)
        .send()
        .await
        .map_err(|e| format!("查询任务状态请求失败: {}", e))?;

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("读取任务状态响应失败: {}", e))?;

    println!("腾讯云DescribeTaskStatus响应: {}", response_text);

    // 解析响应
    let status_response: Value =
        serde_json::from_str(&response_text).map_err(|e| format!("解析任务状态响应失败: {}", e))?;

    // 检查是否有错误
    if let Some(error) = status_response.get("Response").and_then(|r| r.get("Error")) {
        let error_code = error
            .get("Code")
            .and_then(|c| c.as_str())
            .unwrap_or("Unknown");
        let error_message = error
            .get("Message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");
        return Err(format!(
            "腾讯云DescribeTaskStatus API错误: {} - {}",
            error_code, error_message
        ));
    }

    // 获取任务状态
    let data = status_response
        .get("Response")
        .and_then(|r| r.get("Data"))
        .ok_or("响应中没有Data字段")?;

    let status = data
        .get("StatusStr")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown");

    println!("腾讯云任务状态: {}", status);

    match status {
        "success" => {
            // 任务完成，解析结果
            let result_detail = data
                .get("ResultDetail")
                .and_then(|rd| rd.as_array())
                .ok_or("响应中没有ResultDetail字段")?;

            let mut subtitles = Vec::new();

            for (index, item) in result_detail.iter().enumerate() {
                let start_time =
                    item.get("StartMs").and_then(|s| s.as_u64()).unwrap_or(0) as f64 / 1000.0;
                let end_time =
                    item.get("EndMs").and_then(|e| e.as_u64()).unwrap_or(0) as f64 / 1000.0;
                let text = item
                    .get("FinalSentence")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();

                if !text.trim().is_empty() {
                    subtitles.push(crate::video::Subtitle {
                        id: format!("tencent_cos_{}", index + 1),
                        start_time,
                        end_time,
                        text,
                    });
                }
            }

            if subtitles.is_empty() {
                return Err("识别结果为空".to_string());
            }

            Ok(Some(subtitles))
        }
        "failed" => {
            let error_msg = data
                .get("ErrorMsg")
                .and_then(|e| e.as_str())
                .unwrap_or("任务失败");
            Err(format!("腾讯云识别任务失败: {}", error_msg))
        }
        "waiting" | "doing" => {
            // 任务还在进行中
            Ok(None)
        }
        _ => Err(format!("未知的任务状态: {}", status)),
    }
}

/// 生成腾讯云API v3签名
fn generate_tencent_signature_v3(
    secret_id: &str,
    secret_key: &str,
    service: &str,
    region: &str,
    params: &std::collections::HashMap<String, String>,
    timestamp: u64,
) -> Result<String, String> {
    // 构建请求体
    let payload =
        serde_json::to_string(params).map_err(|e| format!("序列化请求参数失败: {}", e))?;

    // 构建日期
    let date = chrono::DateTime::from_timestamp(timestamp as i64, 0)
        .ok_or("无效的时间戳")?
        .format("%Y-%m-%d")
        .to_string();

    // 构建主机名
    let host = format!("{}.tencentcloudapi.com", service);

    // 构建规范请求
    let canonical_request = format!(
        "POST\n/\n\ncontent-type:application/json; charset=utf-8\nhost:{}\nx-tc-action:{}\nx-tc-timestamp:{}\nx-tc-version:2018-05-22\n\ncontent-type;host;x-tc-action;x-tc-timestamp;x-tc-version\n{}",
        host,
        params.get("Action").unwrap_or(&"".to_string()),
        timestamp,
        sha256_hash(&payload)
    );

    // 构建待签名字符串
    let credential_scope = format!("{}/{}/tc3_request", date, service);
    let string_to_sign = format!(
        "TC3-HMAC-SHA256\n{}\n{}\n{}",
        timestamp,
        credential_scope,
        sha256_hash(&canonical_request)
    );

    // 计算签名
    let secret_date = hmac_sha256(format!("TC3{}", secret_key).as_bytes(), &date);
    let secret_service = hmac_sha256(&secret_date, service);
    let secret_signing = hmac_sha256(&secret_service, "tc3_request");
    let signature = hex::encode(hmac_sha256(&secret_signing, &string_to_sign));

    // 构建Authorization头
    let authorization = format!(
        "TC3-HMAC-SHA256 Credential={}/{}, SignedHeaders=content-type;host;x-tc-action;x-tc-timestamp;x-tc-version, Signature={}",
        secret_id,
        credential_scope,
        signature
    );

    Ok(authorization)
}

/// 计算SHA256哈希
fn sha256_hash(data: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hex::encode(hasher.finalize())
}
