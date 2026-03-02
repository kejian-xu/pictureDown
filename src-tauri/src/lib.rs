// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
// {
//     "id": 1255839,
//     "tags": "blue_archive halo sorasaki_hina tagme",
//     "created_at": 1772443117,
//     "updated_at": 1772443117,
//     "creator_id": 294900,
//     "approver_id": null,
//     "author": "Aleax",
//     "change": 6492274,
//     "source": "https://www.pixiv.net/artworks/141370841",
//     "score": 1,
//     "md5": "9bdd440170b23dad4d950a265449ffd2",
//     "file_size": 7123727,
//     "file_ext": "jpg",
//     "file_url": "https://files.yande.re/image/9bdd440170b23dad4d950a265449ffd2/yande.re%201255839%20blue_archive%20halo%20sorasaki_hina%20tagme.jpg",
//     "is_shown_in_index": true,
//     "preview_url": "https://assets.yande.re/data/preview/9b/dd/9bdd440170b23dad4d950a265449ffd2.jpg",
//     "preview_width": 150,
//     "preview_height": 116,
//     "actual_preview_width": 300,
//     "actual_preview_height": 231,
//     "sample_url": "https://files.yande.re/sample/9bdd440170b23dad4d950a265449ffd2/yande.re%201255839%20sample%20blue_archive%20halo%20sorasaki_hina%20tagme.jpg",
//     "sample_width": 1500,
//     "sample_height": 1155,
//     "sample_file_size": 389491,
//     "jpeg_url": "https://files.yande.re/image/9bdd440170b23dad4d950a265449ffd2/yande.re%201255839%20blue_archive%20halo%20sorasaki_hina%20tagme.jpg",
//     "jpeg_width": 3896,
//     "jpeg_height": 3000,
//     "jpeg_file_size": 0,
//     "rating": "s",
//     "is_rating_locked": false,
//     "has_children": false,
//     "parent_id": null,
//     "status": "active",
//     "is_pending": false,
//     "width": 3896,
//     "height": 3000,
//     "is_held": true,
//     "frames_pending_string": "",
//     "frames_pending": [],
//     "frames_string": "",
//     "frames": [],
//     "is_note_locked": false,
//     "last_noted_at": 0,
//     "last_commented_at": 0
// }
struct Post {
    id: u32,
    tags: String,
    created_at: i64,
    updated_at: i64,
    creator_id: Option<u32>,
    approver_id: Option<u32>,
    author: String,
    change: u32,
    source: String,
    score: i32,
    md5: String,
    file_size: u64,
    file_ext: String,
    file_url: String,
    is_shown_in_index: bool,
    preview_url: String,
    sample_url: String,
    sample_width: Option<u32>,
    sample_height: Option<u32>,
    sample_file_size: Option<u64>,
    preview_width: Option<u32>,
    preview_height: Option<u32>,
    actual_preview_width: Option<u32>,
    actual_preview_height: Option<u32>,
    jpeg_url: String,
    jpeg_width: Option<u32>,
    jpeg_height: Option<u32>,
    jpeg_file_size: u64,
    rating: String,
    is_rating_locked: bool,
    has_children: bool,
    parent_id: Option<u32>,
    status: String,
    is_pending: bool,
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
    println!("Constructed API URL: {}", api_url);
    // 发送请求
    match client.get(&api_url)
        .header("Accept", "application/json")
        .send().await {
        Ok(response) => {
            if response.status().is_success() {
                let text = match response.text().await {
                    Ok(t) => t,
                    Err(e) => return Err(format!("Failed to read response text: {}", e)),
                };
                println!("Response length: {}", text.len());
                // Try to parse as JSON
                match serde_json::from_str::<Vec<Post>>(&text) {
                    Ok(posts) => Ok(posts),
                    Err(e) => {
                        // Print error details and a snippet around the error location
                        println!("Parse error: {}", e);
                        let column = e.column();
                        let start = if column > 50 { column - 50 } else { 0 };
                        let end = std::cmp::min(text.len(), column + 50);
                        let snippet = &text[start..end];
                        println!("Error around column {}: \"{}\"", column, snippet);
                        Err(format!("Failed to parse JSON: {}", e))
                    }
                }
            } else {
                Err(format!("Request failed with status: {}", response.status()))
            }
        },
        Err(e) => Err(format!("Failed to send request: {}", e)),
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
        .invoke_handler(tauri::generate_handler![greet, fetch_html, fetch_posts, fetch_image_as_base64])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
