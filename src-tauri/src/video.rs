use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoInfo {
    pub file_path: String,
    pub duration: f64,
    pub width: i32,
    pub height: i32,
    pub audio_tracks: Vec<AudioTrack>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioTrack {
    pub id: u32,
    pub language: Option<String>,
    pub codec: String,
    pub channels: u32,
    pub sample_rate: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Subtitle {
    pub id: String,
    pub start_time: f64,
    pub end_time: f64,
    pub text: String,
}

/// 获取视频文件信息
pub fn get_video_info(file_path: &str) -> Result<VideoInfo, String> {
    use serde_json::Value;
    
    // 使用ffprobe获取视频信息
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_format")
        .arg("-show_streams")
        .arg(file_path)
        .output()
        .map_err(|e| format!("执行ffprobe失败: {}", e))?;

    if !output.status.success() {
        return Err(format!("ffprobe执行失败: {}", String::from_utf8_lossy(&output.stderr)));
    }

    // 解析JSON输出
    let json_str = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("解析ffprobe输出失败: {}", e))?;

    // 获取格式信息
    let format = json["format"].as_object()
        .ok_or("无法获取格式信息")?;
    
    let duration = format["duration"].as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    // 获取流信息
    let streams = json["streams"].as_array()
        .ok_or("无法获取流信息")?;

    let mut width = 0;
    let mut height = 0;
    let mut audio_tracks = Vec::new();

    for (index, stream) in streams.iter().enumerate() {
        let codec_type = stream["codec_type"].as_str().unwrap_or("");
        
        if codec_type == "video" && width == 0 && height == 0 {
            width = stream["width"].as_i64().unwrap_or(0) as i32;
            height = stream["height"].as_i64().unwrap_or(0) as i32;
        } else if codec_type == "audio" {
            let track = AudioTrack {
                id: index as u32,
                language: stream["tags"]["language"].as_str().map(|s| s.to_string()),
                codec: stream["codec_name"].as_str().unwrap_or("unknown").to_string(),
                channels: stream["channels"].as_i64().unwrap_or(2) as u32,
                sample_rate: stream["sample_rate"].as_str()
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(44100),
            };
            audio_tracks.push(track);
        }
    }

    // 如果没有找到音频轨道，添加一个默认的
    if audio_tracks.is_empty() {
        audio_tracks.push(AudioTrack {
            id: 0,
            language: Some("und".to_string()),
            codec: "unknown".to_string(),
            channels: 2,
            sample_rate: 44100,
        });
    }

    Ok(VideoInfo {
        file_path: file_path.to_string(),
        duration,
        width,
        height,
        audio_tracks,
    })
}

/// 从视频中提取音频
pub fn extract_audio(video_path: &str, audio_track_id: u32) -> Result<String, String> {
    // 创建输出文件路径
    let video_path_obj = Path::new(video_path);
    let file_stem = video_path_obj
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "无法获取文件名".to_string())?;

    let output_dir = video_path_obj
        .parent()
        .ok_or_else(|| "无法获取父目录".to_string())?;

    let output_path = output_dir.join(format!("{}_audio_{}.wav", file_stem, audio_track_id));
    let output_path_str = output_path
        .to_str()
        .ok_or_else(|| "输出路径无效".to_string())?;

    // 使用FFmpeg命令行提取音频
    let status = Command::new("ffmpeg")
        .arg("-i")
        .arg(video_path)
        .arg("-map")
        .arg(format!("0:{}", audio_track_id))
        .arg("-acodec")
        .arg("pcm_s16le")
        .arg("-ar")
        .arg("16000")
        .arg("-ac")
        .arg("1")
        .arg("-y")
        .arg(output_path_str)
        .status()
        .map_err(|e| format!("执行FFmpeg命令失败: {}", e))?;

    if !status.success() {
        return Err(format!("FFmpeg命令执行失败，退出码: {:?}", status.code()));
    }

    Ok(output_path_str.to_string())
}

/// 导出字幕到文件
pub fn export_subtitles(subtitles: &[Subtitle], format: &str, file_name: &str) -> Result<String, String> {
    match format.to_lowercase().as_str() {
        "srt" => export_srt(subtitles, file_name),
        "vtt" => export_vtt(subtitles, file_name),
        "ass" => export_ass(subtitles, file_name),
        "txt" => export_txt(subtitles, file_name),
        "json" => export_json(subtitles, file_name),
        _ => Err(format!("不支持的字幕格式: {}", format)),
    }
}

