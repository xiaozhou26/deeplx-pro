# deeplx-pro

[![Deploy with Vercel](https://vercel.com/button)](https://vercel.com/new/clone?repository-url=https%3A%2F%2Fgithub.com%2Fxiaozhou26%2Fdeeplx-pro&env=DEEPL_COOKIES&project-name=deeplx-pro&repository-name=deeplx-pro)

## 简介

`deeplx-pro` 是一个非官方的 DeepL web 翻译接口封装，允许通过 HTTP 请求访问 DeepL 翻译服务。

**该项目仅供学习和参考，请勿用于商业用途。**

## 获得 COOKIES

1. 进入 [DeepL](https://www.deepl.com) 网站，按下 `F12` 打开开发者工具，选择 "Application" 标签页。
2. 复制 `Cookies` 下的 `dl_session` 。

   ![获取 DeepL dl_session](https://cdn.jsdmirror.com/gh/xiaozhou26/tuph@main/images/2024-03-07%20120245.png)

## 环境变量

在使用或部署 `deeplx-pro` 时，需要设置以下环境变量：

```plaintext
DEEPL_COOKIES=
PORT=9000
PROXY_LIST=
```

- **DEEPL_COOKIES**: （必需）通过浏览器获取的 `dl_session` Cookie 值（多个值用逗号分隔）。
- **PORT**: （可选）服务器运行端口，默认为 `9000`。
- **PROXY_LIST**: （可选）代理列表。

### 示例

```
DEEPL_COOKIES="1560565165-1811-481,515156-561561-11651"
```

> **注意**: 如果有任何一个 `dl_session` 失效，将会影响到翻译服务的正常运行。

## 部署使用

### Docker 部署

你可以通过 Docker 轻松部署 `deeplx-pro`：

```bash
docker run -d --name deeplx-pro -p 9000:9000 -e DEEPL_COOKIES="<your_dl_session_values>" xiaoxiaofeihh/deeplx-pro:latest
```

### Windows 平台运行

在 Windows 系统上运行 `deeplx-pro`，请在包含 deeplx-pro 的目录下打开`cmd` ：

1. 设置 `DEEPL_COOKIES` 环境变量（注意不用引号）：
   ```bat
   set DEEPL_COOKIES=<dl_session_values>
   ```
2. 运行可执行文件：
   ```bat
   deeplx-pro-windows-amd64.exe
   ```

### API 使用

你可以通过以下步骤来调用 `deeplx-pro` 提供的 API：

1. 确保已正确设置 `DEEPL_COOKIES` 环境变量。
2. 启动服务器并发送 POST 请求到 `http://localhost:9000/translate`。
3. 请求体应包含以下字段：
   - `text`：要翻译的文本内容。
   - `source_lang`：源语言（可选，默认为 'AUTO'）。
   - `target_lang`：目标语言（可选，默认为 'ZH'）。
   - `quality`：翻译质量（可选），可选值为 'normal' 或 'fast'，默认为 'normal'。

#### 示例请求

##### Linux

```sh
curl 'http://127.0.0.1:9000/translate' \
--header 'Content-Type: application/json' \
--data '{
  "text": "Hello, world!",
  "source_lang": "EN",
  "target_lang": "ZH",
  "quality": "normal"
}'
```

##### Windows

需要将单引号替换为双引号，并转义 JSON 数据中的双引号

```bat
curl "http://127.0.0.1:9000/translate" ^
--header "Content-Type: application/json" ^
--data "{ \"text\": \"Hello, world!\", \"source_lang\": \"EN\", \"target_lang\": \"ZH\", \"quality\": \"normal\" }"
```

服务器将返回翻译结果的 JSON 响应。

有关 API 的更多使用细节，请参考 [API 使用教程](https://github.com/xiaozhou26/deeplx/blob/main/API.md)。

## 注意事项

- **请求频率**: 请不要频繁发送大量请求，否则可能会导致 IP 被 DeepL 暂时封锁。
- **使用条款**: 使用本项目时，请遵守 DeepL 的使用条款和限制。
- **语言支持**: 不同语言的区域变体(RegionalVariant)可能不同，某些语言支持正式和非正式翻译。目前并未支持所有语言，欢迎通过 PR 来贡献更多语言支持。