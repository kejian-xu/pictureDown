// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use reqwest::{Client, header};
use quick_xml::de::from_str;


#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    #[serde(rename = "id")]
    pub id: u32,

    #[serde(rename = "tags")]
    pub tags: String,

    #[serde(rename = "created_at")]
    pub created_at: i64,

    #[serde(rename = "updated_at")]
    pub updated_at: i64,

    #[serde(rename = "creator_id")]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub creator_id: Option<u32>,

    #[serde(rename = "approver_id")]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub approver_id: Option<u32>,

    #[serde(rename = "author")]
    pub author: String,

    #[serde(rename = "change")]
    pub change: u32,

    #[serde(rename = "source")]
    pub source: String,

    #[serde(rename = "score")]
    pub score: i32,

    #[serde(rename = "md5")]
    pub md5: String,

    #[serde(rename = "file_size")]
    pub file_size: u64,

    #[serde(rename = "file_ext")]
    pub file_ext: String,

    #[serde(rename = "file_url")]
    pub file_url: String,

    #[serde(rename = "is_shown_in_index")]
    #[serde(deserialize_with = "string_to_bool")]
    pub is_shown_in_index: bool,

    #[serde(rename = "preview_url")]
    pub preview_url: String,

    #[serde(rename = "sample_url")]
    pub sample_url: String,

    #[serde(rename = "sample_width")]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub sample_width: Option<u32>,

    #[serde(rename = "sample_height")]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub sample_height: Option<u32>,

    #[serde(rename = "sample_file_size")]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub sample_file_size: Option<u64>,

    #[serde(rename = "preview_width")]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub preview_width: Option<u32>,

    #[serde(rename = "preview_height")]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub preview_height: Option<u32>,

    #[serde(rename = "actual_preview_width")]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub actual_preview_width: Option<u32>,

    #[serde(rename = "actual_preview_height")]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub actual_preview_height: Option<u32>,

    #[serde(rename = "jpeg_url")]
    pub jpeg_url: String,

    #[serde(rename = "jpeg_width")]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub jpeg_width: Option<u32>,

    #[serde(rename = "jpeg_height")]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub jpeg_height: Option<u32>,

    #[serde(rename = "jpeg_file_size")]
    pub jpeg_file_size: u64,

    #[serde(rename = "rating")]
    pub rating: String,

    #[serde(rename = "is_rating_locked")]
    #[serde(deserialize_with = "string_to_bool")]
    pub is_rating_locked: bool,

    #[serde(rename = "has_children")]
    #[serde(deserialize_with = "string_to_bool")]
    pub has_children: bool,

    #[serde(rename = "parent_id")]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub parent_id: Option<u32>,

    #[serde(rename = "status")]
    pub status: String,

    #[serde(rename = "is_pending")]
    #[serde(deserialize_with = "string_to_bool")]
    pub is_pending: bool,

    #[serde(rename = "width")]
    pub width: u32,

    #[serde(rename = "height")]
    pub height: u32,

    #[serde(rename = "is_held")]
    #[serde(deserialize_with = "string_to_bool")]
    pub is_held: bool,

    #[serde(rename = "frames_pending_string")]
    pub frames_pending_string: String,

    #[serde(rename = "frames_string")]
    pub frames_string: String,

    #[serde(rename = "is_note_locked")]
    #[serde(deserialize_with = "string_to_bool")]
    pub is_note_locked: bool,

    #[serde(rename = "last_noted_at")]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub last_noted_at: Option<i64>,

    #[serde(rename = "last_commented_at")]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub last_commented_at: Option<i64>,
}

// 辅助函数：将空字符串转为 None
fn empty_string_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        match s.parse::<T>() {
            Ok(val) => Ok(Some(val)),
            Err(e) => Err(serde::de::Error::custom(format!("解析错误: {}", e))),
        }
    }
}

// 辅助函数：将字符串 "true"/"false" 转为 bool
fn string_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(serde::de::Error::custom(format!("无效的布尔值: {}", s))),
    }
}

