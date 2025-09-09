import { invoke } from '@tauri-apps/api/core';
import type { RecognitionEngine, RecognitionTask, Subtitle, ExtendedRecognitionParams } from '../types';
import { generateId } from './videoUtils';
import { ModelApi } from './modelApi';

/**
 * 开始语音识别任务
 * @param audioPath 音频文件路径
 * @param engine 识别引擎
 * @param language 语言代码
 * @param videoInfo 视频信息
 * @param apiKeys API密钥信息
 * @returns 识别任务对象
 */
export async function startRecognition(
  audioPath: string,
  engine: RecognitionEngine,
  language: string,
  videoInfo: any,
  apiKeys?: any
): Promise<RecognitionTask> {
  // 创建任务对象
  const task: RecognitionTask = {
    id: generateId(),
    videoInfo,
    status: 'pending',
    progress: 0,
    engine,
    language,
    createdAt: new Date(),
    updatedAt: new Date()
  };
  
  try {
    // 调用后端开始识别
    await invoke('start_recognition', {
      taskId: task.id,
      audioPath,
      engine,
      language,
      apiKeys
    });
    
    return task;
  } catch (error) {
    console.error('开始识别任务失败:', error);
    throw new Error(`开始识别任务失败: ${error}`);
  }
}

/**
 * 使用扩展配置开始语音识别任务
 */
export async function startRecognitionWithConfig(
  params: ExtendedRecognitionParams
): Promise<RecognitionTask> {
  console.log('开始扩展配置识别任务:', {
    engine: params.engine,
    modelConfig: params.model_config
  });

  try {
    // 验证模型配置
    const validation = ModelApi.validateModelConfig(params.model_config);
    if (!validation.valid) {
      throw new Error(`模型配置无效: ${validation.errors.join(', ')}`);
    }

    // 检查模型是否已安装
    const installed = await ModelApi.checkModelInstallation(params.engine);
    if (!installed) {
      console.warn(`模型 ${params.engine} 未安装，将尝试使用回退方案`);
    }

    // 创建任务对象
    const task: RecognitionTask = {
      id: generateId(),
      videoInfo: {} as any, // 这里需要传入实际的视频信息
      status: 'pending',
      progress: 0,
      engine: params.engine as RecognitionEngine,
      language: params.language,
      createdAt: new Date(),
      updatedAt: new Date()
    };

    // 调用后端API
    await invoke('start_recognition_with_config', {
      taskId: task.id,
      params
    });

    console.log('扩展配置识别任务已启动:', task.id);
    return task;
  } catch (error) {
    console.error('启动扩展配置识别任务失败:', error);
    throw new Error(`启动扩展配置识别任务失败: ${error}`);
  }
}

/**
 * 获取识别任务状态
 * @param taskId 任务ID
 * @returns 任务状态和进度
 */
export async function getRecognitionStatus(taskId: string): Promise<{
  status: 'pending' | 'processing' | 'completed' | 'failed';
  progress: number;
  subtitles?: Subtitle[];
  error?: string;
}> {
  try {
    return await invoke('get_recognition_status', { taskId });
  } catch (error) {
    console.error('获取识别状态失败:', error);
    throw new Error(`获取识别状态失败: ${error}`);
  }
}

/**
 * 取消识别任务
 * @param taskId 任务ID
 */
export async function cancelRecognition(taskId: string): Promise<void> {
  try {
    await invoke('cancel_recognition', { taskId });
  } catch (error) {
    console.error('取消识别任务失败:', error);
    throw new Error(`取消识别任务失败: ${error}`);
  }
}

/**
 * 获取支持的语言列表
 * @param engine 识别引擎
 * @returns 支持的语言列表
 */
export async function getSupportedLanguages(engine: RecognitionEngine): Promise<Array<{
  code: string;
  name: string;
}>> {
  try {
    return await invoke('get_supported_languages', { engine });
  } catch (error) {
    console.error('获取支持的语言列表失败:', error);
    throw new Error(`获取支持的语言列表失败: ${error}`);
  }
}

/**
 * 检查API密钥是否有效
 * @param engine 识别引擎
 * @param apiKeys API密钥信息
 * @returns 是否有效
 */
export async function validateApiKeys(engine: RecognitionEngine, apiKeys: any): Promise<boolean> {
  try {
    return await invoke('validate_api_keys', { engine, apiKeys });
  } catch (error) {
    console.error('验证API密钥失败:', error);
    return false;
  }
}