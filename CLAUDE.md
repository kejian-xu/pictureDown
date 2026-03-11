# CLAUDE.md - PicureDown (yande.re图片抓取工具)

## 项目概述

PicureDown 是一个基于 Tauri + Vue 3 的桌面应用程序，专门用于从 yande.re 图片网站搜索和下载图片。应用通过调用 yande.re 的公开 API 获取图片数据，并提供友好的界面进行浏览和下载。

**主要功能：**
- 通过 yande.re API 搜索图片（支持标签过滤）
- 支持分页和数量限制控制
- 显示图片评分、分数、尺寸等元数据
- 智能图片URL选择（优先使用CDN链接）
- 多级错误回退机制（解决SSL证书问题）
- **点击缩略图浏览大图**（支持键盘导航：←/→切换，ESC关闭）
- **Base64图片缓存**（解决跨域和SSL证书问题）
- 响应式设计，支持暗色主题

## 技术栈

**前端：**
- Vue 3 + Composition API
- Vite 构建工具
- CSS3 + Flexbox 布局

**后端：**
- Rust + Tauri 2
- reqwest HTTP客户端
- serde JSON序列化

**核心依赖：**
```json
// package.json
"@tauri-apps/api": "^2"          // Tauri前端API
"vue": "^3.5.13"                 // Vue框架

// Cargo.toml
tauri = { version = "2" }        // Tauri核心
reqwest = { version = "0.11", features = ["json"] }   // HTTP客户端
serde = { version = "1", features = ["derive"] }      // JSON序列化
serde_json = "1"                 // JSON处理
serde-xml-rs = "0.6"             // XML解析
urlencoding = "2"                // URL编码
base64 = "0.21"                  // Base64编码解码
tauri-plugin-upload = "2.4.0"    // 文件上传插件
tauri-plugin-dialog = "2.6.0"    // 对话框插件
tauri-plugin-opener = "2"        // 打开外部链接插件
```

## 项目结构

```text
picureDown/
├── src/                          # 前端源代码
│   ├── main.js                   # Vue应用入口
│   └── App.vue                   # 主应用组件
├── src-tauri/                    # Rust后端
│   ├── Cargo.toml               # Rust依赖配置
│   ├── Cargo.lock               # 依赖锁文件
│   └── src/
│       ├── lib.rs               # 模块导出和入口
│       ├── commands.rs          # Tauri命令实现
│       └── models.rs            # 数据模型定义
├── public/                       # 静态资源
├── vite.config.js               # Vite配置
├── package.json                 # Node.js配置
└── index.html                   # HTML入口
```

## 运行指南

### 开发环境

```bash
# 安装前端依赖
npm install
# 或使用yarn
yarn install

# 开发模式运行
npm run tauri dev
# 或直接使用Tauri CLI
cargo tauri dev
```

### 生产构建

```bash
# 构建应用
npm run tauri build
# 构建结果在 src-tauri/target/release/
```

### 开发服务器（仅前端）

```bash
npm run dev      # 访问 http://localhost:1420
```

## 开发指南

### 模块结构
- **前端**：[src/App.vue](src/App.vue) - Vue 3 Composition API + Element Plus
- **后端模块**：
  - [src-tauri/src/lib.rs](src-tauri/src/lib.rs) - 模块导出和入口
  - [src-tauri/src/commands.rs](src-tauri/src/commands.rs) - Tauri命令实现
  - [src-tauri/src/models.rs](src-tauri/src/models.rs) - 数据模型定义

### 核心流程
1. **搜索图片**：前端调用 `fetch_posts` → Rust从 `https://yande.re/post.xml` 获取XML数据
2. **图片缓存**：前端维护内存缓存存储图片的Base64数据
3. **大图浏览**：点击缩略图触发Base64图片下载和缓存显示
4. **图片下载**：使用Tauri插件直接下载原图到本地下载目录

## API参考

### Rust后端命令

#### `fetch_posts(tags: Option<String>, limit: Option<u32>, page: Option<u32>) -> Result<Posts, String>`

从 yande.re 获取XML格式的图片数据。

**参数：**
- `tags`: 搜索标签（可选，多个标签用空格分隔）
- `limit`: 每页数量（可选，默认20，最大100）
- `page`: 页码（可选，默认1）

**返回：** `Posts` 结构体（包含count, offset, posts数组）

#### `fetch_image_as_base64(url: String) -> Result<String, String>`

下载图片并返回Base64编码字符串（忽略SSL证书错误）。

#### `fetch_html(url: &str) -> Result<String, String>`

抓取网页HTML内容（保留功能）。

### 核心数据结构
```rust
// XML响应结构
struct Posts {
    count: i32,     // 总数量
    offset: i32,    // 偏移量
    posts: Vec<Post> // 图片数组
}

// 图片数据
struct Post {
    id: u32,
    tags: String,           // 标签（用空格分隔）
    rating: String,         // 评分（s=安全, q=可疑, e=露骨）
    score: i32,             // 社区评分
    width: u32,             // 图片宽度
    height: u32,            // 图片高度
    file_url: String,       // 原图URL（用于下载）
    preview_url: String,    // 预览图URL（用于网格显示）
    sample_url: String,     // 样本图URL
    jpeg_url: String,       // JPEG格式URL
    md5: String,            // 文件MD5
    file_ext: String,       // 文件扩展名
    created_at: i64,        // 创建时间戳
}
```

## 配置说明

### 端口
- 前端开发服务器：1420

### SSL处理
- 后端配置 `danger_accept_invalid_certs(true)` 忽略无效证书
- 前端有多级回退机制处理SSL问题

## 故障排除

### 常见问题
1. **SSL证书错误**：后端忽略证书错误，前端有HTTPS→HTTP回退机制
2. **API请求失败**：检查网络连接和yande.re服务状态
3. **Rust编译错误**：使用 `cd src-tauri && cargo check` 检查依赖

### 调试位置
- Rust控制台：API URL、响应长度、解析错误
- 浏览器控制台：图片加载错误、回退尝试
- 前端日志：API返回的数据结构

## 开发建议

- **代码质量**：使用 `cargo fmt` 和 `cargo clippy` 保持Rust代码规范
- **功能扩展**：支持批量下载、本地收藏、高级搜索
- **性能优化**：实现图片懒加载、虚拟滚动
- **安全性**：用户输入验证、请求速率限制

## 相关链接

- [Tauri文档](https://tauri.app)
- [Vue 3文档](https://vuejs.org)
- [yande.re API参考](https://yande.re/help/api)

---

**最后更新**：2026-03-11
**更新内容**：精简CLAUDE.md文档，移除不必要细节

**项目状态**：功能完整，稳定运行
