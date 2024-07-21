require('dotenv').config(); // Load environment variables from .env file
const express = require('express');
const bodyParser = require('body-parser');
const { translate } = require('./translator');

const app = express();
const PORT = process.env.PORT || 9000;

app.use(bodyParser.json());

app.get('/', (req, res) => {
  res.send('Welcome to deeplx-pro');
});

app.get('/translate', (req, res) => {
  res.status(405).send('GET method not supported for this endpoint. Please use POST.');
});

app.post('/translate', async (req, res) => {
  const { text, source_lang, target_lang, quality } = req.body;
  try {
    const result = await translate(text, source_lang, target_lang, quality || 'normal');
    if (!result) {
      res.status(500).json({ error: 'Translation failed or too many requests' });
      return;
    }
    res.json({
      alternatives: result.alternatives,
      code: 200,
      data: result.text,
      id: Math.floor(Math.random() * 10000000000),
      source_lang: source_lang.toUpperCase(),
      target_lang: target_lang.toUpperCase(),
    });
  } catch (error) {
    res.status(500).json({ error: 'Translation failed', details: error.message });
  }
});

app.listen(PORT, () => {
  console.log(`Server is running on http://localhost:${PORT}`);
  console.log(`DEEPL_COOKIES: ${process.env.DEEPL_COOKIES}`);
  console.log(`PROXY_LIST: ${process.env.PROXY_LIST}`);
});
