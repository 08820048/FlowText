mod video;
mod recognition;

use video::*;
use recognition::*;
use tauri::Manager;

// 视频处理命令
#[tauri::command]
async fn get_video_info(file_path: String) -> Result<VideoInfo, String> {
    video::get_video_info(&file_path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn extract_audio(video_path: String, audio_track_id: u32) -> Result<String, String> {
    video::extract_audio(&video_path, audio_track_id).map_err(|e| e.to_string())
}

// 字幕处理命令
#[tauri::command]
async fn export_subtitles(subtitles: Vec<Subtitle>, format: String, file_name: String) -> Result<String, String> {
    video::export_subtitles(&subtitles, &format, &file_name).map_err(|e| e.to_string())
}

#[tauri::command]
async fn import_subtitles(file_path: String) -> Result<Vec<Subtitle>, String> {
    video::import_subtitles(&file_path).map_err(|e| e.to_string())
}

// 语音识别命令
#[tauri::command]
async fn start_recognition(task_id: String, audio_path: String, engine: String, language: String, api_keys: Option<serde_json::Value>) -> Result<(), String> {
    recognition::start_recognition(task_id, &audio_path, &engine, &language, api_keys).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_recognition_status(task_id: String) -> Result<RecognitionStatus, String> {
    recognition::get_recognition_status(&task_id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn cancel_recognition(task_id: String) -> Result<(), String> {
    recognition::cancel_recognition(&task_id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_supported_languages(engine: String) -> Result<Vec<Language>, String> {
    recognition::get_supported_languages(&engine).map_err(|e| e.to_string())
}

#[tauri::command]
async fn validate_api_keys(engine: String, api_keys: serde_json::Value) -> Result<bool, String> {
    recognition::validate_api_keys(&engine, api_keys).map_err(|e| e.to_string())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_video_info,
            extract_audio,
            export_subtitles,
            import_subtitles,
            start_recognition,
            get_recognition_status,
            cancel_recognition,
            get_supported_languages,
            validate_api_keys
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