/// 导出SRT格式字幕
fn export_srt(subtitles: &[Subtitle], file_name: &str) -> Result<String, String> {
    let path = format!("{}.srt", file_name);
    let mut file = File::create(&path).map_err(|e| format!("创建文件失败: {}", e))?;

    for (i, subtitle) in subtitles.iter().enumerate() {
        // 转换时间格式 (秒 -> 00:00:00,000)
        let start = format_time_srt(subtitle.start_time);
        let end = format_time_srt(subtitle.end_time);

        // 写入字幕块
        writeln!(file, "{}", i + 1).map_err(|e| format!("写入文件失败: {}", e))?;
        writeln!(file, "{} --> {}", start, end).map_err(|e| format!("写入文件失败: {}", e))?;
        writeln!(file, "{}", subtitle.text).map_err(|e| format!("写入文件失败: {}", e))?;
        writeln!(file).map_err(|e| format!("写入文件失败: {}", e))?;
    }

    Ok(path)
}

/// 导出ASS格式字幕
fn export_ass(subtitles: &[Subtitle], file_name: &str) -> Result<String, String> {
    let path = format!("{}.ass", file_name);
    let mut file = File::create(&path).map_err(|e| format!("创建文件失败: {}", e))?;

    // 写入ASS头部
    writeln!(file, "[Script Info]").map_err(|e| format!("写入文件失败: {}", e))?;
    writeln!(file, "Title: FlowText Generated Subtitles").map_err(|e| format!("写入文件失败: {}", e))?;
    writeln!(file, "ScriptType: v4.00+").map_err(|e| format!("写入文件失败: {}", e))?;
    writeln!(file, "WrapStyle: 0").map_err(|e| format!("写入文件失败: {}", e))?;
    writeln!(file, "ScaledBorderAndShadow: yes").map_err(|e| format!("写入文件失败: {}", e))?;
    writeln!(file, "YCbCr Matrix: TV.601").map_err(|e| format!("写入文件失败: {}", e))?;
    writeln!(file).map_err(|e| format!("写入文件失败: {}", e))?;
    
    writeln!(file, "[V4+ Styles]").map_err(|e| format!("写入文件失败: {}", e))?;
    writeln!(file, "Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding").map_err(|e| format!("写入文件失败: {}", e))?;
    writeln!(file, "Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1").map_err(|e| format!("写入文件失败: {}", e))?;
    writeln!(file).map_err(|e| format!("写入文件失败: {}", e))?;
    
    writeln!(file, "[Events]").map_err(|e| format!("写入文件失败: {}", e))?;
    writeln!(file, "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text").map_err(|e| format!("写入文件失败: {}", e))?;

    for subtitle in subtitles {
        let start = format_time_ass(subtitle.start_time);
        let end = format_time_ass(subtitle.end_time);
        
        writeln!(file, "Dialogue: 0,{},{},Default,,0,0,0,,{}", start, end, subtitle.text)
            .map_err(|e| format!("写入文件失败: {}", e))?;
    }

    Ok(path)
}

/// 导出TXT格式字幕
fn export_txt(subtitles: &[Subtitle], file_name: &str) -> Result<String, String> {
    let path = format!("{}.txt", file_name);
    let mut file = File::create(&path).map_err(|e| format!("创建文件失败: {}", e))?;

    for subtitle in subtitles {
        let start = format_time_srt(subtitle.start_time);
        let end = format_time_srt(subtitle.end_time);
        
        writeln!(file, "[{}] - [{}]", start, end).map_err(|e| format!("写入文件失败: {}", e))?;
        writeln!(file, "{}", subtitle.text).map_err(|e| format!("写入文件失败: {}", e))?;
        writeln!(file).map_err(|e| format!("写入文件失败: {}", e))?;
    }

    Ok(path)
}

/// 导出JSON格式字幕
fn export_json(subtitles: &[Subtitle], file_name: &str) -> Result<String, String> {
    let path = format!("{}.json", file_name);
    let json_data = serde_json::to_string_pretty(subtitles)
        .map_err(|e| format!("序列化JSON失败: {}", e))?;
    
    std::fs::write(&path, json_data)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(path)
}

/// 格式化时间为ASS格式 (H:MM:SS.CC)
fn format_time_ass(seconds: f64) -> String {
    let hours = (seconds / 3600.0) as i32;
    let minutes = ((seconds % 3600.0) / 60.0) as i32;
    let secs = seconds % 60.0;
    let centiseconds = ((secs - secs.floor()) * 100.0) as i32;
    
    format!("{}:{:02}:{:02}.{:02}", hours, minutes, secs as i32, centiseconds)
}

