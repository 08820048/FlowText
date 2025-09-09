use chrono::Utc;
use hex;
use hmac::{Hmac, Mac};

use reqwest::Client;
use sha1::Sha1;
use sha2::Digest;
use std::collections::HashMap;
use uuid::Uuid;

type HmacSha1 = Hmac<Sha1>;

/// COS配置信息
#[derive(Debug, Clone)]
pub struct CosConfig {
    pub secret_id: String,
    pub secret_key: String,
    pub bucket: String,
    pub region: String,
    pub domain: Option<String>,
}

/// COS客户端
pub struct CosClient {
    config: CosConfig,
    client: Client,
}

impl CosClient {
    /// 创建新的COS客户端
    pub fn new(config: CosConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    /// 上传文件到COS
    pub async fn upload_file(
        &self,
        file_data: &[u8],
        file_name: &str,
        content_type: Option<&str>,
    ) -> Result<String, String> {
        // 生成唯一的对象键
        let object_key = format!("audio/{}/{}", Uuid::new_v4(), file_name);

        // 构建上传URL
        let host = format!(
            "{}.cos.{}.myqcloud.com",
            self.config.bucket, self.config.region
        );
        let url = format!("https://{}/{}", host, object_key);

        // 获取当前时间
        let now = Utc::now();
        let timestamp = now.timestamp();

        // 构建请求头
        let mut headers = HashMap::new();
        headers.insert("Host".to_string(), host.clone());
        headers.insert(
            "Date".to_string(),
            now.format("%a, %d %b %Y %H:%M:%S GMT").to_string(),
        );

        if let Some(ct) = content_type {
            headers.insert("Content-Type".to_string(), ct.to_string());
        } else {
            // 根据文件扩展名猜测MIME类型
            let mime_type = mime_guess::from_path(file_name)
                .first_or_octet_stream()
                .to_string();
            headers.insert("Content-Type".to_string(), mime_type);
        }

        // 生成签名
        let authorization = self.generate_authorization("PUT", &object_key, &headers, timestamp)?;

        // 构建请求
        let mut request_builder = self
            .client
            .put(&url)
            .header("Authorization", authorization)
            .header("Host", &host)
            .header("Date", now.format("%a, %d %b %Y %H:%M:%S GMT").to_string())
            .body(file_data.to_vec());

        // 添加Content-Type头
        if let Some(ct) = content_type {
            request_builder = request_builder.header("Content-Type", ct);
        } else {
            let mime_type = mime_guess::from_path(file_name)
                .first_or_octet_stream()
                .to_string();
            request_builder = request_builder.header("Content-Type", mime_type);
        }

        // 发送请求
        let response = request_builder.send().await.map_err(|e| {
            if e.is_connect() {
                format!(
                    "COS上传失败: 网络连接错误\n\n可能的原因：\n\
                    1. 网络连接不稳定，请检查网络连接\n\
                    2. 腾讯云COS服务暂时不可用\n\
                    3. 防火墙或代理设置阻止了连接\n\
                    4. DNS解析问题\n\n\
                    建议：\n\
                    - 检查网络连接是否正常\n\
                    - 尝试使用Whisper本地识别（不需要网络）\n\
                    - 稍后重试\n\n\
                    详细错误: {}",
                    e
                )
            } else if e.is_timeout() {
                format!(
                    "COS上传失败: 请求超时\n\n\
                    文件大小: {:.1} MB\n\
                    可能原因：网络速度较慢或文件过大\n\n\
                    建议：\n\
                    - 检查网络连接速度\n\
                    - 尝试使用Whisper本地识别\n\
                    - 压缩音频文件后重试\n\n\
                    详细错误: {}",
                    file_data.len() as f64 / (1024.0 * 1024.0),
                    e
                )
            } else {
                format!("上传文件到COS失败: {}", e)
            }
        })?;

        if response.status().is_success() {
            // 返回文件的访问URL
            let access_url = if let Some(domain) = &self.config.domain {
                format!("https://{}/{}", domain, object_key)
            } else {
                url
            };
            Ok(access_url)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            // 提供更详细的错误信息
            let detailed_error = match status.as_u16() {
                403 => format!(
                    "COS上传失败: 权限被拒绝 (HTTP 403)\n\n可能的原因：\n\
                    1. 存储桶权限设置不正确，请设置为'公有读私有写'\n\
                    2. API密钥缺少COS操作权限\n\
                    3. 存储桶名称或地域配置错误\n\
                    4. 存储桶不存在或已被删除\n\n\
                    当前配置：\n\
                    - 存储桶: {}\n\
                    - 地域: {}\n\
                    - 上传URL: {}\n\n\
                    详细错误: {}",
                    self.config.bucket, self.config.region, url, error_text
                ),
                404 => format!(
                    "COS上传失败: 存储桶不存在 (HTTP 404)\n\n\
                    请检查：\n\
                    1. 存储桶名称是否正确: {}\n\
                    2. 地域是否正确: {}\n\
                    3. 存储桶是否已创建\n\n\
                    详细错误: {}",
                    self.config.bucket, self.config.region, error_text
                ),
                _ => format!("COS上传失败: HTTP {}, {}", status, error_text),
            };

            Err(detailed_error)
        }
    }

    /// 生成COS API签名（按照腾讯云官方文档）
    fn generate_authorization(
        &self,
        method: &str,
        object_key: &str,
        headers: &HashMap<String, String>,
        timestamp: i64,
    ) -> Result<String, String> {
        // 签名有效期（1小时）
        let expire_time = timestamp + 3600;

        // 1. 生成 KeyTime
        let key_time = format!("{};{}", timestamp, expire_time);

        // 2. 生成 SignKey
        let mut mac = HmacSha1::new_from_slice(self.config.secret_key.as_bytes())
            .map_err(|e| format!("创建HMAC失败: {}", e))?;
        mac.update(key_time.as_bytes());
        let sign_key = hex::encode(mac.finalize().into_bytes());

        // 3. 生成 HttpString（按照官方格式）
        let http_string = self.build_http_string_official(method, object_key, headers)?;

        // 4. 生成 StringToSign
        let string_to_sign = format!("sha1\n{}\n{}\n", key_time, self.sha1_hash(&http_string));

        // 5. 生成 Signature
        let mut mac = HmacSha1::new_from_slice(sign_key.as_bytes())
            .map_err(|e| format!("创建签名HMAC失败: {}", e))?;
        mac.update(string_to_sign.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        // 6. 构建 Authorization
        let header_list = self.get_header_list_official(headers);
        let authorization = format!(
            "q-sign-algorithm=sha1&q-ak={}&q-sign-time={}&q-key-time={}&q-header-list={}&q-url-param-list=&q-signature={}",
            self.config.secret_id,
            key_time,
            key_time,
            header_list,
            signature
        );

        Ok(authorization)
    }

    /// 构建HTTP字符串（按照腾讯云官方文档格式）
    fn build_http_string_official(
        &self,
        method: &str,
        object_key: &str,
        headers: &HashMap<String, String>,
    ) -> Result<String, String> {
        // 1. HTTP方法（小写）
        let http_method = method.to_lowercase();

        // 2. URI路径
        let uri_path = format!("/{}", object_key);

        // 3. HTTP参数（暂时为空）
        let http_parameters = "";

        // 4. HTTP头部（按照官方格式）
        let http_headers = self.build_header_string_official(headers);

        // 5. 构建HttpString
        let http_string = format!(
            "{}\n{}\n{}\n{}\n",
            http_method, uri_path, http_parameters, http_headers
        );

        Ok(http_string)
    }

    /// 构建头部字符串（按照腾讯云官方格式）
    fn build_header_string_official(&self, headers: &HashMap<String, String>) -> String {
        let mut header_pairs = Vec::new();

        // 按照腾讯云COS官方文档格式构建头部字符串
        for (key, value) in headers {
            let key_lower = key.to_lowercase();
            // 根据官方文档，需要对头部值进行URL编码
            let encoded_value = self.url_encode(value);
            header_pairs.push(format!("{}={}", key_lower, encoded_value));
        }

        // 按字母顺序排序
        header_pairs.sort();
        header_pairs.join("&")
    }

    /// URL编码（按照腾讯云COS要求）
    fn url_encode(&self, input: &str) -> String {
        input
            .chars()
            .map(|c| match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                _ => format!("%{:02X}", c as u8),
            })
            .collect()
    }

    /// 获取头部列表（按照腾讯云官方格式）
    fn get_header_list_official(&self, headers: &HashMap<String, String>) -> String {
        let mut header_keys: Vec<String> = headers.keys().map(|k| k.to_lowercase()).collect();
        header_keys.sort();
        header_keys.join(";")
    }

    /// 带重试机制的请求发送
    async fn send_request_with_retry(
        &self,
        request_builder: reqwest::RequestBuilder,
        max_retries: u32,
    ) -> Result<reqwest::Response, String> {
        let mut last_error = String::new();

        for attempt in 0..=max_retries {
            // 克隆请求构建器（注意：这里需要重新构建请求）
            let result = if attempt == 0 {
                request_builder
                    .try_clone()
                    .ok_or("无法克隆请求")?
                    .send()
                    .await
            } else {
                // 对于重试，我们需要重新构建整个请求
                // 这里先返回错误，让调用方重新构建请求
                return Err(format!("网络连接失败，已重试{}次: {}", attempt, last_error));
            };

            match result {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = format!("{}", e);

                    // 检查是否是网络连接错误
                    if e.is_connect() || e.is_timeout() {
                        if attempt < max_retries {
                            println!("COS上传失败，第{}次重试中... 错误: {}", attempt + 1, e);
                            // 等待一段时间后重试
                            tokio::time::sleep(tokio::time::Duration::from_secs(
                                2_u64.pow(attempt),
                            ))
                            .await;
                            continue;
                        }
                    }

                    // 非网络错误或已达到最大重试次数
                    return Err(format!("上传文件到COS失败: {}", e));
                }
            }
        }

        Err(format!("上传失败，已重试{}次: {}", max_retries, last_error))
    }

