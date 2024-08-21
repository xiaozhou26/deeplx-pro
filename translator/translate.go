package translator

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"log"
	"math/rand"
	"net/url"
	"time"

	http "github.com/bogdanfinn/fhttp"
	tls_client "github.com/bogdanfinn/tls-client"
	"github.com/xiaozhou26/tlsclient/tlsclient"
)

const deeplBaseURL = "https://api.deepl.com/jsonrpc"

var baseHeaders = map[string]string{
	"Accept":             "*/*",
	"Accept-Language":    "en-US,en;q=0.9",
	"Content-Type":       "application/json",
	"Origin":             "https://www.deepl.com",
	"Priority":           "u=1, i",
	"Referer":            "https://www.deepl.com/",
	"sec-ch-ua":          `"Not)A;Brand";v="99", "Google Chrome";v="127", "Chromium";v="127"`,
	"sec-ch-ua-mobile":   "?0",
	"sec-ch-ua-platform": `"Windows"`,
	"sec-fetch-dest":     "empty",
	"sec-fetch-mode":     "cors",
	"sec-fetch-site":     "same-site",
	"User-Agent":         "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36",
}

func getICount(translateText string) int {
	count := 0
	for _, char := range translateText {
		if char == 'i' {
			count++
		}
	}
	return count
}

func getRandomNumber() int64 {
	rand.Seed(time.Now().UnixNano())
	return rand.Int63n(100000) + 83000000000
}

func getTimestamp(iCount int) int64 {
	ts := time.Now().UnixNano() / int64(time.Millisecond)
	if iCount == 0 {
		return ts
	}
	iCount++
	return ts - (ts % int64(iCount)) + int64(iCount)
}

type translateParams struct {
	Jobs            []job      `json:"jobs"`
	Lang            langParams `json:"lang"`
	Priority        int        `json:"priority"`
	CommonJobParams jobParams  `json:"commonJobParams"`
	Timestamp       int64      `json:"timestamp"`
}

type job struct {
	Kind               string     `json:"kind"`
	Sentences          []sentence `json:"sentences"`
	RawEnContextBefore []string   `json:"raw_en_context_before"`
	RawEnContextAfter  []string   `json:"raw_en_context_after"`
	PreferredNumBeams  int        `json:"preferred_num_beams"`
}

type sentence struct {
	Text   string `json:"text"`
	ID     int    `json:"id"`
	Prefix string `json:"prefix"`
}

type jobParams struct {
	Quality         string `json:"quality"`
	RegionalVariant string `json:"regionalVariant"`
	Mode            string `json:"mode"`
	BrowserType     int    `json:"browserType"`
	TextType        string `json:"textType"`
	AdvancedMode    bool   `json:"advancedMode"`
}

type langParams struct {
	TargetLang         string     `json:"target_lang"`
	Preference         preference `json:"preference"`
	SourceLangComputed string     `json:"source_lang_computed"`
}

type preference struct {
	Weight  map[string]interface{} `json:"weight"`
	Default string                 `json:"default"`
}

type translateRequest struct {
	Jsonrpc string          `json:"jsonrpc"`
	Method  string          `json:"method"`
	ID      int64           `json:"id"`
	Params  translateParams `json:"params"`
}

type translateResponse struct {
	Result struct {
		Translations []struct {
			Beams []struct {
				Sentences []struct {
					Text string `json:"text"`
				} `json:"sentences"`
				NumSymbols int `json:"num_symbols"`
			} `json:"beams"`
			Quality string `json:"quality"`
		} `json:"translations"`
		TargetLang            string                 `json:"target_lang"`
		SourceLang            string                 `json:"source_lang"`
		SourceLangIsConfident bool                   `json:"source_lang_is_confident"`
		DetectedLanguages     map[string]interface{} `json:"detectedLanguages"`
	} `json:"result"`
}

func InitTranslator() {
	initCookies()
	initProxies()
}

