use crate::Posts;
use tauri_plugin_fs::FsExt;


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
    source: Option<String>,
    login: Option<String>,
    api_key: Option<String>,
) -> Result<Posts, String> {
    let base_url = source.unwrap_or_else(|| "yande.re".to_string());

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .cookie_store(true)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create client: {}", e))?;

    let api_url = build_api_url(&base_url, &tags, limit, page, &login, &api_key);
    println!("api_url: {}", api_url);

    if base_url.contains("donmai") || base_url.contains("gelbooru") {
        fetch_json_posts(&client, &api_url, &base_url, &tags).await
    } else {
        fetch_xml_posts(&client, &api_url, &base_url).await
    }
}

// ---- 子函数 ----

/// 构建 API URL（Moebooru / Danbooru / Gelbooru）
fn build_api_url(
    base_url: &str,
    tags: &Option<String>,
    limit: Option<u32>,
    page: Option<u32>,
    login: &Option<String>,
    api_key: &Option<String>,
) -> String {
    if base_url.contains("gelbooru") {
        let pid = page.unwrap_or(1).saturating_sub(1);
        let mut url = format!(
            "https://{}/index.php?page=dapi&s=post&q=index&json=1&pid={}&limit={}&tags={}",
            base_url, pid, limit.unwrap_or(100), tags.as_deref().unwrap_or("")
        );
        if let (Some(ref uid), Some(ref key)) = (login, api_key) {
            url.push_str(&format!("&user_id={}&api_key={}", uid, key));
        }
        url
    } else {
        let is_danbooru = base_url.contains("donmai");
        let endpoint = if is_danbooru { "posts.json" } else { "post.xml" };
        let mut url = format!("https://{}/{}", base_url, endpoint);
        let mut params: Vec<String> = Vec::new();
        if let Some(p) = page { params.push(format!("page={}", p)); }
        if let Some(l) = limit { params.push(format!("limit={}", l)); }
        if let Some(ref t) = tags { params.push(format!("tags={}", t)); }
        if is_danbooru {
            if let Some(ref l) = login { params.push(format!("login={}", l)); }
            if let Some(ref k) = api_key { params.push(format!("api_key={}", k)); }
        }
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }
        url
    }
}

/// 请求 JSON API（Danbooru / Gelbooru）并解析
async fn fetch_json_posts(
    client: &reqwest::Client,
    api_url: &str,
    base_url: &str,
    tags: &Option<String>,
) -> Result<Posts, String> {
    let is_danbooru = base_url.contains("donmai");
    let ua = if is_danbooru {
        "PictureDown/1.0 (xukejian11)"
    } else {
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"
    };

    let response = client.get(api_url)
        .header("User-Agent", ua)
        .header("Accept", "application/json")
        .header("Referer", &format!("https://{}/", base_url))
        .send().await.map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("Request failed: {}", response.status()));
    }

    let json: serde_json::Value = response.json().await
        .map_err(|e| format!("JSON parse error: {}", e))?;

    let raw_posts: Vec<serde_json::Value> = if let Some(arr) = json.as_array() {
        arr.clone()
    } else if let Some(arr) = json["post"].as_array() {
        arr.clone()
    } else {
        return Err("Unexpected JSON format".to_string());
    };

    let posts: Vec<serde_json::Value> = raw_posts.into_iter().map(|mut p| {
        if let Some(obj) = p.as_object_mut() {
            normalize_fields(obj, is_danbooru);
        }
        p
    }).collect();

    let count = if is_danbooru {
        fetch_danbooru_count(client, base_url, tags).await
    } else {
        json["@attributes"]["count"].as_i64().unwrap_or(posts.len() as i64) as i32
    };

    Ok(Posts { count, offset: 0, posts })
}

/// 规范化 JSON 字段名 → Moebooru 兼容格式
fn normalize_fields(obj: &mut serde_json::Map<String, serde_json::Value>, is_danbooru: bool) {
    if is_danbooru {
        rename_field(obj, "preview_file_url", "preview_url");
        rename_field(obj, "large_file_url", "sample_url");
        rename_field(obj, "tag_string", "tags");
        rename_field(obj, "image_width", "width");
        rename_field(obj, "image_height", "height");
    }
    if let Some(serde_json::Value::String(ts)) = obj.get("created_at").cloned() {
        let timestamp = chrono::DateTime::parse_from_rfc3339(&ts)
            .map(|dt| dt.timestamp())
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&ts, "%a %b %d %H:%M:%S %z %Y")
                    .map(|dt| dt.and_utc().timestamp())
            })
            .unwrap_or(0);
        obj.insert("created_at".to_string(), serde_json::Value::Number(timestamp.into()));
    }
}

