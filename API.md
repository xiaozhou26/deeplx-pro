# API 文档

## 基础信息

| 项目 | 值 |
|---|---|
| 接口地址 | `http://127.0.0.1:9000` |
| 请求方式 | `POST` |
| 接口路径 | `/translate` |
| 请求格式 | `application/json` |
| 响应格式 | `application/json` |
| 编码 | UTF-8 |

---

## 翻译接口

`POST /translate`

### 请求参数

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `text` | string | 是 | 待翻译文本，超过 1500 字符会自动分片翻译 |
| `target_lang` | string | 是 | 目标语言代码 |
| `source_lang` | string | 否 | 源语言代码，留空或 `"auto"` 表示自动检测 |
| `quality` | string | 否 | 保留字段，当前未使用 |

### 请求示例

```bash
curl 'http://127.0.0.1:9000/translate' \
  -H 'Content-Type: application/json' \
  -d '{
    "text":"Good morning!",
    "source_lang":"EN",
    "target_lang":"ZH"
  }'
```

```bash
# 自动检测源语言
curl 'http://127.0.0.1:9000/translate' \
  -H 'Content-Type: application/json' \
  -d '{
    "text":"Bonjour le monde",
    "target_lang":"EN"
  }'
```

```bash
# 翻译为日语
curl 'http://127.0.0.1:9000/translate' \
  -H 'Content-Type: application/json' \
  -d '{"text":"Thank you","target_lang":"JA"}'
```

### 成功响应

```json
{
  "code": 200,
  "data": "早上好！",
  "source_lang": "EN"
}
```

| 字段 | 类型 | 说明 |
|---|---|---|
| `code` | number | 状态码，固定为 200 |
| `data` | string | 翻译后的文本 |
| `source_lang` | string/null | 检测到的源语言代码，自动检测时返回 |

### 错误响应

```json
{
  "code": 400,
  "message": "unsupported target language: XX"
}
```

| 字段 | 类型 | 说明 |
|---|---|---|
| `code` | number | HTTP 状态码 |
| `message` | string | 错误描述 |

### HTTP 状态码说明

| 状态码 | 含义 | 触发条件 |
|---|---|---|
| 200 | 成功 | 翻译完成 |
| 400 | 参数错误 | 不支持的语种代码 |
| 413 | 请求体过大 | 文本超过 1500 字符 |
| 429 | 请求过多 | DeepL 端限流 |
| 502 | 上游错误 | DeepL 接口异常、网络错误、JSON 解析失败等 |

---

## 支持的语言

### 目标语言（target_lang）

| 代码 | 语言 |
|---|---|
| `AR` | 阿拉伯语 |
| `BG` | 保加利亚语 |
| `CS` | 捷克语 |
| `DA` | 丹麦语 |
| `DE` | 德语 |
| `EL` | 希腊语 |
| `EN-GB` | 英语（英式） |
| `EN-US` | 英语（美式） |
| `ES` | 西班牙语 |
| `ES-419` | 西班牙语（拉丁美洲） |
| `ET` | 爱沙尼亚语 |
| `FI` | 芬兰语 |
| `FR` | 法语 |
| `HE` | 希伯来语 |
| `HU` | 匈牙利语 |
| `ID` | 印尼语 |
| `IT` | 意大利语 |
| `JA` | 日语 |
| `KO` | 韩语 |
| `LT` | 立陶宛语 |
| `LV` | 拉脱维亚语 |
| `NB` | 挪威语（博克马尔） |
| `NL` | 荷兰语 |
| `PL` | 波兰语 |
| `PT-BR` | 葡萄牙语（巴西） |
| `PT-PT` | 葡萄牙语（欧洲） |
| `RO` | 罗马尼亚语 |
| `RU` | 俄语 |
| `SK` | 斯洛伐克语 |
| `SL` | 斯洛文尼亚语 |
| `SV` | 瑞典语 |
| `TR` | 土耳其语 |
| `UK` | 乌克兰语 |
| `VI` | 越南语 |
| `ZH` / `ZH-HANS` | 简体中文 |
| `ZH-HANT` | 繁体中文 |

### 源语言（source_lang）

与目标语言代码相同，额外支持 `EN`、`PT`、`ZH` 三个简写：

| 简写 | 展开 |
|---|---|
| `EN` | `EN-US` |
| `PT` | `PT-BR` |
| `ZH` | `ZH-HANS` |

