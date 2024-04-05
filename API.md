## 接口调用
### localhost为你的服务器IP
### 接口参数

- `text`: 需要翻译的文本
- `source_lang`: 提交的文本的源语言（可以设置为"auto"以自动识别）
- `target_lang`: 你希望翻译成的目标语言

### 调用示例：

<details>
<summary>Curl</summary>

```bash
curl --location 'https://localhost:8000/translate' \
--header 'Content-Type: application/json' \
--data '{
    "text": "Hello, world!",
    "source_lang": "EN",
    "target_lang": "ZH"
}'
```
</details>

<details>
<summary>JavaScript</summary>

```javascript
var myHeaders = new Headers();
myHeaders.append("Content-Type", "application/json");

var raw = JSON.stringify({
  "text": "Hello, world!",
  "source_lang": "auto",
  "target_lang": "ZH"
});

var requestOptions = {
  method: 'POST',
  headers: myHeaders,
  body: raw,
  redirect: 'follow'
};

fetch("https://localhost:8000/translate", requestOptions)
  .then(response => response.text())
  .then(result => console.log(result))
  .catch(error => console.log('error', error));
```
</details>

<details>
<summary>Node.js</summary>

```javascript
const axios = require('axios');
let data = JSON.stringify({
  "text": "Hello, world!",
  "source_lang": "auto",
  "target_lang": "ZH"
});

let config = {
  method: 'post',
  maxBodyLength: Infinity,
  url: 'https://localhost:8000/translate',
  headers: { 
    'Content-Type': 'application/json'
  },
  data : data
};

axios.request(config)
.then((response) => {
  console.log(JSON.stringify(response.data));
})
.catch((error) => {
  console.log(error);
});
```
</details>

<details>
<summary>Python</summary>

```python
import requests
import json

url = "https://localhost:8000/translate"

payload = json.dumps({
  "text": "Hello, world!",
  "source_lang": "auto",
  "target_lang": "ZH"
})
headers = {
  'Content-Type': 'application/json'
}

response = requests.request("POST", url, headers=headers, data=payload)

print(response.text)
```
</details>

## 如何在沉浸式翻译中使用

首先，访问[沉浸式翻译](https://immersivetranslate.com/)的官方网站，然后根据你的浏览器类型下载相应的插件。


安装插件后，点击浏览器右上角的插件按钮，选择"沉浸式翻译"，然后进入设置页面。
![p1](https://www.jsdelivr.ren/gh/xiaozhou26/tuph@main/images/20240314170457.png)

在左下角点击"开发者设置"，然后开启"Beta测试特性"。

点击"基本设置"，然后在翻译服务中选择"DeepLX"。在"API URL"字段中填入以下API地址：
![p2](https://www.jsdelivr.ren/gh/xiaozhou26/tuph@main/images/20240314170447.png)

```
https://localhost:8000/translate
```


最后，点击右上角的"测试服务"按钮。如果提示服务测试成功，那么你就可以开始使用了。