/// 获取 Danbooru 精确总数
async fn fetch_danbooru_count(
    client: &reqwest::Client,
    base_url: &str,
    tags: &Option<String>,
) -> i32 {
    let mut url = format!("https://{}/counts/posts.json", base_url);
    if let Some(ref t) = tags {
        url.push_str(&format!("?tags={}", t));
    }
    let result = client.get(&url)
        .header("User-Agent", "PictureDown/1.0 (xukejian11)")
        .header("Accept", "application/json")
        .send()
        .await;
    match result {
        Ok(resp) if resp.status().is_success() => {
            let json: serde_json::Value = resp.json().await.unwrap_or_default();
            json["counts"]["posts"].as_i64().unwrap_or(0) as i32
        }
        _ => 0,
    }
}

/// 请求 XML API（Moebooru）并解析
async fn fetch_xml_posts(
    client: &reqwest::Client,
    api_url: &str,
    base_url: &str,
) -> Result<Posts, String> {
    let response = client.get(api_url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .header("Accept", "application/xml, text/xml, */*;q=0.9")
        .header("Accept-Language", "en-US,en;q=0.9,zh-CN;q=0.8,zh;q=0.7")
        .header("Referer", &format!("https://{}/", base_url))
        .send().await.map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("Request failed: {}", response.status()));
    }

    let xml = response.text().await.map_err(|e| e.to_string())?;
    let xml = xml.strip_prefix('\u{FEFF}').unwrap_or(&xml);
    serde_xml_rs::from_str(xml).map_err(|e| format!("XML parse error: {}", e))
}

fn rename_field(obj: &mut serde_json::Map<String, serde_json::Value>, from: &str, to: &str) {
    if let Some(val) = obj.remove(from) {
        obj.insert(to.to_string(), val);
    }
}


#[tauri::command]
async fn fetch_image_as_bytes(url: String) -> Result<Vec<u8>, String> {
    // 创建一个基本的客户端配置
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36")
        .danger_accept_invalid_certs(true) // 忽略证书错误
        .build()
        .map_err(|e| format!("Failed to create client: {}", e))?;

 

    match client.get(&url).header("Accept", "image/*").send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.bytes().await {
                    Ok(bytes) => {
                        Ok(bytes.to_vec())
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

#[tauri::command]
async fn fetch_image_to_local(url: String, cache_dir: String, filename: String) -> Result<String, String> {
    let file_path = std::path::Path::new(&cache_dir).join(&filename);

    // 如果本地已缓存，直接返回路径
    if file_path.exists() {
        return Ok(file_path.to_string_lossy().to_string());
    }

    // 确保缓存目录存在
    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache dir: {}", e))?;

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36")
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("Failed to create client: {}", e))?;

    let response = client
        .get(&url)
        .header("Accept", "image/*")
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Request failed with status: {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read image bytes: {}", e))?;

    std::fs::write(&file_path, &bytes)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn fetch_image_as_base64(url: String) -> Result<String, String> {
    // 创建一个基本的客户端配置
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36")
        .danger_accept_invalid_certs(true) // 忽略证书错误
        .build()
        .map_err(|e| format!("Failed to create client: {}", e))?;

    match client.get(&url).header("Accept", "image/*").send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.bytes().await {
                    Ok(bytes) => {
                        let base64_string = base64::encode(&bytes);
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


#[tauri::command]
async fn grant_path_access(app_handle: tauri::AppHandle, path: String) -> Result<(), String> {
    // 获取 fs 插件的 scope
    let fs_scope = app_handle.fs_scope();
    
    // 动态添加目录访问权限（递归允许所有子文件）
    fs_scope
        .allow_directory(&std::path::Path::new(&path), true)
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_upload::init())
        .plugin(tauri_plugin_cache::init())
        .invoke_handler(tauri::generate_handler![fetch_html, fetch_posts, fetch_image_as_base64, fetch_image_as_bytes, fetch_image_to_local, grant_path_access])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
