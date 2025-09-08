/**
 * 主题管理器
 * 负责管理应用的主题样式应用（仅支持浅色主题）
 */

export type Theme = 'light';

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

// 浅色主题配置 - 扁平化设计
const lightTheme: ThemeConfig = {
  name: 'light',
  colors: {
    primary: '#0fdc78',
    secondary: '#000000',
    background: '#ffffff',
    surface: '#ffffff',
    text: '#000000',
    textSecondary: '#666666',
    border: '#0fdc78',
    shadow: 'none'
  }
};

// 深色主题已移除，仅支持浅色主题

class ThemeManager {
  private currentTheme: Theme = 'light';
  private themes: Record<Theme, ThemeConfig> = {
    light: lightTheme
  };

  /**
   * 初始化主题管理器
   */
  init() {
    // 固定使用浅色主题
    this.currentTheme = 'light';
    this.applyTheme(this.currentTheme);
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
   * 切换主题功能已移除
   */

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

    // 应用Element Plus主题变量
    root.style.setProperty('--el-color-primary', config.colors.primary);
    root.style.setProperty('--el-bg-color', config.colors.surface);
    root.style.setProperty('--el-text-color-primary', config.colors.text);
    root.style.setProperty('--el-text-color-regular', config.colors.textSecondary);
    root.style.setProperty('--el-border-color', config.colors.border);
    root.style.setProperty('--el-fill-color-blank', config.colors.surface);
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
export { lightTheme };