/// 导出WebVTT格式字幕
fn export_vtt(subtitles: &[Subtitle], file_name: &str) -> Result<String, String> {
    let path = format!("{}.vtt", file_name);
    let mut file = File::create(&path).map_err(|e| format!("创建文件失败: {}", e))?;

    // 写入WebVTT头部
    writeln!(file, "WEBVTT").map_err(|e| format!("写入文件失败: {}", e))?;
    writeln!(file).map_err(|e| format!("写入文件失败: {}", e))?;

    for (i, subtitle) in subtitles.iter().enumerate() {
        // 转换时间格式 (秒 -> 00:00:00.000)
        let start = format_time_vtt(subtitle.start_time);
        let end = format_time_vtt(subtitle.end_time);

        // 写入字幕块
        writeln!(file, "{}", i + 1).map_err(|e| format!("写入文件失败: {}", e))?;
        writeln!(file, "{} --> {}", start, end).map_err(|e| format!("写入文件失败: {}", e))?;
        writeln!(file, "{}", subtitle.text).map_err(|e| format!("写入文件失败: {}", e))?;
        writeln!(file).map_err(|e| format!("写入文件失败: {}", e))?;
    }

    Ok(path)
}

/// 导入字幕文件
pub fn import_subtitles(file_path: &str) -> Result<Vec<Subtitle>, String> {
    let path = Path::new(file_path);
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| "无法获取文件扩展名".to_string())?;

    match extension.to_lowercase().as_str() {
        "srt" => import_srt(file_path),
        "vtt" => import_vtt(file_path),
        _ => Err(format!("不支持的字幕格式: {}", extension)),
    }
}

/// 导入SRT格式字幕
fn import_srt(file_path: &str) -> Result<Vec<Subtitle>, String> {
    let file = File::open(file_path).map_err(|e| format!("打开文件失败: {}", e))?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut subtitles = Vec::new();
    let mut current_id = String::new();
    let mut current_times = (0.0, 0.0);
    let mut current_text = String::new();
    let mut state = 0; // 0: 等待ID, 1: 等待时间, 2: 读取文本

    while let Some(Ok(line)) = lines.next() {
        let line = line.trim();

        match state {
            0 => {
                // 跳过空行
                if line.is_empty() {
                    continue;
                }
                // 读取字幕ID
                current_id = line.to_string();
                state = 1;
            }
            1 => {
                // 读取时间行
                if let Some((start, end)) = parse_time_line_srt(line) {
                    current_times = (start, end);
                    state = 2;
                    current_text.clear();
                } else {
                    return Err(format!("无效的时间行: {}", line));
                }
            }
            2 => {
                // 读取文本行
                if line.is_empty() {
                    // 空行表示当前字幕块结束
                    if !current_text.is_empty() {
                        subtitles.push(Subtitle {
                            id: current_id.clone(),
                            start_time: current_times.0,
                            end_time: current_times.1,
                            text: current_text.trim().to_string(),
                        });
                        current_text.clear();
                    }
                    state = 0;
                } else {
                    // 添加文本行
                    if !current_text.is_empty() {
                        current_text.push('\n');
                    }
                    current_text.push_str(line);
                }
            }
            _ => unreachable!(),
        }
    }

    // 处理最后一个字幕块
    if state == 2 && !current_text.is_empty() {
        subtitles.push(Subtitle {
            id: current_id,
            start_time: current_times.0,
            end_time: current_times.1,
            text: current_text.trim().to_string(),
        });
    }

    Ok(subtitles)
}

/// 导入WebVTT格式字幕
fn import_vtt(file_path: &str) -> Result<Vec<Subtitle>, String> {
    let file = File::open(file_path).map_err(|e| format!("打开文件失败: {}", e))?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // 检查WebVTT头部
    if let Some(Ok(first_line)) = lines.next() {
        if !first_line.trim().starts_with("WEBVTT") {
            return Err("无效的WebVTT文件，缺少WEBVTT头部".to_string());
        }
    } else {
        return Err("空的WebVTT文件".to_string());
    }

    let mut subtitles = Vec::new();
    let mut current_id = String::new();
    let mut current_times = (0.0, 0.0);
    let mut current_text = String::new();
    let mut state = 0; // 0: 等待ID, 1: 等待时间, 2: 读取文本

    while let Some(Ok(line)) = lines.next() {
        let line = line.trim();

        match state {
            0 => {
                // 跳过空行和注释
                if line.is_empty() || line.starts_with("NOTE") {
                    continue;
                }
                // 读取字幕ID或时间行
                if line.contains(" --> ") {
                    if let Some((start, end)) = parse_time_line_vtt(line) {
                        current_times = (start, end);
                        current_id = format!("{}", subtitles.len() + 1);
                        state = 2;
                        current_text.clear();
                    } else {
                        return Err(format!("无效的时间行: {}", line));
                    }
                } else {
                    current_id = line.to_string();
                    state = 1;
                }
            }
            1 => {
                // 读取时间行
                if let Some((start, end)) = parse_time_line_vtt(line) {
                    current_times = (start, end);
                    state = 2;
                    current_text.clear();
                } else {
                    return Err(format!("无效的时间行: {}", line));
                }
            }
            2 => {
                // 读取文本行
                if line.is_empty() {
                    // 空行表示当前字幕块结束
                    if !current_text.is_empty() {
                        subtitles.push(Subtitle {
                            id: current_id.clone(),
                            start_time: current_times.0,
                            end_time: current_times.1,
                            text: current_text.trim().to_string(),
                        });
                        current_text.clear();
                    }
                    state = 0;
                } else {
                    // 添加文本行
                    if !current_text.is_empty() {
                        current_text.push('\n');
                    }
                    current_text.push_str(line);
                }
            }
            _ => unreachable!(),
        }
    }

    // 处理最后一个字幕块
    if state == 2 && !current_text.is_empty() {
        subtitles.push(Subtitle {
            id: current_id,
            start_time: current_times.0,
            end_time: current_times.1,
            text: current_text.trim().to_string(),
        });
    }

    Ok(subtitles)
}

