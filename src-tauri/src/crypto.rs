use bcrypt::{hash, verify, BcryptError, DEFAULT_COST};
use sha1::{Sha1, Digest};
use anyhow::{Result, anyhow};

/// Danbooru 密码哈希器
pub struct DanbooruHasher {
    fixed_salt: String,
    bcrypt_cost: u32,
}

impl Default for DanbooruHasher {
    fn default() -> Self {
        Self {
            fixed_salt: "choujin-steiner".to_string(),
            bcrypt_cost: DEFAULT_COST,
        }
    }
}

impl DanbooruHasher {
    /// 创建新的哈希器实例
    pub fn new(fixed_salt: impl Into<String>, bcrypt_cost: u32) -> Self {
        Self {
            fixed_salt: fixed_salt.into(),
            bcrypt_cost,
        }
    }
    
    /// 计算中间 SHA1 值（内部使用）
    fn compute_sha1(&self, password: &str) -> String {
        // 格式: "固定盐--密码--"
        let salted = format!("{}--{}--", self.fixed_salt, password);
        
        let mut hasher = Sha1::new();
        hasher.update(salted.as_bytes());
        let result = hasher.finalize();
        
        // 转换为十六进制字符串
        hex::encode(result)
    }
    
    /// 哈希密码（完整的 Danbooru 风格）
    pub fn hash_password(&self, password: &str) -> Result<String> {
        let sha1_hex = self.compute_sha1(password);
        
        // 使用 bcrypt 进行最终哈希
        hash(sha1_hex, self.bcrypt_cost)
            .map_err(|e| anyhow!("BCrypt 加密失败: {}", e))
    }
    
    /// 验证密码
    pub fn verify_password(&self, password: &str, hashed: &str) -> Result<bool> {
        let sha1_hex = self.compute_sha1(password);
        
        verify(sha1_hex, hashed)
            .map_err(|e| anyhow!("密码验证失败: {}", e))
    }
    
    /// 直接从密码生成哈希（使用默认配置）
    pub fn hash_default(password: &str) -> Result<String> {
        let hasher = DanbooruHasher::default();
        hasher.hash_password(password)
    }
    
    /// 验证密码（使用默认配置）
    pub fn verify_default(password: &str, hashed: &str) -> Result<bool> {
        let hasher = DanbooruHasher::default();
        hasher.verify_password(password, hashed)
    }
    pub fn danbooru_api_password_hash(password: &str) -> String {
      let salted = format!("choujin-steiner--{}--", password);
      let mut hasher = Sha1::new();
      hasher.update(salted.as_bytes());
      format!("{:x}", hasher.finalize())  // 直接返回 SHA1 十六进制字符串
  }
}

// 为了方便，也可以提供独立的函数

/// Danbooru 风格密码加密（快捷函数）
pub fn danbooru_hash(password: &str) -> String {
    DanbooruHasher::danbooru_api_password_hash(password)
}

/// Danbooru 风格密码验证（快捷函数）
pub fn danbooru_verify(password: &str, hashed: &str) -> Result<bool> {
    DanbooruHasher::verify_default(password, hashed)
}