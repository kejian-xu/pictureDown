# CLAUDE.md - PicureDown (yande.re图片抓取工具)

## 项目概述

PicureDown 是一个基于 Tauri + Vue 3 的桌面应用程序，专门用于从 yande.re 图片网站搜索和下载图片。应用通过调用 yande.re 的公开 API 获取图片数据，并提供友好的界面进行浏览和下载。

**主要功能：**
- 通过 yande.re API 搜索图片（支持标签过滤）
- 支持分页和数量限制控制
- 显示图片评分、分数、尺寸等元数据
- 智能图片URL选择（优先使用CDN链接）
- 多级错误回退机制（解决SSL证书问题）
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
reqwest = { version = "0.11" }   // HTTP客户端
serde = { version = "1" }        // JSON序列化
urlencoding = "2"                // URL编码
```

## 项目结构

```
picureDown/
├── src/                          # 前端源代码
│   ├── main.js                   # Vue应用入口
│   └── App.vue                   # 主应用组件
├── src-tauri/                    # Rust后端
│   ├── Cargo.toml               # Rust依赖配置
│   ├── Cargo.lock               # 依赖锁文件
│   └── src/
│       └── lib.rs               # Rust核心逻辑
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

### 前端开发 (Vue 3)
- **核心组件**：[src/App.vue](src/App.vue)
- **状态管理**：使用 Vue 3 Composition API (`ref`, `reactive`)
- **API调用**：通过 `@tauri-apps/api/core` 调用 Rust 命令
- **样式**：内联样式 + CSS类，支持暗色主题

### 后端开发 (Rust)
- **核心文件**：[src-tauri/src/lib.rs](src-tauri/src/lib.rs)
- **Tauri命令**：使用 `#[tauri::command]` 宏定义
- **HTTP请求**：通过 `reqwest` 库发送请求
- **错误处理**：使用 `Result<T, String>` 返回类型

### API调用流程
1. 前端调用 `invoke("fetch_posts", { tags, limit, page })`
2. Rust后端构建API URL：`https://yande.re/post.json?tags=...`
3. 发送HTTP请求并解析JSON响应
4. 返回 `Post` 结构体数组给前端
5. 前端转换数据并渲染图片网格

## API参考

### Rust后端命令

#### `fetch_posts(tags: Option<String>, limit: Option<u32>, page: Option<u32>) -> Result<Vec<Post>, String>`
从 yande.re 获取图片数据。

**参数：**
- `tags`: 搜索标签（可选，多个标签用空格分隔）
- `limit`: 每页数量（可选，默认20，最大100）
- `page`: 页码（可选，默认1）

**返回：** `Post` 结构体数组

#### `fetch_html(url: &str) -> Result<String, String>`
抓取网页HTML内容（保留功能，当前未使用）。

#### `greet(name: &str) -> String`
测试命令，返回问候语。

### Post 数据结构
```rust
struct Post {
    id: u32,
    tags: String,           // 标签（用空格分隔）
    rating: String,         // 评分（s=安全, q=可疑, e=露骨）
    score: i32,             // 社区评分
    width: u32,             // 图片宽度
    height: u32,            // 图片高度
    file_url: String,       // 原图URL
    preview_url: String,    // 预览图URL
    sample_url: String,     // 样本图URL（CDN）
    jpeg_url: String,       // JPEG格式URL
    // ... 其他字段
}
```

### 前端数据转换
前端将 `Post` 转换为以下格式：
```javascript
{
  src: post.preview_url,          // 显示用的图片URL
  alt: post.tags,                 // 标签作为alt文本
  rating: post.rating,           // 评分（s/q/e）
  score: post.score,             // 分数
  dimensions: `${width}x${height}`, // 尺寸
  fallbackUrl: post.file_url     // 备用URL（用于错误回退）
}
```

## 配置说明

### Tauri配置
- **端口**：1420（前端开发服务器）
- **SSL证书**：忽略无效证书（`danger_accept_invalid_certs(true)`）
- **用户代理**：模拟Chrome浏览器

### 前端配置
- **构建工具**：Vite 6
- **Vue插件**：@vitejs/plugin-vue
- **开发服务器**：严格端口模式（1420）

## 故障排除

### 常见问题

#### 1. SSL证书错误
```
Failed to load resource: An SSL error has occurred
```

**解决方案：**
- 应用已内置多级回退机制：
  1. 尝试备用URL（`file_url` → `preview_url` → `jpeg_url`）
  2. HTTPS降级为HTTP（`https://` → `http://`）
  3. 显示占位图片
- 后端配置了 `danger_accept_invalid_certs(true)` 忽略证书错误

#### 2. API请求失败
- 检查网络连接
- 确认 yande.re 服务可用性
- 查看控制台日志获取详细错误信息

#### 3. 图片加载缓慢
- 默认使用 `preview_url`（预览图，加载较快）
- 可考虑实现懒加载优化

#### 4. Rust编译错误
```bash
cd src-tauri && cargo check
```
- 检查依赖是否完整：`cargo build`
- 清理缓存：`cargo clean`

### 调试信息
应用在以下位置输出调试信息：
1. **Rust控制台**：API URL、响应长度、JSON解析错误
2. **浏览器控制台**：图片加载错误、回退尝试
3. **前端日志**：API返回的数据结构

## 开发建议

### 代码质量
- 使用 `cargo fmt` 和 `cargo clippy` 保持Rust代码规范
- Vue组件使用TypeScript增强类型安全（可选项）
- 添加单元测试和集成测试

### 功能扩展
1. **图片下载功能**：添加原图下载按钮
2. **本地收藏**：使用Tauri的本地存储保存收藏
3. **批量操作**：支持批量下载或导出
4. **高级搜索**：添加评分过滤、排序选项
5. **缩略图缓存**：使用IndexedDB缓存图片

### 性能优化
- 实现图片懒加载
- 添加虚拟滚动支持大量图片
- 使用Web Workers处理图片处理任务
- 优化CSS选择器性能

### 安全性
- 考虑实现请求速率限制
- 添加图片内容安全检查
- 用户输入验证和清理
- 敏感信息不硬编码在代码中

## 相关链接

- [Tauri文档](https://tauri.app)
- [Vue 3文档](https://vuejs.org)
- [yande.re API参考](https://yande.re/help/api)
- [Rust编程语言](https://www.rust-lang.org)

---

**最后更新**：2026-03-02
**项目状态**：开发中
**主要维护者**：项目开发者