func Translate(text, sourceLang, targetLang, quality string, tryCount int) (string, error) {
	if quality == "" {
		quality = "normal"
	}

	iCount := getICount(text)
	id := getRandomNumber()
	proxy, _ := GetNextProxy()
	cookie, err := getNextCookie()
	if err != nil {
		return "", err
	}

	maxRetries := 5
	if tryCount >= maxRetries {
		log.Println("Max retry limit reached.")
		return "", fmt.Errorf("max retry limit reached")
	}

	priority := 1
	advancedMode := false

	if sourceLang == "EN" || sourceLang == "ZH" || targetLang == "JA" || targetLang == "DE" {
		advancedMode = true
	}
	regionalVariant := "zh-Hans"
	if targetLang == "EN" {
		regionalVariant = "en-US"
	}
	if quality == "fast" {
		priority = -1
		advancedMode = false
	}

	headers := make(map[string]string)
	for k, v := range baseHeaders {
		headers[k] = v
	}

	postData := translateRequest{
		Jsonrpc: "2.0",
		Method:  "LMT_handle_jobs",
		ID:      id,
		Params: translateParams{
			Jobs: []job{
				{
					Kind: "default",
					Sentences: []sentence{
						{
							Text:   text,
							ID:     1,
							Prefix: "",
						},
					},
					RawEnContextBefore: []string{},
					RawEnContextAfter:  []string{},
					PreferredNumBeams:  4,
				},
			},
			Lang: langParams{
				TargetLang: targetLang,
				Preference: preference{
					Weight:  map[string]interface{}{},
					Default: "default",
				},
				SourceLangComputed: sourceLang,
			},
			Priority: priority,
			CommonJobParams: jobParams{
				Quality:         quality,
				RegionalVariant: regionalVariant,
				Mode:            "translate",
				BrowserType:     1,
				TextType:        "plaintext",
				AdvancedMode:    advancedMode,
			},
			Timestamp: getTimestamp(iCount),
		},
	}

	data, err := json.Marshal(postData)
	if err != nil {
		return "", err
	}

	jar := tls_client.NewCookieJar()
	options := []tls_client.HttpClientOption{
		tls_client.WithTimeoutSeconds(30),
		tls_client.WithClientProfile(tlsclient.Chrome127()),
		tls_client.WithNotFollowRedirects(),
		tls_client.WithCookieJar(jar),
	}

	if proxy != "" {
		options = append(options, tls_client.WithProxyUrl(proxy))
	}

	client, err := tls_client.NewHttpClient(tls_client.NewNoopLogger(), options...)
	if err != nil {
		log.Println(err)
		return "", err
	}

	// 设置cookie
	parsedURL, err := url.Parse(deeplBaseURL)
	if err != nil {
		return "", err
	}
	client.SetCookies(parsedURL, []*http.Cookie{
		{
			Name:  "dl_session",
			Value: cookie,
		},
	})

	req, err := http.NewRequest(http.MethodPost, deeplBaseURL, bytes.NewBuffer(data))
	if err != nil {
		return "", err
	}

	for key, value := range headers {
		req.Header.Set(key, value)
	}

	resp, err := client.Do(req)
	if err != nil {
		if proxy != "" {
			MarkProxyInvalid(proxy)
		}
		markCookieInvalid(cookie)
		log.Println("Retrying due to proxy or cookie error...")
		return Translate(text, sourceLang, targetLang, quality, tryCount+1)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		log.Printf("Error: %d\n", resp.StatusCode)
		return "", fmt.Errorf("error: %d", resp.StatusCode)
	}

	body, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		return "", err
	}

	var translateResp translateResponse
	if err := json.Unmarshal(body, &translateResp); err != nil {
		return "", err
	}

	if len(translateResp.Result.Translations) == 0 || len(translateResp.Result.Translations[0].Beams) == 0 {
		return "", fmt.Errorf("translation failed")
	}

	// 提取第一个翻译结果
	translatedText := translateResp.Result.Translations[0].Beams[0].Sentences[0].Text

	// 检查翻译结果是否为空
	if translatedText == "" {
		return "", fmt.Errorf("translation result is empty")
	}

	return translatedText, nil
}
