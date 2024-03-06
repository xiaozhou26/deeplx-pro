# deeplx-pro

# 使用 DeepL Pro的翻译服务

DeepL 是一个优秀的翻译工具，提供了高质量的翻译结果。然而，它的免费版本有一定的限制。本文将介绍如何使用 DeepL Pro的 API 实现一个免费的翻译服务，并提供一个简单的 Express 服务器示例。

## 准备工作

在开始之前，你需要准备以下内容：

- Node.js 环境
- Express 框架
- Axios 库
- Lodash 库

你可以通过以下命令安装所需的依赖：

```bash
npm install express body-parser axios lodash
```

## 代码实现

下面是使用 DeepL pro 实现api翻译服务的代码：



## 使用方法

F12打开开发者控制台，搜索```https://api.deepl.com/jsonrpc?method=LMT_handle_jobs``` 然后按照图片复制所有的cookie。
![DeepL翻译API示例代码](https://jsd.cdn.zzko.cn/gh/xiaozhou26/tuph@main/images/2024-03-06%20221503.png)


1. 替换代码中的 `cookie` 变量为你自己的 DeepL Pro cookie。
2. 运行代码，启动 Express 服务器。
3. 使用 POST 请求向 `http://localhost:9000/translate` 发送翻译请求，请求体包含以下字段：
   - `text`：要翻译的文本
   - `source_lang`：源语言（可选，默认为 'AUTO'）
   - `target_lang`：目标语言（可选，默认为 'ZH'）
4. 服务器将返回翻译结果的 JSON 响应。

## 注意事项

- 请不要频繁发送大量请求，否则可能会被 DeepL 暂时阻止 IP。
- 本代码仅供学习和参考，请勿用于商业用途。
- 使用 DeepL 时，请遵守其使用条款和限制。