设置为 `"auto"` 或空字符串时自动检测源语言。

### 语言代码别名

| 输入 | 实际使用 |
|---|---|
| `EN` | `EN-US` |
| `PT` | `PT-BR` |
| `ZH` | `ZH-HANS` |
| `en-gb`（不区分大小写） | `en-GB` |
| `zh_hans`（下划线转连字符） | `zh-Hans` |

---

## 浏览器调用（JavaScript）

```javascript
fetch('http://127.0.0.1:9000/translate', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    text: 'Hello',
    source_lang: 'auto',
    target_lang: 'ZH'
  })
})
  .then(res => res.json())
  .then(data => console.log(data.data));
```

> 由于服务启用了全开 CORS，浏览器端可直接调用。

---

## 官方 API 兼容端点

以下三个端点尽量与 [DeepL 官方 API](https://developers.deepl.com/api-reference) 对齐，方便已有官方 SDK 的项目改 `base_url` 直接接入。

---

## 翻译接口（官方兼容）

`POST /v2/translate`

### 请求参数

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `text` | string \| string[] | 是 | 待翻译文本，可以是单条字符串或数组（最多 50 条）。单条超出 1500 字符会自动分片 |
| `target_lang` | string | 是 | 目标语言代码 |
| `source_lang` | string | 否 | 源语言代码，留空或 `"auto"` 表示自动检测 |
| `formality` | string | 否 | 保留字段，当前未使用 |

### 请求示例

```bash
# 单条文本（字符串形式）
curl 'http://127.0.0.1:9000/v2/translate' \
  -H 'Content-Type: application/json' \
  -d '{"text":"Hello, world!","target_lang":"DE"}'

# 批量翻译（数组形式，最多 50 条）
curl 'http://127.0.0.1:9000/v2/translate' \
  -H 'Content-Type: application/json' \
  -d '{"text":["Hello","Good morning"],"target_lang":"ZH"}'
```

### 成功响应

```json
{
  "translations": [
    {
      "detected_source_language": "EN",
      "text": "你好，世界！"
    }
  ]
}
```

| 字段 | 类型 | 说明 |
|---|---|---|
| `translations` | array | 与输入文本一一对应的翻译结果 |
| `translations[].detected_source_language` | string | 检测到的源语言（源语言指定时通常为空字符串） |
| `translations[].text` | string | 翻译后的文本 |

> ⚠️ 与旧 `/translate` 不同，官方兼容端点不返回外层 `code` 字段，错误时直接返回对应 HTTP 状态码 + `{code, message}`。

---

## 语言列表接口（官方兼容）

`GET /v2/languages`

返回支持的语言列表，响应格式与 DeepL 官方 `/v2/languages` 一致。

### 查询参数

| 字段 | 说明 |
|---|---|
| `type` | 可选，`source` 或 `target`（默认 `target`）。`target` 返回 `supports_formality` 字段，`source` 不返回 |

### 请求示例

```bash
curl 'http://127.0.0.1:9000/v2/languages?type=target'
```

### 成功响应

```json
[
  { "language": "DE", "name": "German", "supports_formality": true },
  { "language": "EN-US", "name": "English (American)", "supports_formality": false },
  { "language": "ZH-HANS", "name": "Chinese (Simplified)", "supports_formality": false }
]
```

---

## 健康检查

`GET /health`

供 Docker / Kubernetes / 监控系统做存活探针，无需鉴权，不访问上游。

```bash
curl 'http://127.0.0.1:9000/health'
# {"status":"ok"}
```

---

## 注意事项

1. **免费端点限制**：所有翻译请求固定发送到 `https://oneshot-free.www.deepl.com/v1/translate`，高频调用可能触发限流（429）。
2. **自动重试**：遇到 429 或网络错误时会自动重试 1 次（重试前重新预热 Cookie 并延迟 1 秒）。
3. **长文本分片**：单条文本超过 1500 字符时自动切分并顺序翻译，对调用方透明；`/v2/translate` 单次最多 50 条文本。
4. **Cookie 依赖**：服务启动时会访问 DeepL 网站获取 Cookie，确保网络连通性。
5. **不可用于生产**：本接口模拟浏览器扩展行为，DeepL 官方可能随时调整接口，不建议用于商业/生产环境。
