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

async function translate(text, sourceLang = 'AUTO', targetLang = 'ZH', numberAlternative = 0, tryCount = 0) {
  const iCount = getICount(text);
  const id = getRandomNumber();
  const proxy = getNextProxy();
  const cookie = getNextCookie();

  if (!proxy || !cookie) {
    console.error("No valid proxies or cookies available.");
    return null;
  }

  const headers = { ...baseHeaders, 'proxy': proxy, 'cookie': cookie };

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
    console.error("response error:" + err);
    markProxyInvalid(proxy);
    markCookieInvalid(cookie);
    console.log("Trying again due to assuming the current proxy or cookie is invalid...");
    return await translate(text, sourceLang, targetLang, numberAlternative, tryCount + 1);
  }
}

module.exports = { translate };
