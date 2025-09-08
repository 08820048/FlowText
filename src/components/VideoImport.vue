<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import { Upload } from '@element-plus/icons-vue';
import { useVideoStore } from '../stores';
import { selectVideoFile, getVideoInfo } from '../utils/videoUtils';
import { ProgressMonitor } from '../utils/progressMonitor';
import { ErrorHandler, ErrorType, ErrorSeverity } from '../utils/errorHandler';

// 引入视频存储
const videoStore = useVideoStore();

// 加载状态
const loading = ref(false);

/**
 * 导入视频文件
 */
async function importVideo() {
  await ErrorHandler.withErrorHandling(async () => {
    loading.value = true;
    
    // 打开文件选择对话框
    const filePath = await selectVideoFile();
    if (!filePath) {
      loading.value = false;
      return;
    }
    
    // 创建进度任务
    const progressTaskId = ProgressMonitor.createTask(
      '导入视频文件',
      `正在分析视频文件: ${filePath.split('/').pop()}`,
      10000 // 预估10秒
    );
    
    try {
      ProgressMonitor.updateTask(progressTaskId, {
        progress: 30,
        message: '正在读取视频信息...'
      });
      
      // 获取视频信息
      const videoInfo = await getVideoInfo(filePath);
      
      ProgressMonitor.updateTask(progressTaskId, {
        progress: 80,
        message: '正在处理音频轨道信息...'
      });
      
      // 更新存储中的视频信息
      videoStore.setCurrentVideo(videoInfo);
      
      // 自动选择第一个音频轨道
      if (videoInfo.audioTracks && videoInfo.audioTracks.length > 0) {
        videoStore.setSelectedAudioTrack(videoInfo.audioTracks[0].id);
      }
      
      ProgressMonitor.completeTask(progressTaskId, `视频导入成功: ${videoInfo.fileName}`);
      ElMessage.success('视频导入成功');
    } catch (error) {
      ProgressMonitor.failTask(progressTaskId, `导入失败: ${error}`);
      throw error;
    } finally {
      loading.value = false;
    }
  }, {
    context: {
      component: 'VideoImport',
      action: 'importVideo'
    },
    onError: (error) => {
      loading.value = false;
      ElMessage.error(`视频导入失败: ${error.message}`);
    }
  });
}
</script>

<template>
  <div class="video-import">
    <div v-if="!videoStore.isVideoLoaded" class="import-container">
      <el-empty description="未导入视频">
        <el-button type="primary" :loading="loading" @click="importVideo">
          <el-icon><Upload /></el-icon> 导入视频
        </el-button>
      </el-empty>
    </div>
    
    <div v-else class="video-info">
      <div class="info-header">
        <h3>{{ videoStore.currentVideo?.fileName }}</h3>
        <el-button type="primary" size="small" :loading="loading" @click="importVideo">
          更换视频
        </el-button>
      </div>
      
      <el-descriptions :column="2" border>
        <el-descriptions-item label="时长">
          {{ videoStore.formattedDuration }}
        </el-descriptions-item>
        <el-descriptions-item label="分辨率">
          {{ videoStore.currentVideo?.resolution.width }} × {{ videoStore.currentVideo?.resolution.height }}
        </el-descriptions-item>
        <el-descriptions-item label="帧率">
          {{ videoStore.currentVideo?.frameRate }} fps
        </el-descriptions-item>
        <el-descriptions-item label="编码格式">
          {{ videoStore.currentVideo?.codecInfo }}
        </el-descriptions-item>
        <el-descriptions-item label="音频轨道" :span="2">
          <el-select 
            v-model="videoStore.selectedAudioTrackId" 
            placeholder="选择音频轨道"
            style="width: 100%"
          >
            <el-option 
              v-for="track in videoStore.currentVideo?.audioTracks" 
              :key="track.id"
              :label="`轨道 ${track.id}${track.language ? ` (${track.language})` : ''} - ${track.codecInfo}`"
              :value="track.id"
            />
          </el-select>
        </el-descriptions-item>
      </el-descriptions>
    </div>
  </div>
</template>

<style scoped>
.video-import {
  width: 100%;
  height: 100%;
}

.import-container {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
  min-height: 200px;
}

.video-info {
  padding: 16px;
}

.info-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.info-header h3 {
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>