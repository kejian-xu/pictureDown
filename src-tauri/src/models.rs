use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    pub posts: Vec<Value>, // 可以是任何结构
}
