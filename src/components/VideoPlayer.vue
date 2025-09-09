<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue';
import { useVideoStore } from '../stores';
import { VideoPlay, VideoPause, Loading, Mute, Microphone } from '@element-plus/icons-vue';
import { ErrorHandler, ErrorType, ErrorSeverity } from '../utils/errorHandler';
import { ElMessage } from 'element-plus';

// 获取视频存储
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

// 控制栏显示状态
const showControls = ref(true);
let hideControlsTimer: number | null = null;

// 视频加载错误信息
const videoLoadError = ref('');

// 监听视频变化，设置视频源
watch(() => videoStore.currentVideo, async (newVideo) => {
  console.log('=== 视频变化监听器 ===');
  console.log('newVideo:', newVideo);
  
  if (newVideo) {
    try {
      isLoading.value = true;
      videoLoadError.value = '';
      console.log('开始加载视频:', newVideo.filePath);
      
      // 检查浏览器支持的视频格式
      const videoElement = document.createElement('video');
      const canPlayMp4 = videoElement.canPlayType('video/mp4');
      const canPlayWebm = videoElement.canPlayType('video/webm');
      const canPlayOgg = videoElement.canPlayType('video/ogg');
      
      console.log('浏览器视频格式支持:');
      console.log('- MP4:', canPlayMp4);
      console.log('- WebM:', canPlayWebm);
      console.log('- Ogg:', canPlayOgg);
      
      // 检查是否在Tauri环境中
      if (typeof window !== 'undefined' && ('__TAURI__' in window || '__TAURI_INTERNALS__' in window)) {
        console.log('Tauri环境检测成功');
        try {
          // 使用Tauri的convertFileSrc将本地文件路径转换为可访问的URL
          const { convertFileSrc } = await import('@tauri-apps/api/core');
          const convertedSrc = await convertFileSrc(newVideo.filePath);
          console.log('路径转换结果:');
          console.log('  原始路径:', newVideo.filePath);
          console.log('  转换路径:', convertedSrc);
          videoSrc.value = convertedSrc;
        } catch (tauriError) {
          console.error('Tauri convertFileSrc失败:', tauriError);
          console.log('尝试使用asset协议...');
          // 使用asset协议作为备选方案
          const assetUrl = `asset://localhost/${encodeURIComponent(newVideo.filePath)}`;
          console.log('Asset URL:', assetUrl);
          videoSrc.value = assetUrl;
        }
      } else {
        console.log('非Tauri环境，使用file://协议');
        videoSrc.value = `file://${newVideo.filePath}`;
      }
      
      console.log('设置视频源完成:', videoSrc.value);
      
      // 等待下一个tick确保DOM更新
      await nextTick();
      
      // 检查video元素是否存在
      if (videoRef.value) {
        console.log('video元素存在，开始加载');
        
        // 添加额外的视频属性以提高兼容性
        videoRef.value.crossOrigin = 'anonymous';
        videoRef.value.preload = 'metadata';
        
        // 先尝试加载
        videoRef.value.load();
        console.log('已调用video.load()');
        
        // 设置超时检查
        setTimeout(() => {
          if (videoRef.value && videoRef.value.readyState === 0) {
            console.warn('视频加载超时，尝试重新加载');
            videoLoadError.value = '视频加载超时，可能是格式不兼容';
          }
        }, 5000);
        
      } else {
        console.warn('video元素未找到');
        videoLoadError.value = 'Video元素未找到';
      }
      
    } catch (error) {
      console.error('加载视频失败:', error);
      videoLoadError.value = `加载失败: ${error}`;
      videoSrc.value = '';
    } finally {
      isLoading.value = false;
    }
  } else {
    console.log('清空视频源');
    videoSrc.value = '';
    videoLoadError.value = '';
    if (videoRef.value) {
      videoRef.value.removeAttribute('src');
      videoRef.value.load();
    }
  }
}, { immediate: true });

// 用于跟踪是否是程序内部更新时间（避免循环）
let isInternalTimeUpdate = false;

// 监听store中的currentTime变化，用于字幕跳转功能
watch(() => videoStore.currentTime, (newTime, oldTime) => {
  // 如果是内部更新，忽略
  if (isInternalTimeUpdate) {
    return;
  }

  // 只有当时间差异较大时才认为是跳转（避免正常播放时的小幅更新）
  const timeDiff = Math.abs(newTime - (videoRef.value?.currentTime || 0));
  if (timeDiff < 0.5) {
    return; // 时间差小于0.5秒，认为是正常更新
  }

  console.log('=== 字幕跳转监听器 ===');
  console.log('跳转到时间:', newTime);
  console.log('当前视频时间:', videoRef.value?.currentTime);
  console.log('时间差:', timeDiff);

  if (videoRef.value && typeof newTime === 'number' && newTime >= 0) {
    console.log('执行视频时间跳转:', newTime);
    videoRef.value.currentTime = newTime;

    // 不自动播放，让用户控制播放状态
    console.log('跳转完成，保持当前播放状态');
  }
});

