const url = require('url');

let proxies = process.env.PROXY_LIST ? process.env.PROXY_LIST.split(',') : [];
let invalidProxies = [];
let currentProxyIndex = 0;

function parseProxy(proxy) {
  // 如果包含协议前缀，删除协议前缀
  if (proxy.startsWith('http://')) {
    proxy = proxy.slice(7);
  } else if (proxy.startsWith('socks5://')) {
    proxy = proxy.slice(8);
  }

  const parsedUrl = url.parse(`http://${proxy}`); // 添加临时的协议，以便于使用 url.parse 解析
  const { hostname, port, auth } = parsedUrl;

  return { hostname, port, auth };
}

function getNextProxy() {
  let attempts = 0;

  // 如果所有代理都被标记为无效，重置无效代理列表
  if (invalidProxies.length >= proxies.length) {
    invalidProxies = [];
  }

  while (attempts < proxies.length) {
    const proxy = proxies[currentProxyIndex];
    const { hostname, port, auth } = parseProxy(proxy);

    if (!invalidProxies.includes(proxy)) {
      currentProxyIndex = (currentProxyIndex + 1) % proxies.length;
      return { hostname, port, auth };
    }
    currentProxyIndex = (currentProxyIndex + 1) % proxies.length;
    attempts++;
  }
  return null;
}

function markProxyInvalid(proxy) {
  if (!invalidProxies.includes(proxy)) {
    invalidProxies.push(proxy);
    console.log(`Proxy marked as invalid: ${proxy}`);
  }
}

module.exports = {
  getNextProxy,
  markProxyInvalid
};
