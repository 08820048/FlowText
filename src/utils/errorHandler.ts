import { ElMessage, ElNotification } from 'element-plus';
import type { NotificationOptions } from 'element-plus';

/**
 * 错误类型枚举
 */
export enum ErrorType {
  NETWORK = 'network',
  FILE_SYSTEM = 'file_system',
  VALIDATION = 'validation',
  RECOGNITION = 'recognition',
  VIDEO_PROCESSING = 'video_processing',
  UNKNOWN = 'unknown'
}

/**
 * 错误严重程度
 */
export enum ErrorSeverity {
  LOW = 'low',
  MEDIUM = 'medium',
  HIGH = 'high',
  CRITICAL = 'critical'
}

/**
 * 应用错误接口
 */
export interface AppError {
  id: string;
  type: ErrorType;
  severity: ErrorSeverity;
  message: string;
  details?: string;
  timestamp: Date;
  context?: Record<string, any>;
  stack?: string;
}

/**
 * 错误处理器类
 */
export class ErrorHandler {
  private static errors: AppError[] = [];
  private static maxErrors = 100;

  /**
   * 处理错误
   * @param error 错误对象或错误消息
   * @param type 错误类型
   * @param severity 错误严重程度
   * @param context 错误上下文
   * @param showNotification 是否显示通知
   */
  static handle(
    error: Error | string,
    type: ErrorType = ErrorType.UNKNOWN,
    severity: ErrorSeverity = ErrorSeverity.MEDIUM,
    context?: Record<string, any>,
    showNotification: boolean = true
  ): AppError {
    const appError: AppError = {
      id: this.generateId(),
      type,
      severity,
      message: error instanceof Error ? error.message : error,
      details: error instanceof Error ? error.stack : undefined,
      timestamp: new Date(),
      context,
      stack: error instanceof Error ? error.stack : undefined
    };

    // 添加到错误列表
    this.addError(appError);

    // 控制台输出
    console.error(`[${severity.toUpperCase()}] ${type}:`, appError.message, appError);

    // 显示用户通知
    if (showNotification) {
      this.showErrorNotification(appError);
    }

    return appError;
  }

  /**
   * 显示错误通知
   * @param error 错误对象
   */
  private static showErrorNotification(error: AppError) {
    const message = this.getDisplayMessage(error);
    
    switch (error.severity) {
      case ErrorSeverity.LOW:
        ElMessage.info(message);
        break;
      case ErrorSeverity.MEDIUM:
        ElMessage.warning(message);
        break;
      case ErrorSeverity.HIGH:
        ElMessage.error(message);
        break;
      case ErrorSeverity.CRITICAL:
        ElNotification({
          title: '严重错误',
          message,
          type: 'error',
          duration: 0, // 不自动关闭
          showClose: true
        } as NotificationOptions);
        break;
    }
  }

  /**
   * 获取用户友好的错误消息
   * @param error 错误对象
   * @returns 用户友好的错误消息
   */
  private static getDisplayMessage(error: AppError): string {
    const typeMessages: Record<ErrorType, string> = {
      [ErrorType.NETWORK]: '网络连接错误',
      [ErrorType.FILE_SYSTEM]: '文件操作错误',
      [ErrorType.VALIDATION]: '输入验证错误',
      [ErrorType.RECOGNITION]: '语音识别错误',
      [ErrorType.VIDEO_PROCESSING]: '视频处理错误',
      [ErrorType.UNKNOWN]: '未知错误'
    };

    const typePrefix = typeMessages[error.type] || '错误';
    return `${typePrefix}: ${error.message}`;
  }

  /**
   * 添加错误到列表
   * @param error 错误对象
   */
  private static addError(error: AppError) {
    this.errors.unshift(error);
    
    // 限制错误列表大小
    if (this.errors.length > this.maxErrors) {
      this.errors = this.errors.slice(0, this.maxErrors);
    }
  }

