<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { useVideoStore } from '../stores';
import { VideoPlay, VideoPause, Loading, FullScreen, Mute, Microphone } from '@element-plus/icons-vue';
import { ErrorHandler, ErrorType, ErrorSeverity } from '../utils/errorHandler';

// 引入视频存储
const videoStore = useVideoStore();

// 视频元素引用
const videoRef = ref<HTMLVideoElement | null>(null);

// 视频源URL
const videoSrc = ref('');

// 是否正在播放
const isPlaying = ref(false);

// 视频加载状态
const isLoading = ref(false);

// 音量控制
const volume = ref(1);
const isMuted = ref(false);

// 播放速度
const playbackRate = ref(1);

// 全屏状态
const isFullscreen = ref(false);

// 控制栏显示状态
const showControls = ref(true);
let hideControlsTimer: number | null = null;

// 监听视频变化，更新视频源
watch(() => videoStore.currentVideo, async (newVideo) => {
  if (newVideo) {
    isLoading.value = true;
    await ErrorHandler.withErrorHandling(async () => {
      // 使用Tauri的convertFileSrc将本地文件路径转换为可访问的URL
      const { convertFileSrc } = await import('@tauri-apps/api/core');
      videoSrc.value = convertFileSrc(newVideo.filePath);
    }, {
      context: {
        component: 'VideoPlayer',
        action: 'loadVideo',
        filePath: newVideo.filePath
      },
      onError: (error) => {
        console.error('加载视频失败:', error);
        videoSrc.value = '';
      },
      onFinally: () => {
        isLoading.value = false;
      }
    });
  } else {
    videoSrc.value = '';
  }
}, { immediate: true });

/**
 * 播放/暂停视频
 */
function togglePlay() {
  if (!videoRef.value) return;
  
  if (videoRef.value.paused) {
    videoRef.value.play();
    isPlaying.value = true;
  } else {
    videoRef.value.pause();
    isPlaying.value = false;
  }
}

/**
 * 跳转到指定时间
 * @param time 时间（秒）
 */
function seekTo(time: number) {
  if (!videoRef.value) return;
  
  videoRef.value.currentTime = time;
  videoStore.updateCurrentTime(time);
}

/**
 * 更新当前播放时间
 */
function updateTime() {
  if (!videoRef.value) return;
  
  videoStore.updateCurrentTime(videoRef.value.currentTime);
}

/**
 * 视频播放状态变化处理
 */
function handlePlayStateChange() {
  if (!videoRef.value) return;
  
  isPlaying.value = !videoRef.value.paused;
}

/**
 * 切换静音状态
 */
function toggleMute() {
  if (!videoRef.value) return;
  
  isMuted.value = !isMuted.value;
  videoRef.value.muted = isMuted.value;
}

/**
 * 设置音量
 * @param newVolume 音量值 (0-1)
 */
function setVolume(newVolume: number) {
  if (!videoRef.value) return;
  
  volume.value = newVolume;
  videoRef.value.volume = newVolume;
  
  // 如果音量大于0，取消静音
  if (newVolume > 0 && isMuted.value) {
    isMuted.value = false;
    videoRef.value.muted = false;
  }
}

/**
 * 设置播放速度
 * @param rate 播放速度
 */
function setPlaybackRate(rate: number) {
  if (!videoRef.value) return;
  
  playbackRate.value = rate;
  videoRef.value.playbackRate = rate;
}

/**
 * 切换全屏
 */
function toggleFullscreen() {
  const container = document.querySelector('.video-container') as HTMLElement;
  if (!container) return;
  
  if (!isFullscreen.value) {
    if (container.requestFullscreen) {
      container.requestFullscreen();
    }
  } else {
    if (document.exitFullscreen) {
      document.exitFullscreen();
    }
  }
}

/**
 * 处理全屏状态变化
 */
function handleFullscreenChange() {
  isFullscreen.value = !!document.fullscreenElement;
}

/**
 * 显示控制栏
 */
function showControlsBar() {
  showControls.value = true;
  
  // 清除之前的定时器
  if (hideControlsTimer) {
    clearTimeout(hideControlsTimer);
  }
  
  // 3秒后隐藏控制栏（仅在播放时）
  if (isPlaying.value) {
    hideControlsTimer = window.setTimeout(() => {
      showControls.value = false;
    }, 3000);
  }
}

/**
 * 处理鼠标移动
 */
function handleMouseMove() {
  showControlsBar();
}

/**
 * 处理键盘快捷键
 * @param event 键盘事件
 */
