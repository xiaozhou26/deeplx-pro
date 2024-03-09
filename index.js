const axios = require('axios').default;
const { random } = require('lodash');
const express = require('express');
const bodyParser = require('body-parser');

const DEEPL_BASE_URL = 'https://api.deepl.com/jsonrpc';
const headers = {
  'Content-Type': 'application/json',
  Accept: '*/*',
  'Accept-Language': 'en-US,en;q=0.9',
  'sec-ch-ua': '"Chromium";v="122", "Not(A:Brand";v="24", "Google Chrome";v="122"',
  'sec-ch-ua-mobile': '?0',
  'sec-ch-ua-platform': '"Windows"',
  'sec-fetch-dest': 'empty',
  'sec-fetch-mode': 'cors',
  'sec-fetch-site': 'same-site',
  'cookie': `dl_session=${process.env.DL_SESSION}`, // 从环境变量中读取dl_session
  'Referer': 'https://www.deepl.com/',
  'Referrer-Policy': 'strict-origin-when-cross-origin'
};
function getICount(translateText) {
  return (translateText || '').split('i').length - 1;
}

function getRandomNumber() {
  return Math.floor(Math.random() * 100000) + 83000000000;
}

function getTimestamp(iCount) {
  const ts = Date.now();
  if (iCount === 0) {
    return ts;
  }
  iCount++;
  return ts - (ts % iCount) + iCount;
}

async function translate(
  text,
  sourceLang = 'AUTO',
  targetLang = 'ZH',
  numberAlternative = 0,
  printResult = false,
) {
  const iCount = getICount(text);
  const id = getRandomNumber();

  numberAlternative = Math.max(Math.min(3, numberAlternative), 0);

  const postData = {
    jsonrpc: '2.0',
    method: 'LMT_handle_texts',
    id: id,
    params: {
      texts: [{ text: text, requestAlternatives: numberAlternative }],
      splitting: 'newlines',
      lang: {
        source_lang_user_selected: sourceLang.toUpperCase(),
        target_lang: targetLang.toUpperCase(),
      },
      timestamp: getTimestamp(iCount),
    },
  };

  let postDataStr = JSON.stringify(postData);

  if ((id + 5) % 29 === 0 || (id + 3) % 13 === 0) {
    postDataStr = postDataStr.replace('"method":"', '"method" : "');
  } else {
    postDataStr = postDataStr.replace('"method":"', '"method": "');
  }

  try {
    const response = await axios.post(DEEPL_BASE_URL, postDataStr, {
      headers: headers,
    });

    if (response.status === 429) {
      throw new Error(
        `Too many requests, your IP has been blocked by DeepL temporarily, please don't request it frequently in a short time.`
      );
    }

    if (response.status !== 200) {
      console.error('Error', response.status);
      return;
    }

    const result = response.data.result.texts[0]
    if (printResult) {
      console.log(result);
    }
    return result;
  } catch (err) {
    console.error(err);
  }
}

const app = express();
const PORT = 9000;

app.use(bodyParser.json());
app.get('/', (req, res) => {
  res.send('Welcome to deeplx-pro');
});
app.get('/translate', (req, res) => {
  res.send('This is a translate api');
});

app.post('/translate', async (req, res) => {
  const { text, source_lang, target_lang } = req.body;

  try {
    const result = await translate(text, source_lang, target_lang);
    const responseData = {
      alternatives: result.alternatives,
      code: 200,
      data: result.text, // 取第一个翻译结果
      id: Math.floor(Math.random() * 10000000000), // 生成一个随机 ID
      method: 'Free',
      source_lang,
      target_lang,
    };

    res.json(responseData);
  } catch (error) {
    res.status(500).json({ error: 'Translation failed' });
  }
});

app.listen(PORT, () => {
  console.log(`Server is running on http://localhost:${PORT}`);
});
