let proxies = process.env.PROXY_LIST ? process.env.PROXY_LIST.split(',') : [];
let invalidProxies = [];
let currentProxyIndex = 0;

function parseProxy(proxy) {
  if (proxy.startsWith('http://')) {
    proxy = proxy.slice(7);  // 删除 'http://'
  } else if (proxy.startsWith('socks://')) {
    proxy = proxy.slice(8);  // 删除 'socks://'
  }
  return proxy;
}

function getNextProxy() {
  let attempts = 0;

  // 如果所有代理都被标记为无效，重置无效代理列表
  if (invalidProxies.length >= proxies.length) {
    invalidProxies = [];
  }

  while (attempts < proxies.length) {
    let proxy = proxies[currentProxyIndex];
    proxy = parseProxy(proxy);  // 解析代理，删除前缀

    if (!invalidProxies.includes(proxy)) {
      currentProxyIndex = (currentProxyIndex + 1) % proxies.length;
      return proxy;
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