function handleKeydown(event: KeyboardEvent) {
  if (!videoRef.value) return;
  
  switch (event.code) {
    case 'Space':
      event.preventDefault();
      togglePlay();
      break;
    case 'ArrowLeft':
      event.preventDefault();
      seekTo(Math.max(0, videoRef.value.currentTime - 5));
      break;
    case 'ArrowRight':
      event.preventDefault();
      seekTo(Math.min(videoRef.value.duration, videoRef.value.currentTime + 5));
      break;
    case 'ArrowUp':
      event.preventDefault();
      setVolume(Math.min(1, volume.value + 0.1));
      break;
    case 'ArrowDown':
      event.preventDefault();
      setVolume(Math.max(0, volume.value - 0.1));
      break;
    case 'KeyM':
      event.preventDefault();
      toggleMute();
      break;
    case 'KeyF':
      event.preventDefault();
      toggleFullscreen();
      break;
  }
}

// 定时更新播放时间
let timeUpdateInterval: number | null = null;

onMounted(() => {
  // 每100ms更新一次播放时间，比原生timeupdate事件更平滑
  timeUpdateInterval = window.setInterval(updateTime, 100);
  
  // 添加全屏状态监听
  document.addEventListener('fullscreenchange', handleFullscreenChange);
  
  // 添加键盘事件监听
  document.addEventListener('keydown', handleKeydown);
  
  // 初始化视频属性
  if (videoRef.value) {
    videoRef.value.volume = volume.value;
    videoRef.value.playbackRate = playbackRate.value;
  }
});

onUnmounted(() => {
  if (timeUpdateInterval !== null) {
    clearInterval(timeUpdateInterval);
  }
  
  if (hideControlsTimer) {
    clearTimeout(hideControlsTimer);
  }
  
  // 移除事件监听
  document.removeEventListener('fullscreenchange', handleFullscreenChange);
  document.removeEventListener('keydown', handleKeydown);
});
</script>

<template>
  <div class="video-player">
    <div v-if="!videoStore.isVideoLoaded" class="no-video">
      <el-empty description="请先导入视频" />
    </div>
    
    <div v-else class="player-container">
      <div class="video-container" @mousemove="handleMouseMove">
        <video
          ref="videoRef"
          :src="videoSrc"
          @play="handlePlayStateChange"
          @pause="handlePlayStateChange"
          @timeupdate="updateTime"
          @click="togglePlay"
          @loadedmetadata="showControlsBar"
        ></video>
        
        <!-- 字幕显示区域 -->
        <div v-if="videoStore.currentSubtitle" class="subtitle-overlay">
          <div class="subtitle-text">
            {{ videoStore.currentSubtitle.text }}
          </div>
        </div>
        
        <!-- 加载状态 -->
        <div v-if="isLoading" class="loading-overlay">
          <el-icon class="loading-icon"><Loading /></el-icon>
        </div>
        
        <!-- 播放/暂停按钮 -->
        <div class="play-overlay" @click="togglePlay">
          <el-icon v-if="!isPlaying" class="play-icon"><VideoPlay /></el-icon>
        </div>
      </div>
      
      <!-- 控制栏 -->
      <div class="controls" :class="{ 'controls-hidden': !showControls }">
        <!-- 主要控制区域 -->
        <div class="main-controls">
          <!-- 播放/暂停按钮 -->
          <el-button type="primary" size="small" @click="togglePlay">
            <el-icon v-if="isPlaying"><VideoPause /></el-icon>
            <el-icon v-else><VideoPlay /></el-icon>
          </el-button>
          
          <!-- 时间显示 -->
          <div class="time-display">
            {{ videoStore.formattedCurrentTime }} / {{ videoStore.formattedDuration }}
          </div>
          
          <!-- 进度条 -->
          <el-slider
            v-model="videoStore.currentTime"
            :min="0"
            :max="videoStore.currentVideo?.duration || 0"
            :step="0.1"
            @change="seekTo"
            class="time-slider"
          />
        </div>
        
        <!-- 次要控制区域 -->
        <div class="secondary-controls">
          <!-- 音量控制 -->
          <div class="volume-control">
            <el-button size="small" @click="toggleMute">
              <el-icon v-if="isMuted || volume === 0"><Mute /></el-icon>
              <el-icon v-else><Microphone /></el-icon>
            </el-button>
            <el-slider
              v-model="volume"
              :min="0"
              :max="1"
              :step="0.01"
              @change="setVolume"
              class="volume-slider"
            />
          </div>
          
          <!-- 播放速度 -->
          <el-select
            v-model="playbackRate"
            @change="setPlaybackRate"
            size="small"
            class="speed-select"
          >
            <el-option label="0.5x" :value="0.5" />
            <el-option label="0.75x" :value="0.75" />
            <el-option label="1x" :value="1" />
            <el-option label="1.25x" :value="1.25" />
            <el-option label="1.5x" :value="1.5" />
            <el-option label="2x" :value="2" />
          </el-select>
          
          <!-- 全屏按钮 -->
          <el-button size="small" @click="toggleFullscreen">
            <el-icon><FullScreen /></el-icon>
          </el-button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.video-player {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.no-video {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
  min-height: 200px;
}

.player-container {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.video-container {
  position: relative;
  width: 100%;
  flex: 1;
  background-color: #000;
  overflow: hidden;
}

.video-container video {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.subtitle-overlay {
  position: absolute;
  bottom: 60px;
  left: 0;
  right: 0;
  display: flex;
  justify-content: center;
  pointer-events: none;
}

.subtitle-text {
  background: rgba(0, 0, 0, 0.8);
  color: white;
  padding: 12px 20px;

  max-width: 80%;
  text-align: center;
  font-size: 18px;
  font-weight: 500;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.1);

  line-height: 1.4;
}

.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: rgba(0, 0, 0, 0.5);
}

.loading-icon {
  font-size: 48px;
  color: white;
}

.play-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  justify-content: center;
  align-items: center;
  cursor: pointer;
}

