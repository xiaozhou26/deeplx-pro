# deepl2api

DeepL 非官方翻译 API —— 将 DeepL Chrome 扩展的内部翻译接口包装为标准 REST API，无需 DeepL API Key 即可调用。

## 快速开始

```bash
# 运行服务（默认监听 127.0.0.1:9000）
cargo run --release
```

```bash
# 翻译文本
curl 'http://127.0.0.1:9000/translate' \
  -H 'Content-Type: application/json' \
  -d '{"text":"Hello, world!","source_lang":"EN","target_lang":"ZH"}'

# 响应
{"code":200,"data":"你好，世界！","source_lang":"EN"}
```

## 功能特点

- **无需 API Key** —— 模拟 DeepL Chrome 扩展的行为，通过 DeepL 公开网页接口翻译
- **官方 API 兼容** —— `POST /v2/translate` 响应格式与 DeepL 官方 API 完全一致，现有官方 SDK 改 `base_url` 即可接入
- **批量翻译** —— `/v2/translate` 的 `text` 字段支持单条字符串或数组（最多 50 条）
- **长文本自动分片** —— 超过 1500 字符时服务端按句子/段落切分、顺序翻译、自动拼接，对调用方完全透明
- **自动重试 + Cookie 刷新** —— 429 或网络错误时自动重试 1 次，并重新预热 Cookie
- **Pro 端点支持** —— 设置 `DEEPL_DL_SESSION` 即启用 Pro 端点（`oneshot-pro`）
- **语言元数据接口** —— `GET /v2/languages` 返回支持的语言及 `supports_formality` 标记
- **健康检查** —— `GET /health` 供容器/监控系统探活
- **语言自动检测** —— 设置 `source_lang` 为 `"auto"` 或留空即可
- **代理支持** —— 通过 `PROXY_LIST` 环境变量配置 HTTP/SOCKS 代理
- **CORS 全开** —— 可在浏览器环境中直接调用

## 安装

### 前置依赖

- [Rust](https://www.rust-lang.org/) 2021 edition 或更新版本

### 构建

```bash
git clone https://github.com/your-username/deepl2api.git
cd deepl2api
cargo build --release
```

编译后的二进制文件位于 `target/release/deepl2api.exe`。

## 配置

### 环境变量

| 变量 | 说明 | 默认值 |
|---|---|---|
| `PROXY_LIST` | 代理地址，支持 `http://` / `https://` / `socks4://` / `socks4a://` / `socks5://` / `socks5h://` | 无（直连） |
| `DEEPL_DL_SESSION` | Pro 会话 token。非空时启用 Pro 端点（`oneshot-pro`），否则使用免费端点 | 无（免费端点） |
| `HOST` | 监听地址 | `127.0.0.1` |
| `PORT` | 监听端口 | `9000` |

### 服务器地址

服务硬编码监听 `127.0.0.1:9000`，如需更改请修改 `src/main.rs` 中的 `addr` 变量。

## 项目架构

```
deepl2api/
├── src/
│   └── main.rs          # 全部代码（~380行，单文件）
├── Cargo.toml           # 依赖及元信息
├── Cargo.lock
├── CLAUDE.md            # Claude Code 项目指南
└── API.md               # API 文档
```

代码结构一览：

1. **DeepLClient** —— 核心翻译客户端
   - 启动时访问 `https://www.deepl.com/translator` 预热 Cookie
   - 内部维护带 Cookie 存储的 HTTP 客户端
   - 支持免费（`oneshot-free`）和 Pro（`oneshot-pro`）两个端点，由 `DEEPL_DL_SESSION` 决定
   - 翻译分三层：`translate_single`（单条，≤1500）→ `translate_with_retry`（重试 1 次 + Cookie 刷新）→ `translate_chunked`（长文本分片）

2. **长文本分片** —— 超过 1500 字符自动切分
   - 切分优先级：空行 `\n\n` > 换行 `\n` > 句末标点 `。！？!?．` > 硬切
   - 分片顺序翻译后拼接，避免并发打爆 429

3. **请求伪装** —— 模拟 DeepL Chrome 扩展身份
   - 请求头携带 `Origin: chrome-extension://...`
   - 请求体包含 `app_information`（操作系统、应用版本、实例 ID 等）
   - 使用随机生成的 UUIDv4 作为实例标识

4. **语言表** —— 静态结构数组（语言代码 / 名称 / 内部代码 / 是否支持 formality）
   - 支持目标语言 36 种
   - 自动别名：`EN` → `EN-US`，`PT` → `PT-BR`，`ZH` → `ZH-HANS`

5. **Axum Web 服务器** —— 4 个路由
   - `POST /translate`（旧便捷接口）、`POST /v2/translate`（官方兼容）、`GET /v2/languages`、`GET /health`
   - 全开 CORS（允许任意来源、方法、请求头）
   - 错误映射为合理 HTTP 状态码

## 限制

- 单次翻译文本最长 **1500 个字符**（超出会自动分片；`/v2/translate` 最多 50 条文本）
- 出站 HTTP 请求超时 **20 秒**
- 服务端无认证、无速率限制（依赖 DeepL 端限制）
- 默认使用免费端点（`oneshot-free`）；设置 `DEEPL_DL_SESSION` 启用 Pro 端点

## 开发

```bash
# 构建
cargo build

# 运行（开发模式）
cargo run

# 使用代理
$env:PROXY_LIST="http://127.0.0.1:7890"; cargo run
```

## 许可

本项目仅供学习研究目的。使用 DeepL 服务请遵守 [DeepL 服务条款](https://www.deepl.com/pro-license)。