/**
 * 播放/暂停视频
 */
function togglePlay() {
  console.log('=== 播放按钮点击 ===');

  if (!videoRef.value) {
    console.error('视频元素未找到');
    ElMessage.error('视频元素未找到');
    return;
  }
  
  console.log('视频状态详细信息:');
  console.log('- readyState:', videoRef.value.readyState);
  console.log('- networkState:', videoRef.value.networkState);
  console.log('- duration:', videoRef.value.duration);
  console.log('- currentSrc:', videoRef.value.currentSrc);
  console.log('- paused:', videoRef.value.paused);
  console.log('- ended:', videoRef.value.ended);
  console.log('- error:', videoRef.value.error);
  
  // 检查视频是否有错误
  if (videoRef.value.error) {
    console.error('视频有错误:', videoRef.value.error);
    ElMessage.error(`视频错误: ${videoRef.value.error.message}`);
    return;
  }
  
  // 检查视频是否已加载
  if (videoRef.value.readyState === 0) {
    console.warn('视频尚未开始加载');
    ElMessage.warning('视频尚未加载，请等待...');
    videoRef.value.load();
    return;
  }
  
  if (videoRef.value.readyState < 2) {
    console.warn('视频数据不足，无法播放');
    ElMessage.warning('视频数据不足，请等待加载完成...');
    return;
  }
  
  if (videoRef.value.paused || videoRef.value.ended) {
    console.log('尝试播放视频...');

    // 确保音频设置正确
    videoRef.value.volume = volume.value;
    videoRef.value.muted = isMuted.value;

    const playPromise = videoRef.value.play();

    if (playPromise !== undefined) {
      playPromise.then(() => {
        console.log('✓ 视频播放成功');
        console.log('播放后音频状态:', {
          volume: videoRef.value?.volume,
          muted: videoRef.value?.muted
        });
        // 不在这里设置 isPlaying，让事件处理器处理
        ElMessage.success('视频开始播放');
      }).catch((error) => {
        console.error('✗ 视频播放失败:', error);
        ElMessage.error(`播放失败: ${error.message}`);

        // 特殊错误处理
        if (error.name === 'NotAllowedError') {
          ElMessage.warning('需要用户交互才能播放视频');
        } else if (error.name === 'NotSupportedError') {
          ElMessage.error('浏览器不支持该视频格式');
        }
      });
    }
  } else {
    console.log('暂停视频...');
    videoRef.value.pause();
    console.log('✓ 视频已暂停');
    // 不在这里设置 isPlaying，让事件处理器处理
  }
}

// 是否正在拖动进度条
const isDragging = ref(false);

// 调试函数
function debugLoad() {
  console.log('=== 手动重新加载视频 ===');
  if (videoRef.value) {
    videoRef.value.load();
    ElMessage.info('正在重新加载视频...');
  }
}

function seekTo(time: number) {
  if (!videoRef.value) return;
  console.log('跳转到时间:', time);
  videoRef.value.currentTime = time;
  videoStore.updateCurrentTime(time);
}

// 开始拖动进度条
function onSliderStart() {
  isDragging.value = true;
  console.log('开始拖动进度条');
}

// 结束拖动进度条
function onSliderEnd() {
  isDragging.value = false;
  console.log('结束拖动进度条');
}

// 拖动进度条时的实时更新
function onSliderInput(value: number) {
  if (isDragging.value && videoRef.value) {
    videoRef.value.currentTime = value;
    // 不更新store，避免与视频时间更新冲突
  }
}

// 格式化进度条tooltip显示
function formatTooltip(value: number): string {
  const minutes = Math.floor(value / 60);
  const seconds = Math.floor(value % 60);
  return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
}

function updateTime() {
  if (!videoRef.value || isDragging.value) return; // 拖动时不更新

  // 设置标志位，避免触发跳转监听器
  isInternalTimeUpdate = true;
  videoStore.updateCurrentTime(videoRef.value.currentTime);
  // 下一个tick后重置标志位
  nextTick(() => {
    isInternalTimeUpdate = false;
  });
}

