import { invoke } from '@tauri-apps/api/core';
import type { ExtendedRecognitionParams, ModelConfig } from '../types';

/**
 * 模型API工具类
 * 提供与后端模型管理相关的API调用
 */
export class ModelApi {
  /**
   * 获取所有可用的模型列表
   */
  static async getAvailableModels(): Promise<any[]> {
    try {
      const models = await invoke<any[]>('get_available_models');
      return models;
    } catch (error) {
      console.error('获取可用模型失败:', error);
      throw new Error(`获取可用模型失败: ${error}`);
    }
  }

  /**
   * 检查指定模型是否已安装
   */
  static async checkModelInstallation(engine: string): Promise<boolean> {
    try {
      const installed = await invoke<boolean>('check_model_installation', { engine });
      return installed;
    } catch (error) {
      console.error('检查模型安装状态失败:', error);
      return false;
    }
  }

  /**
   * 获取指定模型的详细信息
   */
  static async getModelInfo(engine: string): Promise<any> {
    try {
      const info = await invoke<any>('get_model_info', { engine });
      return info;
    } catch (error) {
      console.error('获取模型信息失败:', error);
      throw new Error(`获取模型信息失败: ${error}`);
    }
  }

  /**
   * 使用扩展配置开始语音识别
   */
  static async startRecognitionWithConfig(
    taskId: string,
    params: ExtendedRecognitionParams
  ): Promise<void> {
    try {
      await invoke('start_recognition_with_config', {
        taskId,
        params
      });
    } catch (error) {
      console.error('启动扩展识别失败:', error);
      throw new Error(`启动扩展识别失败: ${error}`);
    }
  }

  /**
   * 验证模型配置是否有效
   */
  static validateModelConfig(config: ModelConfig): { valid: boolean; errors: string[] } {
    const errors: string[] = [];

    // 检查必需字段
    if (!config.engine) {
      errors.push('引擎名称不能为空');
    }

    if (!config.size) {
      errors.push('模型大小不能为空');
    }

    if (!config.device) {
      errors.push('计算设备不能为空');
    }

    // 检查设备类型
    if (config.device && !['cpu', 'gpu'].includes(config.device)) {
      errors.push('设备类型必须是 cpu 或 gpu');
    }

    // 检查计算类型（针对faster-whisper）
    if (config.engine === 'faster-whisper' && config.compute_type) {
      const validComputeTypes = ['int8', 'int16', 'float16', 'float32'];
      if (!validComputeTypes.includes(config.compute_type)) {
        errors.push('计算类型无效');
      }
    }

    // 检查beam size
    if (config.beam_size !== undefined) {
      if (config.beam_size < 1 || config.beam_size > 10) {
        errors.push('Beam Size 必须在 1-10 之间');
      }
    }

    // 检查temperature
    if (config.temperature !== undefined) {
      if (config.temperature < 0 || config.temperature > 1) {
        errors.push('Temperature 必须在 0-1 之间');
      }
    }

    return {
      valid: errors.length === 0,
      errors
    };
  }

  /**
   * 获取推荐的模型配置
   */
  static getRecommendedConfig(requirements: {
    speed?: 'fast' | 'balanced' | 'accurate';
    memory?: 'low' | 'medium' | 'high';
    features?: string[];
  }): { engine: string; size: string; config: Partial<ModelConfig> } {
    // 如果需要情感识别，推荐SenseVoice
    if (requirements.features?.includes('emotion')) {
      return {
        engine: 'sensevoice',
        size: 'small',
        config: {
          engine: 'sensevoice',
          size: 'small',
          device: 'cpu',
          enable_emotion_recognition: true,
          enable_event_detection: true
        }
      };
    }

    // 如果需要速度，推荐Faster-Whisper
    if (requirements.speed === 'fast') {
      return {
        engine: 'faster-whisper',
        size: 'base',
        config: {
          engine: 'faster-whisper',
          size: 'base',
          device: 'cpu',
          compute_type: 'int8',
          beam_size: 5,
          temperature: 0.0
        }
      };
    }

    // 如果内存限制，推荐小模型
    if (requirements.memory === 'low') {
      return {
        engine: 'faster-whisper',
        size: 'tiny',
        config: {
          engine: 'faster-whisper',
          size: 'tiny',
          device: 'cpu',
          compute_type: 'int8',
          beam_size: 3,
          temperature: 0.0
        }
      };
    }

    // 如果需要精度，推荐SenseVoice大模型
    if (requirements.speed === 'accurate') {
      return {
        engine: 'sensevoice',
        size: 'large',
        config: {
          engine: 'sensevoice',
          size: 'large',
          device: 'cpu',
          enable_emotion_recognition: true,
          enable_event_detection: true
        }
      };
    }

    // 默认推荐平衡配置
    return {
      engine: 'faster-whisper',
      size: 'base',
      config: {
        engine: 'faster-whisper',
        size: 'base',
        device: 'cpu',
        compute_type: 'int8',
        beam_size: 5,
        temperature: 0.0
      }
    };
  }

  /**
   * 格式化模型大小信息
   */
  static formatModelSize(sizeInfo: any): string {
    if (!sizeInfo) return '';
    
    const parts = [];
    if (sizeInfo.fileSize) parts.push(`文件: ${sizeInfo.fileSize}`);
    if (sizeInfo.memoryUsage) parts.push(`内存: ${sizeInfo.memoryUsage}`);
    
    return parts.join(' • ');
  }

  /**
   * 格式化性能信息
   */
  static formatPerformance(performance: any): string {
    if (!performance) return '';
    
    const parts = [];
    if (performance.wer) parts.push(`WER: ${(performance.wer * 100).toFixed(1)}%`);
    if (performance.throughput) parts.push(`${performance.throughput} 词/秒`);
    if (performance.latency) parts.push(`延迟: ${performance.latency}ms`);
    
    return parts.join(' • ');
  }

  /**
   * 检查GPU是否可用
   */
  static async checkGpuAvailability(): Promise<boolean> {
    try {
      // 这里可以调用后端API检查GPU状态
      // 暂时返回false，实际实现需要检查CUDA/OpenCL等
      return false;
    } catch (error) {
      console.error('检查GPU可用性失败:', error);
      return false;
    }
  }

  /**
   * 获取系统资源信息
   */
  static async getSystemResources(): Promise<{
    memory: number;
    cpu: string;
    gpu?: string;
  }> {
    try {
      // 这里可以调用后端API获取系统信息
      // 暂时返回模拟数据
      return {
        memory: 8192, // MB
        cpu: 'Unknown CPU',
        gpu: undefined
      };
    } catch (error) {
      console.error('获取系统资源信息失败:', error);
      return {
        memory: 0,
        cpu: 'Unknown',
        gpu: undefined
      };
    }
  }
}

export default ModelApi;