    /// 获取头部列表
    fn get_header_list(&self, headers: &HashMap<String, String>) -> String {
        let mut header_keys: Vec<String> = headers.keys().map(|k| k.to_lowercase()).collect();
        header_keys.sort();
        header_keys.join(";")
    }

    /// 构建头部字符串
    fn build_header_string(&self, headers: &HashMap<String, String>, header_list: &str) -> String {
        let header_keys: Vec<&str> = header_list.split(';').collect();
        let mut header_pairs = Vec::new();

        for key in header_keys {
            if let Some(value) = headers.get(&key.to_string()).or_else(|| {
                // 尝试大小写不敏感匹配
                headers
                    .iter()
                    .find(|(k, _)| k.to_lowercase() == key)
                    .map(|(_, v)| v)
            }) {
                header_pairs.push(format!("{}={}", key, value.trim()));
            }
        }

        header_pairs.join("&")
    }

    /// 计算SHA1哈希
    fn sha1_hash(&self, data: &str) -> String {
        let mut hasher = Sha1::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// 删除文件（可选功能）
    pub async fn delete_file(&self, object_key: &str) -> Result<(), String> {
        let host = format!(
            "{}.cos.{}.myqcloud.com",
            self.config.bucket, self.config.region
        );
        let url = format!("https://{}/{}", host, object_key);

        let now = Utc::now();
        let timestamp = now.timestamp();

        let mut headers = HashMap::new();
        headers.insert("Host".to_string(), host.clone());
        headers.insert(
            "Date".to_string(),
            now.format("%a, %d %b %Y %H:%M:%S GMT").to_string(),
        );

        let authorization =
            self.generate_authorization("DELETE", object_key, &headers, timestamp)?;

        let response = self
            .client
            .delete(&url)
            .header("Authorization", authorization)
            .header("Host", &host)
            .header("Date", now.format("%a, %d %b %Y %H:%M:%S GMT").to_string())
            .send()
            .await
            .map_err(|e| format!("删除COS文件失败: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("COS删除失败: HTTP {}, {}", status, error_text))
        }
    }
}

/// 从文件名提取对象键
pub fn extract_object_key_from_url(url: &str) -> Option<String> {
    if let Ok(parsed_url) = url::Url::parse(url) {
        let path = parsed_url.path();
        if path.starts_with('/') {
            Some(path[1..].to_string())
        } else {
            Some(path.to_string())
        }
    } else {
        None
    }
}
