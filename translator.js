const axios = require('axios').default;
const { getNextProxy, markProxyInvalid } = require('./proxyPool');
const { getNextCookie, markCookieInvalid } = require('./cookieManager');

const DEEPL_BASE_URL = 'https://api.deepl.com/jsonrpc';

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

async function translate(text, sourceLang = 'AUTO', targetLang = 'ZH', quality = 'normal', tryCount = 0) {
  const iCount = getICount(text);
  const id = getRandomNumber();
  const proxy = getNextProxy();
  const cookie = getNextCookie();

  const maxRetries = 5;
  if (tryCount >= maxRetries) {
    console.error("Max retry limit reached.");
    return null;
  }

  if (!proxy && !cookie) {
    console.error("No valid proxies or cookies available.");
    return null;
  }

  const headers = { ...baseHeaders };
  if (cookie) headers['cookie'] = cookie;

  const priority = quality === 'fast' ? -1 : 1;
  const advancedMode = quality !== 'fast';

  const postData = {
    jsonrpc: '2.0',
    method: 'LMT_handle_jobs',
    id: id,
    params: {
      jobs: [
        {
          kind: 'default',
          sentences: [
            {
              text: text,
              id: 1,
              prefix: '',
            },
          ],
          raw_en_context_before: [],
          raw_en_context_after: [],
          preferred_num_beams: 4,
        },
      ],
      lang: {
        target_lang: targetLang.toUpperCase(),
        preference: {
          weight: {},
          default: 'default',
        },
        source_lang_computed: sourceLang.toUpperCase(),
      },
      priority: priority,
      commonJobParams: {
        quality: quality,
        regionalVariant: 'zh-Hans',
        mode: 'translate',
        browserType: 1,
        textType: 'plaintext',
        advancedMode: advancedMode,
      },
      timestamp: getTimestamp(iCount),
    },
  };

  const axiosConfig = {
    headers,
    proxy: proxy ? {
      host: proxy.hostname,
      port: parseInt(proxy.port, 10),
      auth: proxy.auth ? {
        username: proxy.auth.split(':')[0],
        password: proxy.auth.split(':')[1]
      } : undefined,
      protocol: 'http'  // 确保协议是正确的
    } : false,
  };

  try {
    const response = await axios.post(DEEPL_BASE_URL, postData, axiosConfig);
    if (response.status !== 200) {
      console.error('Error', response.status);
      return null;
    }

    const result = response.data.result;
    const translations = result && result.translations;
    if (translations && translations.length > 0 && translations[0].beams.length > 0) {
      const texts = translations[0].beams.flatMap(beam => beam.sentences.map(sentence => sentence.text));
      return {
        text: texts[0], // 返回第一个翻译结果
        alternatives: texts.slice(1) // 返回剩余的备用翻译
      };
    }
    return null;
  } catch (err) {
    console.error("response error:", err);

    if (proxy) markProxyInvalid(proxy);
    if (cookie) markCookieInvalid(cookie);

    console.log("Trying again due to assuming the current proxy or cookie is invalid...");
    return await translate(text, sourceLang, targetLang, quality, tryCount + 1);
  }
}

module.exports = { translate };
