import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { VideoInfo, Subtitle, RecognitionTask } from '../types';

/**
 * 视频状态管理存储
 * 用于管理视频文件、字幕和识别任务的状态
 */
export const useVideoStore = defineStore('video', () => {
  // 当前加载的视频信息
  const currentVideo = ref<VideoInfo | null>(null);
  
  // 当前视频的字幕列表
  const subtitles = ref<Subtitle[]>([]);
  
  // 当前播放位置（秒）
  const currentTime = ref(0);
  
  // 识别任务列表
  const recognitionTasks = ref<RecognitionTask[]>([]);
  
  // 当前选中的音频轨道ID
  const selectedAudioTrackId = ref<number | null>(null);
  
  // 计算属性：当前视频是否已加载
  const isVideoLoaded = computed(() => currentVideo.value !== null);
  
  // 计算属性：当前视频的时长（格式化为 HH:MM:SS）
  const formattedDuration = computed(() => {
    if (!currentVideo.value) return '00:00:00';
    
    const duration = currentVideo.value.duration;
    const hours = Math.floor(duration / 3600);
    const minutes = Math.floor((duration % 3600) / 60);
    const seconds = Math.floor(duration % 60);
    
    return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
  });
  
  // 计算属性：当前播放位置（格式化为 HH:MM:SS）
  const formattedCurrentTime = computed(() => {
    const time = currentTime.value;
    const hours = Math.floor(time / 3600);
    const minutes = Math.floor((time % 3600) / 60);
    const seconds = Math.floor(time % 60);
    
    return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
  });
  
  // 计算属性：当前时间点的字幕
  const currentSubtitle = computed(() => {
    return subtitles.value.find(subtitle => 
      currentTime.value >= subtitle.startTime && currentTime.value <= subtitle.endTime
    );
  });
  
  /**
   * 设置当前视频信息
   * @param video 视频信息对象
   */
  function setCurrentVideo(video: VideoInfo | null) {
    currentVideo.value = video;
    // 重置相关状态
    subtitles.value = [];
    currentTime.value = 0;
    selectedAudioTrackId.value = video?.audioTracks[0]?.id || null;
  }
  
  /**
   * 设置字幕列表
   * @param newSubtitles 字幕列表
   */
  function setSubtitles(newSubtitles: Subtitle[]) {
    subtitles.value = newSubtitles;
  }
  
  /**
   * 更新字幕项
   * @param updatedSubtitle 更新后的字幕
   */
  function updateSubtitle(updatedSubtitle: Subtitle) {
    const index = subtitles.value.findIndex(s => s.id === updatedSubtitle.id);
    if (index !== -1) {
      subtitles.value[index] = updatedSubtitle;
    }
  }
  
  /**
   * 添加字幕项
   * @param subtitle 新字幕
   */
  function addSubtitle(subtitle: Subtitle) {
    subtitles.value.push(subtitle);
    // 按开始时间排序
    subtitles.value.sort((a, b) => a.startTime - b.startTime);
  }
  
  /**
   * 删除字幕项
   * @param id 字幕ID
   */
  function removeSubtitle(id: string) {
    subtitles.value = subtitles.value.filter(s => s.id !== id);
  }
  
  /**
   * 更新当前播放时间
   * @param time 播放时间（秒）
   */
  function updateCurrentTime(time: number) {
    currentTime.value = time;
  }
  
  /**
   * 添加识别任务
   * @param task 识别任务
   */
  function addRecognitionTask(task: RecognitionTask) {
    recognitionTasks.value.push(task);
  }
  
  /**
   * 更新识别任务状态
   * @param id 任务ID
   * @param updates 更新的字段
   */
  function updateRecognitionTask(id: string, updates: Partial<RecognitionTask>) {
    const index = recognitionTasks.value.findIndex(task => task.id === id);
    if (index !== -1) {
      recognitionTasks.value[index] = { ...recognitionTasks.value[index], ...updates };
    }
  }
  
  /**
   * 设置选中的音频轨道
   * @param trackId 音频轨道ID
   */
  function setSelectedAudioTrack(trackId: number) {
    selectedAudioTrackId.value = trackId;
  }
  
  return {
    currentVideo,
    subtitles,
    currentTime,
    recognitionTasks,
    selectedAudioTrackId,
    isVideoLoaded,
    formattedDuration,
    formattedCurrentTime,
    currentSubtitle,
    setCurrentVideo,
    setSubtitles,
    updateSubtitle,
    addSubtitle,
    removeSubtitle,
    updateCurrentTime,
    addRecognitionTask,
    updateRecognitionTask,
    setSelectedAudioTrack
  };
});