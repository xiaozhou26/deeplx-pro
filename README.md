# deeplx-pro

DeepL 非官方翻译 API。项目模拟 DeepL Chrome 扩展的内部翻译请求，并将其包装成简单 REST API，无需 DeepL API Key。

仓库：<https://github.com/xiaozhou26/deeplx-pro>

## 功能

- `POST /translate`：简单单文本翻译接口
- `POST /v2/translate`：兼容 DeepL 官方 API 的翻译接口
- `GET /v2/languages`：查询源语言或目标语言列表
- `GET /health`：容器和监控健康检查
- 单请求最多 50 条文本
- 超过 1500 字符时自动按自然边界分片并顺序翻译
- 429 和网络错误自动刷新 Cookie 并重试一次
- 支持 HTTP、HTTPS、SOCKS4、SOCKS4A、SOCKS5 和 SOCKS5H 代理
- 所有翻译请求固定使用 `https://oneshot-free.www.deepl.com/v1/translate`，无需账号 Cookie
- 可通过 `API_KEY` 保护翻译接口

## Docker 部署

镜像发布到 GitHub Container Registry（GHCR）：

- `ghcr.io/xiaozhou26/deeplx-pro:latest`

支持 `linux/amd64` 和 `linux/arm64`。

```bash
docker pull ghcr.io/xiaozhou26/deeplx-pro:latest

docker run -d \
  --name deeplx-pro \
  --restart unless-stopped \
  -p 9000:9000 \
  ghcr.io/xiaozhou26/deeplx-pro:latest
```

容器镜像默认设置 `HOST=0.0.0.0` 和 `PORT=9000`，因此可以直接通过宿主机映射端口访问。

### 使用代理

```bash
docker run -d \
  --name deeplx-pro \
  --restart unless-stopped \
  -p 9000:9000 \
  -e PROXY_LIST=socks5h://host.docker.internal:1080 \
  ghcr.io/xiaozhou26/deeplx-pro:latest
```

### 使用 API 鉴权

```bash
docker run -d \
  --name deeplx-pro \
  --restart unless-stopped \
  -p 9000:9000 \
  -e API_KEY=change-me \
  ghcr.io/xiaozhou26/deeplx-pro:latest
```

### Docker Compose

```yaml
services:
  deeplx-pro:
    image: ghcr.io/xiaozhou26/deeplx-pro:latest
    container_name: deeplx-pro
    restart: unless-stopped
    ports:
      - "9000:9000"
    environment:
      HOST: 0.0.0.0
      PORT: 9000
      # PROXY_LIST: socks5h://host.docker.internal:1080
      # API_KEY: change-me
```

启动并检查状态：

```bash
docker compose up -d
curl http://127.0.0.1:9000/health
```

预期响应：

```json
{"status":"ok"}
```

## GitHub Release 使用

Release 页面：<https://github.com/xiaozhou26/deeplx-pro/releases>

每个版本提供以下文件：

| 平台 | 文件 |
|---|---|
| Linux x86-64 | `deeplx-pro-linux-amd64.tar.gz` |
| Windows x86-64 | `deeplx-pro-windows-amd64.zip` |

### Linux

```bash
VERSION=v1.1.1
curl -LO "https://github.com/xiaozhou26/deeplx-pro/releases/download/${VERSION}/deeplx-pro-linux-amd64.tar.gz"
tar -xzf deeplx-pro-linux-amd64.tar.gz
chmod +x deeplx-pro
HOST=0.0.0.0 PORT=9000 ./deeplx-pro
```

### Windows PowerShell

```powershell
$Version = "v1.1.1"
Invoke-WebRequest `
  -Uri "https://github.com/xiaozhou26/deeplx-pro/releases/download/$Version/deeplx-pro-windows-amd64.zip" `
  -OutFile "deeplx-pro-windows-amd64.zip"
Expand-Archive .\deeplx-pro-windows-amd64.zip -DestinationPath .\deeplx-pro
$env:HOST = "0.0.0.0"
$env:PORT = "9000"
.\deeplx-pro\deeplx-pro.exe
```

