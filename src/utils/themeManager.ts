/**
 * 主题管理器
 * 负责管理应用的主题切换和样式应用
 */

export type Theme = 'light' | 'dark';

export interface ThemeConfig {
  name: Theme;
  colors: {
    primary: string;
    secondary: string;
    background: string;
    surface: string;
    text: string;
    textSecondary: string;
    border: string;
    shadow: string;
  };
}

// 浅色主题配置
const lightTheme: ThemeConfig = {
  name: 'light',
  colors: {
    primary: '#667eea',
    secondary: '#764ba2',
    background: 'linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%)',
    surface: '#ffffff',
    text: '#1e293b',
    textSecondary: '#64748b',
    border: '#e2e8f0',
    shadow: 'rgba(0, 0, 0, 0.1)'
  }
};

// 深色主题配置
const darkTheme: ThemeConfig = {
  name: 'dark',
  colors: {
    primary: '#818cf8',
    secondary: '#a78bfa',
    background: 'linear-gradient(135deg, #1e293b 0%, #334155 100%)',
    surface: '#334155',
    text: '#f1f5f9',
    textSecondary: '#cbd5e1',
    border: '#475569',
    shadow: 'rgba(0, 0, 0, 0.3)'
  }
};

class ThemeManager {
  private currentTheme: Theme = 'light';
  private themes: Record<Theme, ThemeConfig> = {
    light: lightTheme,
    dark: darkTheme
  };

  /**
   * 初始化主题管理器
   */
  init() {
    // 从本地存储读取主题设置
    const savedTheme = localStorage.getItem('app-theme') as Theme;
    if (savedTheme && this.themes[savedTheme]) {
      this.currentTheme = savedTheme;
    } else {
      // 检测系统主题偏好
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      this.currentTheme = prefersDark ? 'dark' : 'light';
    }
    
    this.applyTheme(this.currentTheme);
    
    // 监听系统主题变化
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
      if (!localStorage.getItem('app-theme')) {
        this.setTheme(e.matches ? 'dark' : 'light');
      }
    });
  }

  /**
   * 设置主题
   * @param theme 主题名称
   */
  setTheme(theme: Theme) {
    if (!this.themes[theme]) {
      console.warn(`Theme '${theme}' not found`);
      return;
    }

    this.currentTheme = theme;
    this.applyTheme(theme);
    localStorage.setItem('app-theme', theme);
  }

  /**
   * 切换主题
   */
  toggleTheme() {
    const newTheme = this.currentTheme === 'light' ? 'dark' : 'light';
    this.setTheme(newTheme);
  }

  /**
   * 获取当前主题
   */
  getCurrentTheme(): Theme {
    return this.currentTheme;
  }

  /**
   * 获取主题配置
   * @param theme 主题名称
   */
  getThemeConfig(theme?: Theme): ThemeConfig {
    return this.themes[theme || this.currentTheme];
  }

  /**
   * 应用主题样式
   * @param theme 主题名称
   */
  private applyTheme(theme: Theme) {
    const config = this.themes[theme];
    const root = document.documentElement;

    // 设置CSS变量
    root.style.setProperty('--theme-primary', config.colors.primary);
    root.style.setProperty('--theme-secondary', config.colors.secondary);
    root.style.setProperty('--theme-background', config.colors.background);
    root.style.setProperty('--theme-surface', config.colors.surface);
    root.style.setProperty('--theme-text', config.colors.text);
    root.style.setProperty('--theme-text-secondary', config.colors.textSecondary);
    root.style.setProperty('--theme-border', config.colors.border);
    root.style.setProperty('--theme-shadow', config.colors.shadow);

    // 设置主题类名
    root.className = root.className.replace(/theme-\w+/g, '');
    root.classList.add(`theme-${theme}`);

    // 更新Element Plus主题变量
    if (theme === 'dark') {
      root.style.setProperty('--el-color-primary', config.colors.primary);
      root.style.setProperty('--el-bg-color', config.colors.surface);
      root.style.setProperty('--el-text-color-primary', config.colors.text);
      root.style.setProperty('--el-text-color-regular', config.colors.textSecondary);
      root.style.setProperty('--el-border-color', config.colors.border);
      root.style.setProperty('--el-fill-color-blank', config.colors.surface);
    } else {
      // 重置为默认值
      root.style.removeProperty('--el-color-primary');
      root.style.removeProperty('--el-bg-color');
      root.style.removeProperty('--el-text-color-primary');
      root.style.removeProperty('--el-text-color-regular');
      root.style.removeProperty('--el-border-color');
      root.style.removeProperty('--el-fill-color-blank');
    }
  }

  /**
   * 注册主题变化监听器
   * @param callback 回调函数
   */
  onThemeChange(callback: (theme: Theme) => void) {
    const originalSetTheme = this.setTheme.bind(this);
    this.setTheme = (theme: Theme) => {
      originalSetTheme(theme);
      callback(theme);
    };
  }
}

// 导出单例实例
export const themeManager = new ThemeManager();

// 导出主题配置
export { lightTheme, darkTheme };