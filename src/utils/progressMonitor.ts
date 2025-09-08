import { ref, computed, type Ref } from 'vue';
import { ElMessage } from 'element-plus';

/**
 * 进度状态枚举
 */
export enum ProgressStatus {
  IDLE = 'idle',
  PENDING = 'pending',
  RUNNING = 'running',
  COMPLETED = 'completed',
  FAILED = 'failed',
  CANCELLED = 'cancelled'
}

/**
 * 进度任务接口
 */
export interface ProgressTask {
  id: string;
  name: string;
  status: ProgressStatus;
  progress: number; // 0-100
  message?: string;
  error?: string;
  startTime?: Date;
  endTime?: Date;
  estimatedDuration?: number; // 预估时长（毫秒）
  context?: Record<string, any>;
}

/**
 * 进度监控器类
 */
export class ProgressMonitor {
  private static tasks: Map<string, Ref<ProgressTask>> = new Map();
  private static listeners: Map<string, ((task: ProgressTask) => void)[]> = new Map();

  /**
   * 创建新的进度任务
   * @param name 任务名称
   * @param estimatedDuration 预估时长（毫秒）
   * @param context 任务上下文
   * @returns 任务ID
   */
  static createTask(
    name: string,
    estimatedDuration?: number,
    context?: Record<string, any>
  ): string {
    const id = this.generateId();
    const task: ProgressTask = {
      id,
      name,
      status: ProgressStatus.IDLE,
      progress: 0,
      estimatedDuration,
      context
    };

    this.tasks.set(id, ref(task));
    this.listeners.set(id, []);
    
    return id;
  }

  /**
   * 开始任务
   * @param taskId 任务ID
   * @param message 开始消息
   */
  static startTask(taskId: string, message?: string) {
    const taskRef = this.tasks.get(taskId);
    if (taskRef) {
      taskRef.value = {
        ...taskRef.value,
        status: ProgressStatus.RUNNING,
        progress: 0,
        message,
        startTime: new Date()
      };
      this.notifyListeners(taskId, taskRef.value);
    }
  }

  /**
   * 更新任务进度
   * @param taskId 任务ID
   * @param progress 进度值（0-100）
   * @param message 进度消息
   */
  static updateProgress(taskId: string, progress: number, message?: string) {
    const taskRef = this.tasks.get(taskId);
    if (taskRef) {
      taskRef.value = {
        ...taskRef.value,
        progress: Math.max(0, Math.min(100, progress)),
        message
      };
      this.notifyListeners(taskId, taskRef.value);
    }
  }

  /**
   * 完成任务
   * @param taskId 任务ID
   * @param message 完成消息
   */
  static completeTask(taskId: string, message?: string) {
    const taskRef = this.tasks.get(taskId);
    if (taskRef) {
      taskRef.value = {
        ...taskRef.value,
        status: ProgressStatus.COMPLETED,
        progress: 100,
        message,
        endTime: new Date()
      };
      this.notifyListeners(taskId, taskRef.value);
      
      if (message) {
        ElMessage.success(message);
      }
    }
  }

  /**
   * 任务失败
   * @param taskId 任务ID
   * @param error 错误消息
   */
  static failTask(taskId: string, error: string) {
    const taskRef = this.tasks.get(taskId);
    if (taskRef) {
      taskRef.value = {
        ...taskRef.value,
        status: ProgressStatus.FAILED,
        error,
        endTime: new Date()
      };
      this.notifyListeners(taskId, taskRef.value);
      
      ElMessage.error(`任务失败: ${error}`);
    }
  }

  /**
   * 取消任务
   * @param taskId 任务ID
   * @param message 取消消息
   */
  static cancelTask(taskId: string, message?: string) {
    const taskRef = this.tasks.get(taskId);
    if (taskRef) {
      taskRef.value = {
        ...taskRef.value,
        status: ProgressStatus.CANCELLED,
        message,
        endTime: new Date()
      };
      this.notifyListeners(taskId, taskRef.value);
      
      if (message) {
        ElMessage.info(message);
      }
    }
  }

  /**
   * 获取任务
   * @param taskId 任务ID
   * @returns 任务响应式引用
   */
  static getTask(taskId: string): Ref<ProgressTask> | undefined {
    return this.tasks.get(taskId);
  }

  /**
   * 获取所有任务
   * @returns 所有任务的响应式引用
   */
  static getAllTasks(): Ref<ProgressTask[]> {
    return computed(() => {
      return Array.from(this.tasks.values()).map(taskRef => taskRef.value);
    });
  }

  /**
   * 获取运行中的任务
   * @returns 运行中任务的响应式引用
   */
  static getRunningTasks(): Ref<ProgressTask[]> {
    return computed(() => {
      return Array.from(this.tasks.values())
        .map(taskRef => taskRef.value)
        .filter(task => task.status === ProgressStatus.RUNNING);
    });
  }

  /**
   * 删除任务
   * @param taskId 任务ID
   */
  static removeTask(taskId: string) {
    this.tasks.delete(taskId);
    this.listeners.delete(taskId);
  }

