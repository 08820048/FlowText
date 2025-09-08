# FlowText - 智能视频字幕提取工具

<div align="center">
  <img src="https://img.shields.io/badge/Vue.js-3.x-4FC08D?style=for-the-badge&logo=vue.js&logoColor=white" alt="Vue.js">
  <img src="https://img.shields.io/badge/TypeScript-5.x-3178C6?style=for-the-badge&logo=typescript&logoColor=white" alt="TypeScript">
  <img src="https://img.shields.io/badge/Tauri-2.0-FFC131?style=for-the-badge&logo=tauri&logoColor=white" alt="Tauri">
  <img src="https://img.shields.io/badge/Rust-1.70+-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/Element_Plus-2.x-409EFF?style=for-the-badge&logo=element&logoColor=white" alt="Element Plus">
</div>

## 📖 项目简介

FlowText 是一款基于 Tauri 2.0 开发的桌面应用程序，专为视频字幕提取和编辑而设计。它集成了多种语音识别引擎，支持从视频文件中自动提取音频并生成高质量的字幕文件。

### ✨ 主要特性

- 🎥 **视频文件支持** - 支持多种主流视频格式（MP4、AVI、MOV、MKV等）
- 🎵 **音频提取** - 使用 FFmpeg 高效提取视频中的音频轨道
- 🤖 **多引擎语音识别** - 集成百度智能云、腾讯云、阿里云和本地 Whisper 模型
- ✏️ **智能字幕编辑** - 提供直观的字幕编辑界面，支持时间轴调整和内容修改
- 📄 **多格式导出** - 支持 SRT、ASS、VTT、TXT、JSON 等多种字幕格式
- 🎨 **现代化界面** - 基于 Element Plus 的美观 UI，支持浅色/深色主题
- 📊 **进度监控** - 实时显示任务进度和状态
- 🛡️ **错误处理** - 完善的错误处理和用户反馈机制
- 🌐 **多语言支持** - 支持中文、英语、日语、韩语等多种语言识别

## 🚀 快速开始

### 环境要求

- **Node.js** >= 18.0.0
- **Rust** >= 1.70.0
- **操作系统**: Windows 10+, macOS 10.15+, Linux (Ubuntu 18.04+)

### 安装步骤

1. **克隆项目**
   ```bash
   git clone https://github.com/your-username/FlowText.git
   cd FlowText
   ```

2. **安装依赖**
   ```bash
   npm install
   ```

3. **开发模式运行**
   ```bash
   npm run tauri dev
   ```

4. **构建生产版本**
   ```bash
   npm run tauri build
   ```

## 🔧 配置说明

### API 密钥配置

在使用云端语音识别服务前，需要配置相应的 API 密钥：