function handlePlayStateChange(event?: Event) {
  if (!videoRef.value) return;

  const newPlaying = !videoRef.value.paused && !videoRef.value.ended;
  console.log('播放状态变化:', newPlaying ? '播放' : '暂停');

  isPlaying.value = newPlaying;
}

function handleVolumeChange() {
  if (!videoRef.value) return;
  console.log('音量变化:', {
    volume: videoRef.value.volume,
    muted: videoRef.value.muted
  });
  volume.value = videoRef.value.volume;
  isMuted.value = videoRef.value.muted;
}

function toggleMute() {
  if (!videoRef.value) return;
  console.log('切换静音状态:', !isMuted.value);
  isMuted.value = !isMuted.value;
  videoRef.value.muted = isMuted.value;
  console.log('静音设置完成:', {
    isMuted: isMuted.value,
    videoMuted: videoRef.value.muted,
    volume: videoRef.value.volume
  });
}

function setVolume(newVolume: number) {
  if (!videoRef.value) return;
  console.log('设置音量:', newVolume);
  volume.value = newVolume;
  videoRef.value.volume = newVolume;
  if (newVolume > 0 && isMuted.value) {
    console.log('音量大于0，取消静音');
    isMuted.value = false;
    videoRef.value.muted = false;
  }
  console.log('音量设置完成:', {
    volume: videoRef.value.volume,
    muted: videoRef.value.muted
  });
}

function setPlaybackRate(rate: number) {
  if (!videoRef.value) return;
  playbackRate.value = rate;
  videoRef.value.playbackRate = rate;
}

function showControlsBar() {
  showControls.value = true;
  if (hideControlsTimer) {
    clearTimeout(hideControlsTimer);
  }
  if (isPlaying.value) {
    hideControlsTimer = window.setTimeout(() => {
      showControls.value = false;
    }, 3000);
  }
}

function handleMouseMove() {
  showControlsBar();
}

function handleVideoError(event: Event) {
  console.error('=== 视频错误事件 ===');
  console.error('event:', event);
  isLoading.value = false;
  
  const video = event.target as HTMLVideoElement;
  const error = video.error;
  
  if (error) {
    console.error('错误详情:', {
      code: error.code,
      message: error.message
    });
    
    let errorMsg = '';
    switch (error.code) {
      case MediaError.MEDIA_ERR_ABORTED:
        errorMsg = '视频加载被中止';
        break;
      case MediaError.MEDIA_ERR_NETWORK:
        errorMsg = '网络错误';
        break;
      case MediaError.MEDIA_ERR_DECODE:
        errorMsg = '视频解码错误';
        break;
      case MediaError.MEDIA_ERR_SRC_NOT_SUPPORTED:
        errorMsg = '视频格式不支持';
        break;
      default:
        errorMsg = '未知视频错误';
    }
    
    videoLoadError.value = errorMsg;
    ElMessage.error(errorMsg);
  }
}

function handleCanPlay() {
  console.log('=== 视频可播放 ===');
  isLoading.value = false;
  console.log('readyState:', videoRef.value?.readyState);
  console.log('duration:', videoRef.value?.duration);
  ElMessage.success('视频加载完成，可以播放');
}

function handleLoadedMetadata() {
  console.log('=== 视频元数据加载完成 ===');
  if (videoRef.value) {
    // 确保音频设置正确
    videoRef.value.volume = volume.value;
    videoRef.value.muted = isMuted.value;

    console.log('视频元数据:', {
      duration: videoRef.value.duration,
      videoWidth: videoRef.value.videoWidth,
      videoHeight: videoRef.value.videoHeight,
      volume: videoRef.value.volume,
      muted: videoRef.value.muted
    });

    videoStore.updateCurrentTime(0);
  }
  showControlsBar();
}

function handleLoadedData() {
  console.log('=== 视频数据加载完成 ===');
  isLoading.value = false;
}

function handleLoadStart() {
  console.log('=== 视频开始加载 ===');
  if (videoRef.value) {
    console.log('视频加载开始 - currentSrc:', videoRef.value.currentSrc);
  }
}

function handleProgress() {
  if (videoRef.value && videoRef.value.buffered.length > 0) {
    const buffered = videoRef.value.buffered;
    console.log('缓冲进度:', {
      ranges: buffered.length,
      start: buffered.start(0),
      end: buffered.end(buffered.length - 1),
      duration: videoRef.value.duration
    });
  }
}

let timeUpdateInterval: number | null = null;

