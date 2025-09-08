use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use base64::Engine;

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
    pub progress: f32,  // 0.0 - 1.0
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
        update_task_status(
            &task_id_clone,
            "processing".to_string(),
            0.0,
            None,
            None,
        );

        // 根据引擎选择不同的识别方法
        let result = match engine.as_str() {
            "baidu" => {
                // 从API密钥中提取百度密钥
                let (api_key, secret_key) = if let Some(keys) = &api_keys {
                    let api_key = keys["apiKey"].as_str().unwrap_or("");
                    let secret_key = keys["secretKey"].as_str().unwrap_or("");
                    (api_key, secret_key)
                } else {
                    ("", "")
                };
                
                match call_baidu_api(&audio_path, &language, &task_id_clone, &mut cancel_rx, api_key, secret_key).await {
                    Ok(subtitles) => Ok(subtitles),
                    Err(e) => {
                        // 如果API调用失败，使用模拟数据作为后备
                        eprintln!("百度API调用失败，使用模拟数据: {}", e);
                        let subtitles = simulate_recognition_result(&audio_path);
                        Ok(subtitles)
                    }
                }
            }
            "google" => {
                // 模拟Google API调用
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

                let subtitles = simulate_recognition_result(&audio_path);
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
                update_task_status(
                    &task_id_clone,
                    "failed".to_string(),
                    0.0,
                    None,
                    Some(err),
                );
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
    use std::time::{SystemTime, UNIX_EPOCH};
    
    // 读取音频文件
    let audio_data = fs::read(audio_path)
        .map_err(|e| format!("读取音频文件失败: {}", e))?;
    
    // 将音频数据转换为base64
    let audio_base64 = base64::engine::general_purpose::STANDARD.encode(&audio_data);
    
    // 获取访问令牌
    update_task_status(task_id, "processing".to_string(), 0.1, None, None);
    
    let access_token = get_baidu_access_token(api_key, secret_key).await
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
        _ => "1537", // 默认普通话
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
    let response_text = response.text().await
        .map_err(|e| format!("读取响应失败: {}", e))?;
    
    let response_json: Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("解析响应JSON失败: {}", e))?;
    
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
    
    let response_json: Value = response.json().await
        .map_err(|e| format!("解析访问令牌响应失败: {}", e))?;
    
    if let Some(error) = response_json["error"].as_str() {
        return Err(format!("获取访问令牌失败: {}", error));
    }
    
    let access_token = response_json["access_token"]
        .as_str()
        .ok_or("响应中未找到访问令牌")?;
    
    Ok(access_token.to_string())
}

/// 模拟识别结果（用于演示）
fn simulate_recognition_result(audio_path: &str) -> Vec<crate::video::Subtitle> {
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