#### 百度智能云
1. 访问 [百度智能云控制台](https://console.bce.baidu.com/)
2. 创建语音识别应用
3. 获取 App ID、API Key 和 Secret Key
4. 在应用设置中填入相关信息

#### 腾讯云
1. 访问 [腾讯云控制台](https://console.cloud.tencent.com/)
2. 开通语音识别服务
3. 获取 Secret ID 和 Secret Key
4. 在应用设置中填入相关信息

#### 阿里云
1. 访问 [阿里云控制台](https://ecs.console.aliyun.com/)
2. 开通智能语音交互服务
3. 获取 Access Key ID 和 Access Key Secret
4. 在应用设置中填入相关信息

### Whisper 本地模型

Whisper 是 OpenAI 开发的开源语音识别模型，无需 API 密钥即可使用：

- **Tiny**: 最快速度，较低精度
- **Base**: 平衡速度和精度
- **Small**: 较慢速度，较高精度

## 📱 使用指南

### 1. 导入视频文件

- 点击「选择视频文件」按钮或直接拖拽视频文件到应用窗口
- 支持的格式：MP4、AVI、MOV、MKV、WMV、FLV 等
- 应用会自动显示视频信息（时长、分辨率、编码格式等）

### 2. 配置识别参数

- **识别引擎**: 选择合适的语音识别引擎
- **识别语言**: 根据视频内容选择对应语言
- **音频质量**: 调整音频提取质量（影响识别精度）

### 3. 开始识别

- 点击「开始识别」按钮
- 应用会自动提取音频并发送到选定的识别引擎
- 可在任务状态栏查看实时进度

### 4. 编辑字幕

- 识别完成后，在「字幕编辑」标签页中查看和编辑字幕
- 支持的操作：
  - 添加新字幕条目
  - 修改字幕文本内容
  - 调整时间轴（开始时间、结束时间）
  - 删除不需要的字幕
  - 跳转到指定时间点

### 5. 导出字幕

- 点击「导出字幕」按钮
- 选择导出格式和文件名
- 支持的格式：
  - **SRT**: 最常用的字幕格式
  - **ASS**: 高级字幕格式，支持样式
  - **VTT**: Web 视频字幕格式
  - **TXT**: 纯文本格式
  - **JSON**: 结构化数据格式

## 🎨 界面预览

### 主界面
- 左侧：视频导入和播放器
- 右侧：语音识别、字幕编辑、应用设置
- 底部：任务状态栏

### 主题支持
- **浅色主题**: 适合白天使用的明亮界面
- **深色主题**: 适合夜间使用的暗色界面
- 可在设置中一键切换

## 🛠️ 技术架构

### 前端技术栈
- **Vue 3**: 渐进式 JavaScript 框架
- **TypeScript**: 类型安全的 JavaScript 超集
- **Element Plus**: Vue 3 组件库
- **Pinia**: Vue 状态管理库
- **Vite**: 现代化构建工具

### 后端技术栈
- **Tauri 2.0**: 跨平台桌面应用框架
- **Rust**: 系统级编程语言
- **FFmpeg**: 音视频处理库
- **Tokio**: 异步运行时

### 项目结构
```
FlowText/
├── src/                    # 前端源码
│   ├── components/         # Vue 组件
│   ├── stores/            # Pinia 状态管理
│   ├── utils/             # 工具函数
│   ├── types/             # TypeScript 类型定义
│   └── styles/            # 样式文件
├── src-tauri/             # Tauri 后端源码
│   ├── src/               # Rust 源码
│   ├── Cargo.toml         # Rust 依赖配置
│   └── tauri.conf.json    # Tauri 配置
├── public/                # 静态资源
└── dist/                  # 构建输出
```

## 🤝 贡献指南

我们欢迎所有形式的贡献！请遵循以下步骤：

1. Fork 本项目
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

### 开发规范

- 使用 TypeScript 进行类型安全开发
- 遵循 Vue 3 Composition API 最佳实践
- 添加适当的注释和文档
- 确保代码通过 ESLint 检查
- 编写单元测试（如适用）

## 📝 更新日志

### v1.0.0 (2024-01-XX)

#### 新增功能
- ✨ 完整的视频字幕提取工作流
- 🎵 支持多种视频格式的音频提取
- 🤖 集成四种语音识别引擎
- ✏️ 直观的字幕编辑界面
- 📄 多格式字幕导出
- 🎨 现代化 UI 设计
- 🌙 深色主题支持
- 📊 实时进度监控
- 🛡️ 完善的错误处理

## 🐛 问题反馈

如果您遇到任何问题或有改进建议，请通过以下方式联系我们：

- [GitHub Issues](https://github.com/your-username/FlowText/issues)
- 邮箱：your-email@example.com

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

感谢以下开源项目和服务提供商：

- [Tauri](https://tauri.app/) - 跨平台桌面应用框架
- [Vue.js](https://vuejs.org/) - 渐进式 JavaScript 框架
- [Element Plus](https://element-plus.org/) - Vue 3 组件库
- [FFmpeg](https://ffmpeg.org/) - 音视频处理库
- [OpenAI Whisper](https://openai.com/research/whisper) - 开源语音识别模型
- 百度智能云、腾讯云、阿里云 - 语音识别服务

---

<div align="center">
  <p>如果这个项目对您有帮助，请给我们一个 ⭐️</p>
  <p>Made with ❤️ by FlowText Team</p>
</div>