/// 解析SRT时间行 (00:00:00,000 --> 00:00:00,000)
fn parse_time_line_srt(line: &str) -> Option<(f64, f64)> {
    let parts: Vec<&str> = line.split(" --> ").collect();
    if parts.len() != 2 {
        return None;
    }

    let start = parse_time_str_srt(parts[0])?;
    let end = parse_time_str_srt(parts[1])?;

    Some((start, end))
}

/// 解析WebVTT时间行 (00:00:00.000 --> 00:00:00.000)
fn parse_time_line_vtt(line: &str) -> Option<(f64, f64)> {
    let parts: Vec<&str> = line.split(" --> ").collect();
    if parts.len() != 2 {
        return None;
    }

    let start_parts: Vec<&str> = parts[0].split(' ').collect();
    let end_parts: Vec<&str> = parts[1].split(' ').collect();

    let start = parse_time_str_vtt(start_parts[0])?;
    let end = parse_time_str_vtt(end_parts[0])?;

    Some((start, end))
}

/// 解析SRT时间字符串 (00:00:00,000)
fn parse_time_str_srt(time_str: &str) -> Option<f64> {
    let parts: Vec<&str> = time_str.trim().split(':').collect();
    if parts.len() != 3 {
        return None;
    }

    let hours: u32 = parts[0].parse().ok()?;
    let minutes: u32 = parts[1].parse().ok()?;

    let sec_parts: Vec<&str> = parts[2].split(',').collect();
    if sec_parts.len() != 2 {
        return None;
    }

    let seconds: u32 = sec_parts[0].parse().ok()?;
    let milliseconds: u32 = sec_parts[1].parse().ok()?;

    let total_seconds = (hours as f64) * 3600.0
        + (minutes as f64) * 60.0
        + (seconds as f64)
        + (milliseconds as f64) / 1000.0;

    Some(total_seconds)
}

/// 解析WebVTT时间字符串 (00:00:00.000)
fn parse_time_str_vtt(time_str: &str) -> Option<f64> {
    let parts: Vec<&str> = time_str.trim().split(':').collect();
    if parts.len() != 3 {
        return None;
    }

    let hours: u32 = parts[0].parse().ok()?;
    let minutes: u32 = parts[1].parse().ok()?;

    let sec_parts: Vec<&str> = parts[2].split('.').collect();
    if sec_parts.len() != 2 {
        return None;
    }

    let seconds: u32 = sec_parts[0].parse().ok()?;
    let milliseconds: u32 = sec_parts[1].parse().ok()?;

    let total_seconds = (hours as f64) * 3600.0
        + (minutes as f64) * 60.0
        + (seconds as f64)
        + (milliseconds as f64) / 1000.0;

    Some(total_seconds)
}

/// 格式化时间为SRT格式 (00:00:00,000)
fn format_time_srt(seconds: f64) -> String {
    let hours = (seconds / 3600.0).floor() as u32;
    let minutes = ((seconds % 3600.0) / 60.0).floor() as u32;
    let secs = (seconds % 60.0).floor() as u32;
    let millis = ((seconds % 1.0) * 1000.0).round() as u32;

    format!("{:02}:{:02}:{:02},{:03}", hours, minutes, secs, millis)
}

/// 格式化时间为WebVTT格式 (00:00:00.000)
fn format_time_vtt(seconds: f64) -> String {
    let hours = (seconds / 3600.0).floor() as u32;
    let minutes = ((seconds % 3600.0) / 60.0).floor() as u32;
    let secs = (seconds % 60.0).floor() as u32;
    let millis = ((seconds % 1.0) * 1000.0).round() as u32;

    format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, secs, millis)
}