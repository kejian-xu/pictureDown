use crate::Posts;
use crate::models::{ComicPost, ComicPosts, ComicParseConfig, DetailParseConfig};
use tauri_plugin_fs::FsExt;
use scraper::{Html, Selector};
use urlencoding;


#[tauri::command]
async fn fetch_html(url: &str) -> Result<String, String> {
    // 创建客户端，强制HTTP/1.1 + 忽略证书 + rustls TLS
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36")
        .danger_accept_invalid_certs(true)
        .http1_only()
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

/// 解析漫画站HTML，提取图片信息
/// 通过 config 中的 CSS 选择器配置适配不同的漫画站
#[tauri::command]
fn parse_comic_html(html: &str, base_url: Option<String>, config: Option<ComicParseConfig>) -> Result<ComicPosts, String> {
    let base = base_url.unwrap_or_else(|| "https://www.177pica.com".to_string());
    let cfg = config.unwrap_or_default();
    let document = Html::parse_document(html);

    let item_selector = Selector::parse(&cfg.item_selector)
        .map_err(|e| format!("item_selector 解析失败: {}", e))?;
    let link_selector = Selector::parse(&cfg.link_selector)
        .map_err(|e| format!("link_selector 解析失败: {}", e))?;
    let img_selector = Selector::parse(&cfg.img_selector)
        .map_err(|e| format!("img_selector 解析失败: {}", e))?;

    let mut posts: Vec<ComicPost> = Vec::new();

    for item in document.select(&item_selector) {
        for a in item.select(&link_selector) {
            if let Some(img) = a.select(&img_selector).next() {
                let href = match a.value().attr(&cfg.link_attr) {
                    Some(h) => h,
                    None => continue,
                };
                let src = match img.value().attr(&cfg.thumb_attr) {
                    Some(s) => s,
                    None => continue,
                };
                let title = img.value().attr(&cfg.title_attr).unwrap_or("").to_string();

                let post_url = resolve_url(href, &base);
                let thumb_url = src.to_string();

                // 根据配置决定原图URL提取方式
                let original_url = match cfg.original_from_thumb.as_str() {
                    "timthumb" => extract_original_url(&thumb_url)
                        .unwrap_or_else(|| thumb_url.clone()),
                    _ => thumb_url.clone(), // "direct": 缩略图即原图
                };

                // 提取文件扩展名
                let file_ext = original_url
                    .split('?')
                    .next()
                    .and_then(|u| u.rsplit('.').next())
                    .unwrap_or("jpg")
                    .to_string();

                // 根据配置决定UID来源
                let uid_source = match cfg.uid_from.as_str() {
                    "original_url" => &original_url,
                    _ => &post_url, // 默认从post_url提取
                };
                let uid = url_to_uid(uid_source).unwrap_or_else(|| {
                    use std::hash::Hasher;
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    std::hash::Hash::hash(uid_source, &mut hasher);
                    format!("{:x}", hasher.finish())
                });

                posts.push(ComicPost {
                    thumb_url,
                    original_url,
                    post_url,
                    title,
                    file_ext,
                    uid,
                });
            }
        }
    }

    let count = posts.len() as i32;
    Ok(ComicPosts { posts, count })
}

/// 拼接相对URL为绝对URL
fn resolve_url(href: &str, base: &str) -> String {
    if href.starts_with("http") {
        href.to_string()
    } else if href.starts_with('/') {
        format!("{}{}", base, href)
    } else {
        format!("{}/{}", base, href)
    }
}

/// 解析详情页HTML，直接提取所有图片URL
/// 适用于 177pica.com 详情页等直接在 <p> 中包含 <img> 的页面
#[tauri::command]
fn parse_detail_images(html: &str, config: Option<DetailParseConfig>) -> Result<ComicPosts, String> {
    let cfg = config.unwrap_or_default();
    let document = Html::parse_document(html);

    let img_selector = Selector::parse(&cfg.img_selector)
        .map_err(|e| format!("img_selector 解析失败: {}", e))?;

    let mut posts: Vec<ComicPost> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for img in document.select(&img_selector) {
        // 按优先级依次尝试 src_attr 中配置的属性（用 | 分隔），取第一个有效URL
        let src = cfg.src_attr
            .split('|')
            .map(|a| a.trim())
            .find_map(|attr| {
                img.value().attr(attr)
                    .filter(|s| !s.is_empty() && !s.starts_with("data:"))
            });

        let src = match src {
            Some(s) => s,
            None => continue,
        };

        // URL过滤
        if !cfg.url_filter.is_empty() && !src.contains(&cfg.url_filter) {
            continue;
        }

        // 去重
        if cfg.deduplicate {
            if seen.contains(src) {
                continue;
            }
            seen.insert(src.to_string());
        }

        let file_url = src.to_string();

        let file_ext = file_url
            .split('?')
            .next()
            .and_then(|u| u.rsplit('.').next())
            .unwrap_or("jpg")
            .to_string();

        let uid = url_to_uid(&file_url).unwrap_or_else(|| {
            use std::hash::Hasher;
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(&file_url, &mut hasher);
            format!("{:x}", hasher.finish())
        });

        posts.push(ComicPost {
            thumb_url: file_url.clone(),
            original_url: file_url,
            post_url: String::new(),
            title: String::new(),
            file_ext,
            uid,
        });
    }

    let count = posts.len() as i32;
    Ok(ComicPosts { posts, count })
}

/// 从timthumb.php的URL中提取原图URL
/// 例如: https://...timthumb.php?src=http://img.177pica.com/uploads/2026/02a/004640-8.jpg&w=280&h=210
/// 提取: http://img.177pica.com/uploads/2026/02a/004640-8.jpg
fn extract_original_url(thumb_url: &str) -> Option<String> {
    if thumb_url.contains("timthumb.php") {
        // 解析URL参数
        if let Some(query_start) = thumb_url.find('?') {
            let query_str = &thumb_url[query_start + 1..];
            for param in query_str.split('&') {
                if let Some(eq_pos) = param.find('=') {
                    let key = &param[..eq_pos];
                    let value = &param[eq_pos + 1..];
                    if key == "src" && !value.is_empty() {
                        // URL解码并处理HTML实体
                        let decoded = urlencoding::decode(value)
                            .map(|cow| cow.into_owned())
                            .unwrap_or_else(|_| value.to_string());
                        return Some(decoded);
                    }
                }
            }
        }
    }
    None
}

/// 从URL中提取唯一标识（如文章ID）
fn url_to_uid(url: &str) -> Option<String> {
    // 尝试从URL中提取数字ID
    // 例如: https://www.177pica.com/html/2026/07/8615523.html → 8615523
    let without_query = url.split('?').next()?;
    let filename = without_query.rsplit('/').next()?;
    let stem = filename.rsplit('.').nth(1)?;
    if stem.chars().all(|c| c.is_ascii_digit()) && stem.len() >= 3 {
        Some(stem.to_string())
    } else {
        None
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
        .http1_only()
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
    if base_url.contains("rule34") {
        // rule34 用 rating:explicit/questionable/safe 而非 rating:e/q/s
        let tags_str = tags.as_deref().unwrap_or("")
            .replace("rating:e", "rating:explicit")
            .replace("rating:q", "rating:questionable")
            .replace("rating:s", "rating:general");
        let mut url = format!(
            "https://{}/index.php?page=dapi&s=post&q=index&pid={}&limit={}&tags={}",
            base_url, page.unwrap_or(1).saturating_sub(1), limit.unwrap_or(100), tags_str
        );
        if let (Some(ref uid), Some(ref key)) = (login, api_key) {
            url.push_str(&format!("&user_id={}&api_key={}", uid, key));
        }
        url
    } else if base_url.contains("gelbooru") {
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
    } else if base_url.contains("rule34") || base_url.contains("gelbooru") {
        json["count"].as_i64().unwrap_or(posts.len() as i64) as i32
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
        if let Some(t) = parse_date_string(&ts) {
            obj.insert("created_at".to_string(), serde_json::Value::Number(t.into()));
        }
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
    let mut posts: Posts = serde_xml_rs::from_str(xml).map_err(|e| format!("XML parse error: {}", e))?;
    // 修复 created_at：日期字符串 → Unix 时间戳（rule34 等站返回日期字符串）
    for p in &mut posts.posts {
        if let Some(obj) = p.as_object_mut() {
            if let Some(serde_json::Value::String(ts)) = obj.get("created_at").cloned() {
                if let Some(t) = parse_date_string(&ts) {
                    obj.insert("created_at".to_string(), serde_json::Value::Number(t.into()));
                }
            }
        }
    }
    Ok(posts)
}

fn parse_date_string(ts: &str) -> Option<i64> {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
        return Some(dt.timestamp());
    }
    if let Ok(dt) = chrono::DateTime::parse_from_str(ts, "%a %b %d %H:%M:%S %z %Y") {
        return Some(dt.timestamp());
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S") {
        return Some(dt.and_utc().timestamp());
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(ts, "%Y-%m-%d") {
        if let Some(dt) = d.and_hms_opt(0, 0, 0) {
            return Some(dt.and_utc().timestamp());
        }
    }
    None
}

fn rename_field(obj: &mut serde_json::Map<String, serde_json::Value>, from: &str, to: &str) {
    if let Some(val) = obj.remove(from) {
        obj.insert(to.to_string(), val);
    }
}


/// 根据图片 URL 返回合适的 Referer 头
fn get_ua_for_image(url: &str) -> &str {
    if url.contains("donmai") {
        "PictureDown/1.0 (xukejian11)"
    } else {
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36"
    }
}

fn get_referer_for_image(url: &str) -> &str {
    if url.contains("donmai") || url.contains("cdn.donmai") {
        "https://danbooru.donmai.us/"
    } else if url.contains("gelbooru") || url.contains("img3.gelbooru") {
        "https://gelbooru.com/"
    } else if url.contains("rule34") {
        "https://rule34.xxx/"
    } else if url.contains("img.177pica") || url.contains("177pica.com") {
        "https://www.177pica.com/"
    } else {
        ""
    }
}

#[tauri::command]
async fn fetch_image_as_bytes(url: String) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .http1_only()
        .build()
        .map_err(|e| format!("Failed to create client: {}", e))?;

    let ua = get_ua_for_image(&url);
    let referer = get_referer_for_image(&url);
    let mut req = client.get(&url).header("Accept", "image/*").header("User-Agent", ua);
    if !referer.is_empty() {
        req = req.header("Referer", referer);
    }

    match req.send().await {
        Ok(response) => {
            if response.status().is_success() {
                response.bytes().await.map(|b| b.to_vec()).map_err(|e| format!("Failed to read image bytes: {}", e))
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

    if file_path.exists() {
        return Ok(file_path.to_string_lossy().to_string());
    }

    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache dir: {}", e))?;

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .http1_only()
        .build()
        .map_err(|e| format!("Failed to create client: {}", e))?;

    let ua = get_ua_for_image(&url);
    let referer = get_referer_for_image(&url);
    let mut req = client.get(&url).header("Accept", "image/*").header("User-Agent", ua);
    if !referer.is_empty() {
        req = req.header("Referer", referer);
    }

    let response = req.send().await.map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Request failed with status: {}", response.status()));
    }

    let bytes = response.bytes().await.map_err(|e| format!("Failed to read image bytes: {}", e))?;
    std::fs::write(&file_path, &bytes).map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn fetch_image_as_base64(url: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .http1_only()
        .build()
        .map_err(|e| format!("Failed to create client: {}", e))?;

    let ua = get_ua_for_image(&url);
    let referer = get_referer_for_image(&url);
    let mut req = client.get(&url).header("Accept", "image/*").header("User-Agent", ua);
    if !referer.is_empty() {
        req = req.header("Referer", referer);
    }

    match req.send().await {
        Ok(response) => {
            if response.status().is_success() {
                response.bytes().await
                    .map(|b| base64::encode(&b))
                    .map_err(|e| format!("Failed to read image bytes: {}", e))
            } else {
                Err(format!("Request failed with status: {}", response.status()))
            }
        }
        Err(e) => Err(format!("Failed to send request: {}", e)),
    }
}


#[tauri::command]
async fn download_file(url: String, save_path: String) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .http1_only()
        .build()
        .map_err(|e| format!("Failed to create client: {}", e))?;

    let ua = get_ua_for_image(&url);
    let referer = get_referer_for_image(&url);
    let mut req = client.get(&url).header("Accept", "image/*").header("User-Agent", ua);
    if !referer.is_empty() {
        req = req.header("Referer", referer);
    }

    let response = req.send().await.map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Request failed with status: {}", response.status()));
    }

    let bytes = response.bytes().await.map_err(|e| format!("Failed to read bytes: {}", e))?;

    if let Some(parent) = std::path::Path::new(&save_path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
    }
    std::fs::write(&save_path, &bytes).map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(())
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
        .invoke_handler(tauri::generate_handler![fetch_html, parse_comic_html, parse_detail_images, fetch_posts, fetch_image_as_base64, fetch_image_as_bytes, fetch_image_to_local, download_file, grant_path_access])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
