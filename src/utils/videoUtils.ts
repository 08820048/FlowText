import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { VideoInfo, Subtitle, SubtitleFormat } from '../types';

/**
 * 检查Tauri API是否可用
 * 在Tauri v2中，我们检查window.__TAURI__对象和相关API
 */
function checkTauriAvailable(): boolean {
  if (typeof window === 'undefined') {
    return false;
  }
  
  // 检查是否在Tauri环境中
  return '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
}

/**
 * 打开文件选择对话框并选择视频文件
 * @returns 选择的视频文件路径，如果用户取消则返回null
 */
export async function selectVideoFile(): Promise<string | null> {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: '视频文件',
        extensions: ['mp4', 'mkv', 'avi', 'mov', 'wmv', 'flv', 'webm']
      }]
    });
    
    if (selected === null || Array.isArray(selected)) {
      return null;
    }
    
    return selected as string;
  } catch (error) {
    console.error('选择视频文件失败:', error);
    throw new Error(`文件选择失败: ${error}`);
  }
}

/**
 * 获取视频文件信息
 * @param filePath 视频文件路径
 * @returns 视频信息对象
 */
export async function getVideoInfo(filePath: string): Promise<VideoInfo> {
  try {
    return await invoke<VideoInfo>('get_video_info', { filePath });
  } catch (error) {
    console.error('获取视频信息失败:', error);
    throw new Error(`获取视频信息失败: ${error}`);
  }
}

/**
 * 从视频中提取音频
 * @param videoPath 视频文件路径
 * @param audioTrackId 音频轨道ID
 * @returns 提取的音频文件路径
 */
export async function extractAudio(videoPath: string, audioTrackId: number): Promise<string> {
  try {
    return await invoke<string>('extract_audio', { videoPath, audioTrackId });
  } catch (error) {
    console.error('提取音频失败:', error);
    throw new Error(`提取音频失败: ${error}`);
  }
}

/**
 * 将秒数转换为时间字符串（HH:MM:SS.mmm）
 * @param seconds 秒数
 * @returns 格式化的时间字符串
 */
export function formatTime(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);
  const milliseconds = Math.floor((seconds % 1) * 1000);
  
  return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}.${milliseconds.toString().padStart(3, '0')}`;
}

/**
 * 将时间字符串（HH:MM:SS.mmm）转换为秒数
 * @param timeString 时间字符串
 * @returns 秒数
 */
export function parseTime(timeString: string): number {
  const parts = timeString.split(':');
  if (parts.length !== 3) {
    throw new Error('无效的时间格式，应为 HH:MM:SS.mmm');
  }
  
  const [hours, minutes, secondsPart] = parts;
  const [seconds, milliseconds = '0'] = secondsPart.split('.');
  
  return (
    parseInt(hours) * 3600 +
    parseInt(minutes) * 60 +
    parseInt(seconds) +
    parseInt(milliseconds) / 1000
  );
}

/**
 * 导出字幕为指定格式
 * @param subtitles 字幕数组
 * @param format 字幕格式
 * @param fileName 文件名（不含扩展名）
 * @returns 导出的文件路径
 */
export async function exportSubtitles(subtitles: Subtitle[], format: SubtitleFormat, fileName: string): Promise<string> {
  try {
    return await invoke<string>('export_subtitles', { subtitles, format, fileName });
  } catch (error) {
    console.error('导出字幕失败:', error);
    throw new Error(`导出字幕失败: ${error}`);
  }
}

/**
 * 导入字幕文件
 * @param filePath 字幕文件路径
 * @returns 字幕数组
 */
export async function importSubtitles(filePath: string): Promise<Subtitle[]> {
  try {
    return await invoke<Subtitle[]>('import_subtitles', { filePath });
  } catch (error) {
    console.error('导入字幕失败:', error);
    throw new Error(`导入字幕失败: ${error}`);
  }
}

/**
 * 生成唯一ID
 * @returns 唯一ID字符串
 */
export function generateId(): string {
  return Date.now().toString(36) + Math.random().toString(36).substring(2);
}