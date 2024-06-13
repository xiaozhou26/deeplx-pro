// proxyPool.js

let proxies = process.env.PROXY_LIST ? process.env.PROXY_LIST.split(',') : [];
let invalidProxies = [];
let currentProxyIndex = 0;

function getNextProxy() {
  let attempts = 0;
  while (attempts < proxies.length) {
    const proxy = proxies[currentProxyIndex];
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
  }
}

module.exports = {
  getNextProxy,
  markProxyInvalid
};