// 辅助函数：空字符串转为默认值（空 Vec）
fn empty_string_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Default + serde::Deserialize<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(T::default())
    } else {
        T::deserialize(serde::de::value::StringDeserializer::new(s))
    }
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Posts {
    #[serde(rename = "count")]
    pub count: i32,
    #[serde(rename = "offset")]
    pub offset: i32,
    #[serde(rename = "post")]
    #[serde(default)]
    // pub posts: Vec<Post>,
     pub posts: Vec<Value>,  // 可以是任何结构
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn fetch_html(url: &str) -> Result<String, String> {
    // 创建一个基本的客户端配置，完全使用HTTP/1.1以避免HTTP/2帧错误
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36")
        .danger_accept_invalid_certs(true) // 忽略证书错误
        // 不使用http2_prior_knowledge方法，让reqwest自动协商HTTP版本
        .build()
        .map_err(|e| format!("Failed to create client: {}", e))?;
    
    // 发送请求，使用链式调用添加必要的头信息
    match client.get(url)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Connection", "keep-alive")
        .header("Upgrade-Insecure-Requests", "1")
        .header("Referer", "https://www.google.com/")
        .send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.text().await {
                    Ok(html) => Ok(html),
                    Err(e) => Err(format!("Failed to read response text: {}", e)),
                }
            } else {
                Err(format!("Request failed with status: {}", response.status()))
            }
        },
        Err(e) => Err(format!("Failed to send request: {}", e)),
    }
}
#[tauri::command]
async fn fetch_posts(
    tags: Option<String>,
    limit: Option<u32>,
    page: Option<u32>,
) -> Result<Posts, String> {

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0")
        .build()
        .map_err(|e| e.to_string())?;

    let api_url = "https://yande.re/post.xml";

    let mut query: HashMap<&str, String> = HashMap::new();

    if let Some(t) = tags {
        query.insert("tags", t);
    }

    if let Some(l) = limit {
        query.insert("limit", l.to_string());
    }

    if let Some(p) = page {
        query.insert("page", p.to_string());
    }

    let response = client
        .get(api_url)
        .query(&query)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("Request failed: {}", response.status()));
    }

    let xml = response.text().await.map_err(|e| e.to_string())?;
    println!("Received XML: {}", xml); // 调试输出
    debug_find_empty_fields(&xml); // 调试函数，查找空字段
    let posts: Posts = serde_xml_rs::from_str(&xml).map_err(|e| format!("XML parse error: {}", e))?;
    Ok(posts)
}
fn debug_find_empty_fields(xml: &str) {
    println!("在 XML 中搜索空字段...");
    
    // 查找所有属性模式
    let re = regex::Regex::new(r#"(\w+)=""#).unwrap();
    
    for cap in re.captures_iter(xml) {
        println!("发现空字段: {}=''", &cap[1]);
    }
    
    // 打印 XML 中所有的属性
    let attr_re = regex::Regex::new(r#"(\w+)="([^"]*)""#).unwrap();
    
    println!("\n所有字段的值:");
    for cap in attr_re.captures_iter(xml) {
        let field = &cap[1];
        let value = &cap[2];
        
        if value.is_empty() {
            println!("⚠️  {}: 空字符串", field);
        } else {
            println!("✅ {}: '{}'", field, value);
        }
    }
}
#[tauri::command]
async fn fetch_image_as_base64(url: String) -> Result<String, String> {
    // 创建一个基本的客户端配置
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36")
        .danger_accept_invalid_certs(true) // 忽略证书错误
        .build()
        .map_err(|e| format!("Failed to create client: {}", e))?;

    println!("Fetching image from URL: {}", url);

    match client.get(&url)
        .header("Accept", "image/*")
        .send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.bytes().await {
                    Ok(bytes) => {
                        let base64_string = base64::encode(&bytes);
                        println!("Image fetched successfully, size: {} bytes, base64 length: {}", bytes.len(), base64_string.len());
                        Ok(base64_string)
                    },
                    Err(e) => Err(format!("Failed to read image bytes: {}", e)),
                }
            } else {
                Err(format!("Request failed with status: {}", response.status()))
            }
        },
        Err(e) => Err(format!("Failed to send request: {}", e)),
    }
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_upload::init())
        .invoke_handler(tauri::generate_handler![greet, fetch_html, fetch_posts, fetch_image_as_base64])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