.play-icon {
  font-size: 72px;
  color: white;
  opacity: 0.9;
  background: rgba(0, 0, 0, 0.3);

  padding: 20px;
  backdrop-filter: blur(5px);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);

}

.play-icon:hover {
  opacity: 1;
  transform: scale(1.1);
  background: rgba(0, 0, 0, 0.4);

}

.controls {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background: rgba(0, 0, 0, 0.9);
  backdrop-filter: blur(10px);
  padding: 20px 24px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  opacity: 1;
  transform: translateY(0);

}

.controls-hidden {
  opacity: 0;
  transform: translateY(100%);
  pointer-events: none;
}

.main-controls {
  display: flex;
  align-items: center;
  gap: 20px;
  flex: 1;
}

.secondary-controls {
  display: flex;
  align-items: center;
  gap: 12px;
}



.time-display {
  margin: 0 20px;
  font-size: 14px;
  color: white;
  font-size: 14px;
  font-weight: 500;
}

.time-slider {
  flex: 1;
  margin-right: 20px;
}

.volume-control {
  display: flex;
  align-items: center;
  gap: 10px;
  background: rgba(255, 255, 255, 0.1);
  padding: 8px 12px;

  border-radius: 0;
  backdrop-filter: blur(5px);
  transition: all 0.3s ease;
}

.volume-control:hover {
  background: rgba(255, 255, 255, 0.15);
}

.volume-slider {
  width: 90px;
}

.speed-select {
  width: 90px;
}

:deep(.el-slider__runway) {
  margin: 0;
  background-color: rgba(255, 255, 255, 0.2);
  border-radius: 0;
}

/* 扁平化按钮样式 */
:deep(.el-button) {
  background: #0fdc78;
  border: 2px solid #0fdc78;
  color: #000000;
  border-radius: 0;
  padding: 12px 16px;
  font-weight: 600;
}

:deep(.el-button:hover) {
  background: #000000;
  border-color: #000000;
  color: #0fdc78;
}

:deep(.el-button:active) {
  background: #0fdc78;
  color: #000000;
}

/* 扁平化选择器样式 */
:deep(.el-select .el-input__wrapper) {
  background: #0fdc78;
  border: 2px solid #0fdc78;
  color: #000000;
  border-radius: 0;
}

:deep(.el-select .el-input__wrapper:hover) {
  background: #000000;
  border-color: #000000;
  color: #0fdc78;
}

:deep(.el-input__inner) {
  color: inherit;
}

/* 扁平化滑块样式 */
:deep(.el-slider__bar) {
  background: #0fdc78;
  border-radius: 0;
}

:deep(.el-slider__button) {
  border: 2px solid #0fdc78;
  background: #ffffff;
  border-radius: 0;
}

:deep(.el-slider__button:hover) {
  border-color: #000000;
  background: #0fdc78;
}

/* 全屏模式样式 */
.player-container:fullscreen {
  background: black;
  display: flex;
  align-items: center;
  justify-content: center;
}

.player-container:fullscreen .video-container {
  width: 100%;
  height: 100%;
  max-width: none;
  max-height: none;
}

.player-container:fullscreen video {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .controls {
    padding: 10px 15px;
  }
  
  .main-controls {
    gap: 10px;
  }
  
  .secondary-controls {
    gap: 8px;
  }
  
  .volume-control {
    display: none; /* 在小屏幕上隐藏音量控制 */
  }
  
  .speed-select {
    width: 60px;
  }
  
  .time-display {
    font-size: 12px;
  }
}
</style>