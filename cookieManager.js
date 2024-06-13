// cookieManager.js

require('dotenv').config(); // Ensure .env is loaded

let cookies = process.env.DEEPL_COOKIES ? process.env.DEEPL_COOKIES.split(',') : [];
let invalidCookies = [];
let currentCookieIndex = 0;

// Validate cookies format
function validateCookies() {
  cookies = cookies.filter(cookie => /^[0-9a-fA-F-]{8}-[0-9a-fA-F-]{4}-[0-9a-fA-F-]{4}-[0-9a-fA-F-]{4}-[0-9a-fA-F-]{12}$/.test(cookie));
  if (cookies.length === 0) {
    console.error("No valid cookies provided. Please check your DEEPL_COOKIES environment variable.");
    console.error("Cookies found:", process.env.DEEPL_COOKIES);
    process.exit(1);
  }
}

function getNextCookie() {
  let attempts = 0;
  while (attempts < cookies.length) {
    const cookie = cookies[currentCookieIndex];
    if (!invalidCookies.includes(cookie)) {
      currentCookieIndex = (currentCookieIndex + 1) % cookies.length;
      return `dl_session=${cookie}`;
    }
    currentCookieIndex = (currentCookieIndex + 1) % cookies.length;
    attempts++;
  }
  return null;
}

function markCookieInvalid(cookie) {
  const cookieValue = cookie.replace('dl_session=', '');
  if (!invalidCookies.includes(cookieValue)) {
    invalidCookies.push(cookieValue);
  }
}

// Initialize and validate cookies on startup
validateCookies();

module.exports = {
  getNextCookie,
  markCookieInvalid
};