onMounted(() => {
  console.log('=== VideoPlayer组件挂载 ===');

  // 恢复定时器，但降低频率避免冲突
  timeUpdateInterval = window.setInterval(updateTime, 250);

  nextTick(() => {
    if (videoRef.value) {
      console.log('初始化视频元素');
      console.log('设置初始音频参数:', {
        volume: volume.value,
        muted: isMuted.value,
        playbackRate: playbackRate.value
      });

      videoRef.value.volume = volume.value;
      videoRef.value.muted = isMuted.value;
      videoRef.value.playbackRate = playbackRate.value;

      // 确保音频轨道可用
      if (videoRef.value.audioTracks && videoRef.value.audioTracks.length > 0) {
        console.log('音频轨道数量:', videoRef.value.audioTracks.length);
        for (let i = 0; i < videoRef.value.audioTracks.length; i++) {
          console.log(`音频轨道 ${i}:`, {
            enabled: videoRef.value.audioTracks[i].enabled,
            kind: videoRef.value.audioTracks[i].kind,
            label: videoRef.value.audioTracks[i].label,
            language: videoRef.value.audioTracks[i].language
          });
        }
      }
    }
  });
});

onUnmounted(() => {
  console.log('=== VideoPlayer组件卸载 ===');
  if (timeUpdateInterval !== null) {
    clearInterval(timeUpdateInterval);
  }
  if (hideControlsTimer) {
    clearTimeout(hideControlsTimer);
  }
});

// 开发环境调试器（仅保留基本功能）
if (import.meta.env.DEV) {
  (window as any).debugVideoPlayer = {
    debugLoad: debugLoad,
    videoRef: videoRef,
    videoSrc: videoSrc,
    videoStore: videoStore
  };
}
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
          preload="metadata"
          :controls="false"
          :muted="isMuted"
          :volume="volume"
          playsinline
          webkit-playsinline
          x-webkit-airplay="deny"
          disablepictureinpicture
          controlslist="nodownload nofullscreen noremoteplayback"
          @play="handlePlayStateChange"
          @pause="handlePlayStateChange"
          @timeupdate="updateTime"
          @loadedmetadata="handleLoadedMetadata"
          @loadeddata="handleLoadedData"
          @canplay="handleCanPlay"
          @error="handleVideoError"
          @loadstart="handleLoadStart"
          @progress="handleProgress"
          @volumechange="handleVolumeChange"
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
          <p class="loading-text">正在加载视频...</p>
        </div>
        
        <!-- 错误状态 -->
        <div v-if="videoLoadError" class="error-overlay">
          <div class="error-text">
            <h3>视频加载失败</h3>
            <p>{{ videoLoadError }}</p>
            <el-button @click="debugLoad" type="primary">重试</el-button>
          </div>
        </div>
        
        <!-- 播放/暂停按钮 -->
        <div class="play-overlay" @click="togglePlay">
          <el-icon v-if="!isPlaying" class="play-icon"><VideoPlay /></el-icon>
        </div>
        
        <!-- 集成的进度条和控制器 -->
        <div class="integrated-controls" :class="{ 'controls-hidden': !showControls }">
          <!-- 进度条 -->
          <el-slider
            v-model="videoStore.currentTime"
            :min="0"
            :max="videoStore.currentVideo?.duration || 0"
            :step="0.1"
            :show-tooltip="true"
            :format-tooltip="formatTooltip"
            @change="seekTo"
            @input="onSliderInput"
            @mousedown="onSliderStart"
            @mouseup="onSliderEnd"
            class="integrated-progress-bar"
          />
          
          <!-- 控制按钮组 -->
          <div class="control-buttons">
            <!-- 播放/暂停按钮 -->
            <el-button type="primary" size="small" @click="togglePlay">
              <el-icon v-if="isPlaying"><VideoPause /></el-icon>
              <el-icon v-else><VideoPlay /></el-icon>
            </el-button>
            
            <!-- 时间显示 -->
            <div class="time-display">
              {{ videoStore.formattedCurrentTime }} / {{ videoStore.formattedDuration }}
            </div>
            
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
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.video-player {
  width: 100%;
  flex: 1;
  display: flex;
  flex-direction: column;
  background-color: #f5f5f5;
  min-height: 0;
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

/* 强制隐藏浏览器原生控制条 */
.video-container video::-webkit-media-controls {
  display: none !important;
}

.video-container video::-webkit-media-controls-enclosure {
  display: none !important;
}

.video-container video::-webkit-media-controls-panel {
  display: none !important;
}

.video-container video::-webkit-media-controls-play-button {
  display: none !important;
}

.video-container video::-webkit-media-controls-timeline {
  display: none !important;
}

.video-container video::-webkit-media-controls-current-time-display {
  display: none !important;
}

.video-container video::-webkit-media-controls-time-remaining-display {
  display: none !important;
}

.video-container video::-webkit-media-controls-mute-button {
  display: none !important;
}

.video-container video::-webkit-media-controls-volume-slider {
  display: none !important;
}

.video-container video::-webkit-media-controls-fullscreen-button {
  display: none !important;
}

/* Firefox 浏览器控制条隐藏 */
.video-container video::-moz-media-controls {
  display: none !important;
}

/* 通用控制条隐藏 */
.video-container video {
  outline: none;
}

.video-container video:focus {
  outline: none;
}

.subtitle-overlay {
  position: absolute;
  bottom: 100px;
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
  border-radius: 4px;
  max-width: 80%;
  text-align: center;
  font-size: 18px;
  font-weight: 500;
}

.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  background-color: rgba(0, 0, 0, 0.7);
}