也可以从 Release 页面手动下载对应压缩包。程序默认监听 `127.0.0.1:9000`；需要让局域网或容器外部访问时，将 `HOST` 设置为 `0.0.0.0`。

## 源码构建

需要安装 Rust stable 工具链。

```bash
git clone https://github.com/xiaozhou26/deeplx-pro.git
cd deeplx-pro
cargo build --release
```

生成文件：

- Linux/macOS：`target/release/deeplx-pro`
- Windows：`target/release/deeplx-pro.exe`

运行：

```bash
HOST=0.0.0.0 PORT=9000 cargo run --release
```

Windows PowerShell：

```powershell
$env:HOST = "0.0.0.0"
$env:PORT = "9000"
cargo run --release
```

## 配置

程序会自动读取项目目录下的 `.env` 文件。

| 变量 | 用途 | 默认值 |
|---|---|---|
| `HOST` | 服务监听地址 | `127.0.0.1` |
| `PORT` | 服务监听端口 | `9000` |
| `PROXY_LIST` | 出站代理 URL | 空，直接连接 |
| `API_KEY` | 可选服务端鉴权密钥；只保护两个翻译接口 | 空，不启用鉴权 |

### API 鉴权

设置 `API_KEY` 后，`POST /translate` 和 `POST /v2/translate` 必须携带密钥。`GET /health` 与 `GET /v2/languages` 保持公开，方便探活和语言查询。

以下三种请求头均受支持：

```bash
curl http://127.0.0.1:9000/v2/translate \
  -H 'Authorization: DeepL-Auth-Key change-me' \
  -H 'Content-Type: application/json' \
  -d '{"text":"Hello","target_lang":"ZH"}'
```

也可以使用 `Authorization: Bearer change-me` 或 `X-API-Key: change-me`。

代理 URL 支持：

- `http://`
- `https://`
- `socks4://`：本地 DNS 解析
- `socks4a://`：代理端 DNS 解析
- `socks5://`：本地 DNS 解析
- `socks5h://`：代理端 DNS 解析

## API 示例

### 简单翻译接口

```bash
curl http://127.0.0.1:9000/translate \
  -H 'Content-Type: application/json' \
  -d '{"text":"Hello, world!","source_lang":"EN","target_lang":"ZH"}'
```

响应示例：

```json
{"code":200,"data":"你好，世界！","source_lang":"EN"}
```

### DeepL 官方兼容接口

```bash
curl http://127.0.0.1:9000/v2/translate \
  -H 'Content-Type: application/json' \
  -d '{"text":["Hello","World"],"source_lang":"EN","target_lang":"ZH"}'
```

响应格式：

```json
{
  "translations": [
    {"detected_source_language":"EN","text":"你好"},
    {"detected_source_language":"EN","text":"世界"}
  ]
}
```

`source_lang` 可设置为 `"auto"` 或空字符串进行自动检测。常用别名会自动转换：

- `EN` → `EN-US`
- `PT` → `PT-BR`
- `ZH` → `ZH-HANS`

### 查询语言

```bash
curl 'http://127.0.0.1:9000/v2/languages?type=source'
curl 'http://127.0.0.1:9000/v2/languages?type=target'
```

完整字段说明见 [`API.md`](API.md)。

## 限制

- 每个上游请求最多 1500 个 Unicode 字符；长文本会自动分片
- `/v2/translate` 每次最多接收 50 条文本
- 出站连接和请求总超时均为 20 秒
- 429 和网络错误只自动重试一次
- 服务端不提供认证、TLS 或自身速率限制

## 许可与使用说明

本项目仅供学习和研究。使用 DeepL 服务时请遵守 [DeepL 服务条款](https://www.deepl.com/pro-license)。
