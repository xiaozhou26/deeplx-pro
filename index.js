const axios = require('axios').default;
const express = require('express');
const bodyParser = require('body-parser');
const DEEPL_BASE_URL = 'https://api.deepl.com/jsonrpc';
const app = express();
const PORT = 9000;

let currentCookieIndex = 0;
// Select the next cookie in a round-robin fashion
function getNextCookie() {
  const cookieValue = cookies[currentCookieIndex];
  const cookie = `dl_session=${cookieValue}`; // 重新添加 "dl_session=" 前缀
  currentCookieIndex = (currentCookieIndex + 1) % cookies.length;
  return cookie;
}

// Basic headers template, excluding the cookie which will be dynamically inserted
const baseHeaders = {
  'Content-Type': 'application/json',
  Accept: '*/*',
  'Accept-Language': 'en-US,en;q=0.9',
  'sec-ch-ua': '"Chromium";v="122", "Not(A:Brand";v="24", "Google Chrome";v="122"',
  'sec-ch-ua-mobile': '?0',
  'sec-ch-ua-platform': '"Windows"',
  'sec-fetch-dest': 'empty',
  'sec-fetch-mode': 'cors',
  'sec-fetch-site': 'same-site',
  'Referer': 'https://www.deepl.com/',
  'Referrer-Policy': 'strict-origin-when-cross-origin'
};

// Utility functions for the translation logic
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

// The translation function
async function translate(text, sourceLang = 'AUTO', targetLang = 'ZH', numberAlternative = 0, printResult = false) {
  const iCount = getICount(text);
  const id = getRandomNumber();
  const headers = { ...baseHeaders, 'cookie': getNextCookie() }; // Include the selected cookie

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

  try {
    const response = await axios.post(DEEPL_BASE_URL, JSON.stringify(postData), { headers });
    if (response.status !== 200) {
      console.error('Error', response.status);
      return null;
    }
    return response.data.result.texts[0];
  } catch (err) {
    console.error(err);
    return null;
  }
}

// Express server setup
app.use(bodyParser.json());

app.get('/', (req, res) => {
  res.send('Welcome to deeplx-pro');
});

app.post('/translate', async (req, res) => {
  const { text, source_lang, target_lang } = req.body;
  try {
    const result = await translate(text, source_lang, target_lang);
    if (!result) {
      res.status(500).json({ error: 'Translation failed or too many requests' });
      return;
    }
    res.json({
      alternatives: result.alternatives,
      code: 200,
      data: result.text,
      id: Math.floor(Math.random() * 10000000000),
      method: 'Free',
      source_lang: source_lang.toUpperCase(),
      target_lang: target_lang.toUpperCase(),
    });
  } catch (error) {
    res.status(500).json({ error: 'Translation failed' });
  }
});

app.listen(PORT, () => {
  console.log(`Server is running on http://localhost:${PORT}`);
});