.loading-icon {
  font-size: 48px;
  color: white;
  margin-bottom: 10px;
}

.loading-text {
  color: white;
  font-size: 16px;
}

.error-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: rgba(0, 0, 0, 0.8);
}

.error-text {
  text-align: center;
  color: white;
  padding: 20px;
}

.error-text h3 {
  color: #ff6b6b;
  margin-bottom: 10px;
}

.play-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 80px;
  display: flex;
  justify-content: center;
  align-items: center;
  cursor: pointer;
  pointer-events: auto;
}

.play-icon {
  font-size: 72px;
  color: #67C23A;
  opacity: 1;
  background: transparent;
  border: 3px solid #67C23A;
  border-radius: 50%;
  padding: 20px;
  transition: all 0.3s ease;
}

.play-icon:hover {
  transform: scale(1.1);
  color: #5daf34;
  border-color: #5daf34;
}

.integrated-controls {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background: linear-gradient(to top, rgba(0, 0, 0, 0.8) 0%, rgba(0, 0, 0, 0.4) 50%, transparent 100%);
  padding: 10px 20px 20px;
  transition: all 0.3s ease;
  opacity: 1;
  z-index: 10;
  pointer-events: auto;
}

.controls-hidden {
  opacity: 0.3;
}

.integrated-progress-bar {
  margin-bottom: 15px;
}

.integrated-progress-bar :deep(.el-slider__runway) {
  background-color: rgba(255, 255, 255, 0.3);
  height: 6px;
  cursor: pointer;
}

.integrated-progress-bar :deep(.el-slider__bar) {
  background-color: #0fdc78;
  height: 6px;
}

.integrated-progress-bar :deep(.el-slider__button) {
  width: 16px;
  height: 16px;
  border: 3px solid #0fdc78;
  background-color: white;
  cursor: grab;
  transition: all 0.2s ease;
}

.integrated-progress-bar :deep(.el-slider__button:hover) {
  transform: scale(1.2);
  box-shadow: 0 0 8px rgba(15, 220, 120, 0.6);
}

.integrated-progress-bar :deep(.el-slider__button:active) {
  cursor: grabbing;
  transform: scale(1.3);
}

/* tooltip 样式优化 */
.integrated-progress-bar :deep(.el-tooltip__trigger) {
  outline: none;
}

.integrated-progress-bar :deep(.el-slider__button-wrapper) {
  outline: none;
}

.control-buttons {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 15px;
}

.time-display {
  margin: 0 10px;
  font-size: 12px;
  color: white;
  font-weight: 500;
  white-space: nowrap;
  min-width: 100px;
  text-align: center;
}

.volume-control {
  display: flex;
  align-items: center;
  gap: 10px;
  background: rgba(255, 255, 255, 0.1);
  padding: 8px 12px;
  border-radius: 4px;
}

.volume-slider {
  width: 90px;
}

.speed-select {
  width: 90px;
}

:deep(.el-button) {
  background: #0fdc78;
  border: 2px solid #0fdc78;
  color: #000000;
  border-radius: 4px;
  padding: 12px 16px;
  font-weight: 600;
}

:deep(.el-button:hover) {
  background: #000000;
  border-color: #000000;
  color: #0fdc78;
}

:deep(.el-select .el-input__wrapper) {
  background: #0fdc78;
  border: 2px solid #0fdc78;
  color: #000000;
  border-radius: 4px;
}

:deep(.el-slider__bar) {
  background: #0fdc78;
}

:deep(.el-slider__button) {
  border: 2px solid #0fdc78;
  background: #ffffff;
}
</style>