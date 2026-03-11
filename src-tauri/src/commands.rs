use crate::Posts;
use std::collections::HashMap;


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
    match client
        .get(url)
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
        )
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Connection", "keep-alive")
        .header("Upgrade-Insecure-Requests", "1")
        .header("Referer", "https://www.google.com/")
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.text().await {
                    Ok(html) => Ok(html),
                    Err(e) => Err(format!("Failed to read response text: {}", e)),
                }
            } else {
                Err(format!("Request failed with status: {}", response.status()))
            }
        }
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

    let posts: Posts = serde_xml_rs::from_str(&xml)
        .map_err(|e| format!("XML parse error: {}", e))?;
    Ok(posts)
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

    match client.get(&url).header("Accept", "image/*").send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.bytes().await {
                    Ok(bytes) => {
                        let base64_string = base64::encode(&bytes);
                        println!(
                            "Image fetched successfully, size: {} bytes, base64 length: {}",
                            bytes.len(),
                            base64_string.len()
                        );
                        Ok(base64_string)
                    }
                    Err(e) => Err(format!("Failed to read image bytes: {}", e)),
                }
            } else {
                Err(format!("Request failed with status: {}", response.status()))
            }
        }
        Err(e) => Err(format!("Failed to send request: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_upload::init())
        .invoke_handler(tauri::generate_handler![fetch_html, fetch_posts, fetch_image_as_base64])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
