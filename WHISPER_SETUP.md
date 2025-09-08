# Whisper 语音识别引擎安装指南

## 问题说明

当前 FlowText 应用的 Whisper 引擎尚未安装真正的 Whisper 模型，所以会显示安装指导信息而不是真实的语音识别结果。

## 安装真正的 Whisper 引擎

### 方法 1: 使用 pip 安装（推荐）

```bash
# 安装openai-whisper
pip install openai-whisper

# 或者使用pip3
pip3 install openai-whisper
```

### 方法 2: 使用 Homebrew 安装

```bash
# 如果您使用Mac并安装了Homebrew
brew install openai-whisper
```

### 方法 3: 使用 conda 安装

```bash
# 如果您使用conda环境
conda install -c conda-forge openai-whisper
```

## 验证安装

安装完成后，您可以通过以下命令验证：

```bash
# 检查whisper命令是否可用
whisper --help

# 或者检查Python模块
python3 -c "import whisper; print('Whisper安装成功!')"
```

## 安装完成后的效果

一旦正确安装 Whisper，FlowText 将能够：

1. **真实语音识别**：分析您的音频内容并生成准确的字幕
2. **动态字幕数量**：根据实际语音内容生成相应数量的字幕片段
3. **准确的时间轴**：字幕时间将与音频内容精确对应
4. **多语言支持**：支持中文、英文、日文等多种语言的识别

## 系统要求

- Python 3.7+
- 足够的内存和存储空间（Whisper 模型文件较大）
- 对于大文件，建议使用 GPU 加速（可选）

## 故障排除

### 如果安装失败

1. 确保 Python 版本符合要求：`python3 --version`
2. 更新 pip：`pip3 install --upgrade pip`
3. 如果网络问题，可以使用镜像源：
   ```bash
   pip3 install -i https://pypi.tuna.tsinghua.edu.cn/simple openai-whisper
   ```

### 如果运行时出错

1. 检查音频文件是否损坏
2. 确保有足够的磁盘空间存储临时文件
3. 对于大文件，可能需要较长的处理时间

## 其他识别引擎

如果您不想安装 Whisper，也可以：

1. **配置百度 API**：在设置中填入百度智能云的 API 密钥
2. **配置腾讯云 API**：填入腾讯云的 SecretId 和 SecretKey
3. **配置阿里云 API**：填入阿里云的 AccessKey 信息

这些云端 API 无需本地安装，但需要网络连接和有效的 API 密钥。

## 总结

目前您看到的固定字幕内容确实不是真正的语音识别结果。安装真正的 Whisper 引擎后，您将获得基于音频内容的准确识别结果。