  /**
   * 获取所有错误
   * @returns 错误列表
   */
  static getErrors(): AppError[] {
    return [...this.errors];
  }

  /**
   * 清除所有错误
   */
  static clearErrors() {
    this.errors = [];
  }

  /**
   * 获取特定类型的错误
   * @param type 错误类型
   * @returns 错误列表
   */
  static getErrorsByType(type: ErrorType): AppError[] {
    return this.errors.filter(error => error.type === type);
  }

  /**
   * 获取特定严重程度的错误
   * @param severity 错误严重程度
   * @returns 错误列表
   */
  static getErrorsBySeverity(severity: ErrorSeverity): AppError[] {
    return this.errors.filter(error => error.severity === severity);
  }

  /**
   * 生成唯一ID
   * @returns 唯一ID
   */
  private static generateId(): string {
    return `error_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * 网络错误处理
   * @param error 错误对象
   * @param context 错误上下文
   */
  static handleNetworkError(error: Error | string, context?: Record<string, any>) {
    return this.handle(error, ErrorType.NETWORK, ErrorSeverity.HIGH, context);
  }

  /**
   * 文件系统错误处理
   * @param error 错误对象
   * @param context 错误上下文
   */
  static handleFileSystemError(error: Error | string, context?: Record<string, any>) {
    return this.handle(error, ErrorType.FILE_SYSTEM, ErrorSeverity.MEDIUM, context);
  }

  /**
   * 验证错误处理
   * @param error 错误对象
   * @param context 错误上下文
   */
  static handleValidationError(error: Error | string, context?: Record<string, any>) {
    return this.handle(error, ErrorType.VALIDATION, ErrorSeverity.LOW, context);
  }

  /**
   * 识别错误处理
   * @param error 错误对象
   * @param context 错误上下文
   */
  static handleRecognitionError(error: Error | string, context?: Record<string, any>) {
    return this.handle(error, ErrorType.RECOGNITION, ErrorSeverity.HIGH, context);
  }

  /**
   * 视频处理错误处理
   * @param error 错误对象
   * @param context 错误上下文
   */
  static handleVideoProcessingError(error: Error | string, context?: Record<string, any>) {
    return this.handle(error, ErrorType.VIDEO_PROCESSING, ErrorSeverity.HIGH, context);
  }
}

/**
 * 异步函数错误包装器
 * @param fn 异步函数
 * @param errorType 错误类型
 * @param errorSeverity 错误严重程度
 * @param context 错误上下文
 * @returns 包装后的函数
 */
export function withErrorHandling<T extends (...args: any[]) => Promise<any>>(
  fn: T,
  errorType: ErrorType = ErrorType.UNKNOWN,
  errorSeverity: ErrorSeverity = ErrorSeverity.MEDIUM,
  context?: Record<string, any>
): T {
  return (async (...args: any[]) => {
    try {
      return await fn(...args);
    } catch (error) {
      ErrorHandler.handle(error as Error, errorType, errorSeverity, {
        ...context,
        functionName: fn.name,
        arguments: args
      });
      throw error;
    }
  }) as T;
}

/**
 * 同步函数错误包装器
 * @param fn 同步函数
 * @param errorType 错误类型
 * @param errorSeverity 错误严重程度
 * @param context 错误上下文
 * @returns 包装后的函数
 */
export function withSyncErrorHandling<T extends (...args: any[]) => any>(
  fn: T,
  errorType: ErrorType = ErrorType.UNKNOWN,
  errorSeverity: ErrorSeverity = ErrorSeverity.MEDIUM,
  context?: Record<string, any>
): T {
  return ((...args: any[]) => {
    try {
      return fn(...args);
    } catch (error) {
      ErrorHandler.handle(error as Error, errorType, errorSeverity, {
        ...context,
        functionName: fn.name,
        arguments: args
      });
      throw error;
    }
  }) as T;
}