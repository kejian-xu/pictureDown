// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Post {
    id: u32,
    tags: String,
    created_at: i64,
    updated_at: i64,
    creator_id: u32,
    approver_id: Option<u32>,
    author: String,
    change: u32,
    source: String,
    score: i32,
    md5: String,
    file_size: u64,
    file_ext: String,
    file_url: String,
    preview_url: String,
    sample_url: String,
    sample_width: u32,
    sample_height: u32,
    preview_width: u32,
    preview_height: u32,
    actual_preview_width: u32,
    actual_preview_height: u32,
    jpeg_url: String,
    jpeg_width: u32,
    jpeg_height: u32,
    jpeg_file_size: u64,
    rating: String,
    has_children: bool,
    parent_id: Option<u32>,
    status: String,
    width: u32,
    height: u32,
    is_held: bool,
    frames_pending_string: String,
    frames_pending: Vec<serde_json::Value>,
    frames_string: String,
    frames: Vec<serde_json::Value>,
    is_note_locked: bool,
    last_noted_at: i64,
    last_commented_at: i64,
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
async fn fetch_posts(tags: Option<String>, limit: Option<u32>, page: Option<u32>) -> Result<Vec<Post>, String> {
    // 创建一个基本的客户端配置
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36")
        .danger_accept_invalid_certs(true) // 忽略证书错误
        .build()
        .map_err(|e| format!("Failed to create client: {}", e))?;

    // 构建API URL
    let mut api_url = "https://yande.re/post.json".to_string();
    let mut query_params = vec![];

    if let Some(t) = tags {
        if !t.trim().is_empty() {
            query_params.push(format!("tags={}", urlencoding::encode(&t)));
        }
    }

    if let Some(l) = limit {
        query_params.push(format!("limit={}", l));
    }

    if let Some(p) = page {
        query_params.push(format!("page={}", p));
    }

    if !query_params.is_empty() {
        api_url.push('?');
        api_url.push_str(&query_params.join("&"));
    }

    // 发送请求
    match client.get(&api_url)
        .header("Accept", "application/json")
        .send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<Vec<Post>>().await {
                    Ok(posts) => Ok(posts),
                    Err(e) => Err(format!("Failed to parse JSON: {}", e)),
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
        .invoke_handler(tauri::generate_handler![greet, fetch_html, fetch_posts])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
