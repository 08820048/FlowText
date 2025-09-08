use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

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
    pub state: String, // "pending", "processing", "completed", "failed", "cancelled"
    pub progress: f32, // 0.0 - 1.0
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
            state: "pending".to_string(),
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

        // 根据引擎选择不同的识别方法
        let result = match engine.as_str() {
            "whisper" => {
                // Whisper本地识别
                println!("使用Whisper引擎进行本地识别...");
                println!("音频文件路径: {}", audio_path);

                match call_whisper_api(&audio_path, &language, &task_id_clone, &mut cancel_rx).await
                {
                    Ok(subtitles) => {
                        println!("Whisper识别成功，共生成{}条字幕", subtitles.len());
                        Ok(subtitles)
                    }
                    Err(e) => {
                        eprintln!("Whisper识别失败: {}", e);
                        // 如果Whisper未安装，提供安装指导和测试数据
                        if e.contains("未找到whisper") || e.contains("ModuleNotFoundError") {
                            println!("生成Whisper安装指导的测试数据...");
                            let installation_guide =
                                generate_whisper_installation_guide(&audio_path);
                            Ok(installation_guide)
                        } else {
                            Err(format!("Whisper识别失败: {}", e))
                        }
                    }
                }
            }
            "baidu" => {
                // 从API密钥中提取百度密钥
                let (api_key, secret_key) = if let Some(keys) = &api_keys {
                    let api_key = keys["api_key"].as_str().unwrap_or("");
                    let secret_key = keys["secret_key"].as_str().unwrap_or("");
                    (api_key, secret_key)
                } else {
                    ("", "")
                };

                println!("使用百度引擎进行识别...");
                println!(
                    "API Key: {}",
                    if api_key.is_empty() {
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

                match call_baidu_api(
                    &audio_path,
                    &language,
                    &task_id_clone,
                    &mut cancel_rx,
                    api_key,
                    secret_key,
                )
                .await
                {
                    Ok(subtitles) => {
                        println!("百度API识别成功，共生成{}条字幕", subtitles.len());
                        Ok(subtitles)
                    }
                    Err(e) => {
                        // 记录详细的错误信息
                        eprintln!("百度API调用失败: {}", e);

                        // 检查是否是密钥问题
                        if api_key.is_empty() || secret_key.is_empty() {
                            Err("请在设置中配置百度API密钥".to_string())
                        } else {
                            Err(format!("百度API调用失败: {}", e))
                        }
                    }
                }
            }
            "tencent" => {
                // 腾讯云API调用，当前使用测试数据
                println!("使用腾讯云引擎进行识别... (测试模式)");

                // 模拟进度更新
                for i in 1..=8 {
                    // 检查是否收到取消信号
                    if cancel_rx.try_recv().is_ok() {
                        update_task_status(
                            &task_id_clone,
                            "cancelled".to_string(),
                            i as f32 / 8.0,
                            None,
                            Some("任务已取消".to_string()),
                        );
                        return;
                    }

                    // 更新进度
                    update_task_status(
                        &task_id_clone,
                        "processing".to_string(),
                        i as f32 / 8.0,
                        None,
                        None,
                    );

                    sleep(Duration::from_secs(1)).await;
                }

                let subtitles = generate_test_data_result(&audio_path, "腾讯云");
                println!("腾讯云测试数据生成完成，共{}条字幕", subtitles.len());
                Ok(subtitles)
            }
            "aliyun" => {
                // 阿里云API调用，当前使用测试数据
                println!("使用阿里云引擎进行识别... (测试模式)");

                // 模拟进度更新
                for i in 1..=8 {
                    // 检查是否收到取消信号
                    if cancel_rx.try_recv().is_ok() {
                        update_task_status(
                            &task_id_clone,
                            "cancelled".to_string(),
                            i as f32 / 8.0,
                            None,
                            Some("任务已取消".to_string()),
                        );
                        return;
                    }

                    // 更新进度
                    update_task_status(
                        &task_id_clone,
                        "processing".to_string(),
                        i as f32 / 8.0,
                        None,
                        None,
                    );

                    sleep(Duration::from_secs(1)).await;
                }

                let subtitles = generate_test_data_result(&audio_path, "阿里云");
                println!("阿里云测试数据生成完成，共{}条字幕", subtitles.len());
                Ok(subtitles)
            }
            "google" => {
                // Google API调用，当前使用测试数据
                println!("使用Google引擎进行识别... (测试模式)");

                // 模拟进度更新
                for i in 1..=5 {
                    // 检查是否收到取消信号
                    if cancel_rx.try_recv().is_ok() {
                        update_task_status(
                            &task_id_clone,
                            "cancelled".to_string(),
                            i as f32 / 5.0,
                            None,
                            Some("任务已取消".to_string()),
                        );
                        return;
                    }

                    // 更新进度
                    update_task_status(
                        &task_id_clone,
                        "processing".to_string(),
                        i as f32 / 5.0,
                        None,
                        None,
                    );

                    sleep(Duration::from_secs(2)).await;
                }

                let subtitles = generate_test_data_result(&audio_path, "Google");
                println!("Google测试数据生成完成，共{}条字幕", subtitles.len());
                Ok(subtitles)
            }
            _ => Err(format!("不支持的识别引擎: {}", engine)),
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
            }
            Err(err) => {
                update_task_status(&task_id_clone, "failed".to_string(), 0.0, None, Some(err));
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
    if let Ok(mut tasks) = RECOGNITION_TASKS.lock() {
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = RecognitionStatus {
                state,
                progress,
                result,
                error,
            };
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
    use std::fs;
    use std::process::Command;

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

/// 使用本地whisper命令进行识别
async fn call_local_whisper(
    audio_path: &str,
    language: &str,
    task_id: &str,
    cancel_rx: &mut mpsc::Receiver<()>,
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

    update_task_status(task_id, "processing".to_string(), 0.9, None, None);

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

    update_task_status(task_id, "processing".to_string(), 0.9, None, None);

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