  /**
   * 清除所有已完成的任务
   */
  static clearCompletedTasks() {
    for (const [taskId, taskRef] of this.tasks.entries()) {
      if (taskRef.value.status === ProgressStatus.COMPLETED ||
          taskRef.value.status === ProgressStatus.FAILED ||
          taskRef.value.status === ProgressStatus.CANCELLED) {
        this.removeTask(taskId);
      }
    }
  }

  /**
   * 添加任务监听器
   * @param taskId 任务ID
   * @param listener 监听器函数
   */
  static addListener(taskId: string, listener: (task: ProgressTask) => void) {
    const listeners = this.listeners.get(taskId);
    if (listeners) {
      listeners.push(listener);
    }
  }

  /**
   * 移除任务监听器
   * @param taskId 任务ID
   * @param listener 监听器函数
   */
  static removeListener(taskId: string, listener: (task: ProgressTask) => void) {
    const listeners = this.listeners.get(taskId);
    if (listeners) {
      const index = listeners.indexOf(listener);
      if (index > -1) {
        listeners.splice(index, 1);
      }
    }
  }

  /**
   * 通知监听器
   * @param taskId 任务ID
   * @param task 任务对象
   */
  private static notifyListeners(taskId: string, task: ProgressTask) {
    const listeners = this.listeners.get(taskId);
    if (listeners) {
      listeners.forEach(listener => listener(task));
    }
  }

  /**
   * 生成唯一ID
   * @returns 唯一ID
   */
  private static generateId(): string {
    return `task_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * 计算任务剩余时间
   * @param taskId 任务ID
   * @returns 剩余时间（毫秒），如果无法计算则返回null
   */
  static getEstimatedTimeRemaining(taskId: string): number | null {
    const taskRef = this.tasks.get(taskId);
    if (!taskRef || !taskRef.value.startTime || !taskRef.value.estimatedDuration) {
      return null;
    }

    const task = taskRef.value;
    const elapsed = Date.now() - task.startTime!.getTime();
    const progressRatio = task.progress / 100;
    
    if (progressRatio <= 0) {
      return task.estimatedDuration!;
    }

    const estimatedTotal = elapsed / progressRatio;
    return Math.max(0, estimatedTotal - elapsed);
  }

  /**
   * 格式化剩余时间
   * @param taskId 任务ID
   * @returns 格式化的剩余时间字符串
   */
  static getFormattedTimeRemaining(taskId: string): string {
    const remaining = this.getEstimatedTimeRemaining(taskId);
    if (remaining === null) {
      return '未知';
    }

    const seconds = Math.ceil(remaining / 1000);
    if (seconds < 60) {
      return `${seconds}秒`;
    }

    const minutes = Math.ceil(seconds / 60);
    if (minutes < 60) {
      return `${minutes}分钟`;
    }

    const hours = Math.ceil(minutes / 60);
    return `${hours}小时`;
  }
}

/**
 * 进度任务装饰器
 * @param taskName 任务名称
 * @param estimatedDuration 预估时长（毫秒）
 * @returns 装饰器函数
 */
export function withProgress(
  taskName: string,
  estimatedDuration?: number
) {
  return function <T extends (...args: any[]) => Promise<any>>(
    target: any,
    propertyKey: string,
    descriptor: PropertyDescriptor
  ) {
    const originalMethod = descriptor.value;

    descriptor.value = async function (...args: any[]) {
      const taskId = ProgressMonitor.createTask(taskName, estimatedDuration);
      
      try {
        ProgressMonitor.startTask(taskId, `开始${taskName}`);
        const result = await originalMethod.apply(this, args);
        ProgressMonitor.completeTask(taskId, `${taskName}完成`);
        return result;
      } catch (error) {
        ProgressMonitor.failTask(taskId, error instanceof Error ? error.message : String(error));
        throw error;
      }
    };

    return descriptor;
  };
}

/**
 * 创建进度任务的组合式函数
 * @param taskName 任务名称
 * @param estimatedDuration 预估时长（毫秒）
 * @returns 任务控制对象
 */
export function useProgressTask(taskName: string, estimatedDuration?: number) {
  const taskId = ProgressMonitor.createTask(taskName, estimatedDuration);
  const task = ProgressMonitor.getTask(taskId)!;

  const start = (message?: string) => {
    ProgressMonitor.startTask(taskId, message);
  };

  const updateProgress = (progress: number, message?: string) => {
    ProgressMonitor.updateProgress(taskId, progress, message);
  };

  const complete = (message?: string) => {
    ProgressMonitor.completeTask(taskId, message);
  };

  const fail = (error: string) => {
    ProgressMonitor.failTask(taskId, error);
  };

  const cancel = (message?: string) => {
    ProgressMonitor.cancelTask(taskId, message);
  };

  const remove = () => {
    ProgressMonitor.removeTask(taskId);
  };

  const timeRemaining = computed(() => {
    return ProgressMonitor.getFormattedTimeRemaining(taskId);
  });

  return {
    taskId,
    task,
    start,
    updateProgress,
    complete,
    fail,
    cancel,
    remove,
    timeRemaining
  };
}