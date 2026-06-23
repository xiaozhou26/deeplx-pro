use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use axum::http::HeaderName;
use tower_http::cors::{Any, CorsLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use wreq::Client;
use wreq_util::Emulation;

// ═══════════════════════════════════════════════════════════════
//  Constants
// ═══════════════════════════════════════════════════════════════

const ONESHOT_FREE: &str = "https://oneshot-free.www.deepl.com/v1/translate";
const ONESHOT_PRO: &str = "https://oneshot-pro.www.deepl.com/v1/translate";
const MAX_TEXT_LENGTH: usize = 1500;

// ═══════════════════════════════════════════════════════════════
//  Language table
// ═══════════════════════════════════════════════════════════════

struct LangInfo {
    code: &'static str,      // canonical ISO-ish code (uppercase)
    name: &'static str,      // English display name (matches DeepL's official naming)
    internal: &'static str,  // DeepL internal lowercase code
    formality: bool,         // whether DeepL supports formality for this target
}

/// Languages supported by DeepL text translation.
/// `formality` is true for the languages DeepL documents as supporting
/// the formality parameter: DE, FR, IT, ES, ES-419, NL, PL, PT-BR, PT-PT, RU, JA.
static LANGUAGES: &[LangInfo] = &[
    LangInfo { code: "AR",      name: "Arabic",                   internal: "ar",      formality: false },
    LangInfo { code: "BG",      name: "Bulgarian",                internal: "bg",      formality: false },
    LangInfo { code: "CS",      name: "Czech",                    internal: "cs",      formality: false },
    LangInfo { code: "DA",      name: "Danish",                   internal: "da",      formality: false },
    LangInfo { code: "DE",      name: "German",                   internal: "de",      formality: true  },
    LangInfo { code: "EL",      name: "Greek",                    internal: "el",      formality: false },
    LangInfo { code: "EN-GB",   name: "English (British)",        internal: "en-GB",   formality: false },
    LangInfo { code: "EN-US",   name: "English (American)",       internal: "en-US",   formality: false },
    LangInfo { code: "ES",      name: "Spanish",                  internal: "es",      formality: true  },
    LangInfo { code: "ES-419",  name: "Spanish (Latin American)", internal: "es-419",  formality: true  },
    LangInfo { code: "ET",      name: "Estonian",                 internal: "et",      formality: false },
    LangInfo { code: "FI",      name: "Finnish",                  internal: "fi",      formality: false },
    LangInfo { code: "FR",      name: "French",                   internal: "fr",      formality: true  },
    LangInfo { code: "HE",      name: "Hebrew",                   internal: "he",      formality: false },
    LangInfo { code: "HU",      name: "Hungarian",                internal: "hu",      formality: false },
    LangInfo { code: "ID",      name: "Indonesian",               internal: "id",      formality: false },
    LangInfo { code: "IT",      name: "Italian",                  internal: "it",      formality: true  },
    LangInfo { code: "JA",      name: "Japanese",                 internal: "ja",      formality: true  },
    LangInfo { code: "KO",      name: "Korean",                   internal: "ko",      formality: false },
    LangInfo { code: "LT",      name: "Lithuanian",               internal: "lt",      formality: false },
    LangInfo { code: "LV",      name: "Latvian",                  internal: "lv",      formality: false },
    LangInfo { code: "NB",      name: "Norwegian (Bokmål)",       internal: "nb",      formality: false },
    LangInfo { code: "NL",      name: "Dutch",                    internal: "nl",      formality: true  },
    LangInfo { code: "PL",      name: "Polish",                   internal: "pl",      formality: true  },
    LangInfo { code: "PT-BR",   name: "Portuguese (Brazilian)",   internal: "pt-BR",   formality: true  },
    LangInfo { code: "PT-PT",   name: "Portuguese (European)",    internal: "pt-PT",   formality: true  },
    LangInfo { code: "RO",      name: "Romanian",                 internal: "ro",      formality: false },
    LangInfo { code: "RU",      name: "Russian",                  internal: "ru",      formality: true  },
    LangInfo { code: "SK",      name: "Slovak",                   internal: "sk",      formality: false },
    LangInfo { code: "SL",      name: "Slovenian",                internal: "sl",      formality: false },
    LangInfo { code: "SV",      name: "Swedish",                  internal: "sv",      formality: false },
    LangInfo { code: "TR",      name: "Turkish",                  internal: "tr",      formality: false },
    LangInfo { code: "UK",      name: "Ukrainian",                internal: "uk",      formality: false },
    LangInfo { code: "VI",      name: "Vietnamese",               internal: "vi",      formality: false },
    LangInfo { code: "ZH-HANS", name: "Chinese (Simplified)",     internal: "zh-Hans", formality: false },
    LangInfo { code: "ZH-HANT", name: "Chinese (Traditional)",    internal: "zh-Hant", formality: false },
];

/// Find a language entry by (normalized) input code. Handles common aliases:
/// `EN` → `EN-US`, `PT` → `PT-BR`, `ZH` → `ZH-HANS`, case-insensitive,
/// `_` ↔ `-`.
fn find_lang(code: &str) -> Option<&'static LangInfo> {
    let normalized = code.to_uppercase().replace('_', "-");
    let key = match normalized.as_str() {
        "EN" => "EN-US",
        "PT" => "PT-BR",
        "ZH" => "ZH-HANS",
        c => c,
    };
    LANGUAGES.iter().find(|l| l.code == key)
}

fn resolve_target_lang(code: &str) -> Result<&'static str, String> {
    find_lang(code)
        .map(|l| l.internal)
        .ok_or_else(|| format!("unsupported target language: {}", code))
}

fn resolve_source_lang(code: &str) -> Result<Option<&'static str>, String> {
    let c = code.to_uppercase().replace('_', "-");
    if c.is_empty() || c == "AUTO" {
        return Ok(None);
    }
    // Source language also accepts the bare EN/PT/ZH forms; find_lang already
    // aliases those to the region-specific entries.
    find_lang(&c)
        .map(|l| Some(l.internal))
        .ok_or_else(|| format!("unsupported source language: {}", c))
}

// ═══════════════════════════════════════════════════════════════
//  Instance ID
// ═══════════════════════════════════════════════════════════════

fn new_instance_id() -> String {
    let mut b = [0u8; 16];
    rand::thread_rng().fill(&mut b);
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15],
    )
}

static INSTANCE_ID: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(new_instance_id);

// ═══════════════════════════════════════════════════════════════
//  Text chunking
// ═══════════════════════════════════════════════════════════════
//
// Splits text into chunks of at most `max_chars` Unicode scalar values,
// preferring natural break points so translations stitch back together
// coherently. Priority of split points:
//   1. blank line (\n\n)
//   2. single newline (\n)
//   3. sentence-ending punctuation (. ! ? ．。！？)
//   4. hard cut (no nice boundary available)
//
// Counting uses `chars().count()` to match MAX_TEXT_LENGTH's semantics.

fn count_chars(s: &str) -> usize {
    s.chars().count()
}

/// Index in `chars` *after* which we may split (left = chars[..end]).
struct Boundary {
    end: usize,
}

/// Scan for the best boundary within the window [start, limit) and return the
/// last (greediest) occurrence of the highest-priority class found; fall back to
/// a hard cut at `limit` if none exists.
fn find_boundary(chars: &[char], start: usize, limit: usize) -> Boundary {
    let end_exclusive = limit.min(chars.len());

    let mut best_newline_newline: Option<usize> = None; // after the 2nd \n of \n\n
    let mut best_newline: Option<usize> = None;          // after a single \n
    let mut best_sentence: Option<usize> = None;         // after a sentence punct

    let mut i = start;
    while i < end_exclusive {
        let c = chars[i];
        if c == '\n' {
            if i + 1 < end_exclusive && chars[i + 1] == '\n' {
                best_newline_newline = Some(i + 2);
            } else {
                best_newline = Some(i + 1);
            }
        } else if matches!(c, '.' | '!' | '?' | '．' | '。' | '！' | '？') {
            best_sentence = Some(i + 1);
        }
        i += 1;
    }

    if let Some(end) = best_newline_newline {
        Boundary { end }
    } else if let Some(end) = best_newline {
        Boundary { end }
    } else if let Some(end) = best_sentence {
        Boundary { end }
    } else {
        Boundary { end: end_exclusive }
    }
}

/// Split `text` into chunks each ≤ `max_chars` Unicode scalars.
pub fn chunk_text(text: &str, max_chars: usize) -> Vec<String> {
    if max_chars == 0 {
        return Vec::new();
    }
    if text.is_empty() {
        return vec![String::new()];
    }

    let chars: Vec<char> = text.chars().collect();
    let total = chars.len();
    let mut chunks: Vec<String> = Vec::new();
    let mut start = 0usize;

    while start < total {
        if total - start <= max_chars {
            let tail: String = chars[start..].iter().collect();
            chunks.push(tail);
            break;
        }

        let limit = start + max_chars;
        let boundary = find_boundary(&chars, start, limit);

        // Guarantee forward progress: never emit a zero-length chunk.
        let end = if boundary.end <= start {
            limit.min(total)
        } else {
            boundary.end
        };

        let chunk: String = chars[start..end].iter().collect();
        chunks.push(chunk);
        start = end;
    }

    chunks
}

// ═══════════════════════════════════════════════════════════════
//  Error type
// ═══════════════════════════════════════════════════════════════

/// Translation errors classified by retryability.
enum TranslateError {
    /// DeepL returned 429. Worth retrying after a short delay + fresh cookies.
    RateLimit,
    /// Network-layer failure (connect/timeout/reset). Worth retrying.
    Network,
    /// Other upstream HTTP error (4xx/5xx). Not retried.
    Upstream(String),
    /// Client-side problem (empty text, unsupported language). Not retried.
    Client(String),
}

impl std::fmt::Display for TranslateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TranslateError::RateLimit => write!(f, "429: too many requests"),
            TranslateError::Network => write!(f, "network error"),
            TranslateError::Upstream(s) => write!(f, "{}", s),
            TranslateError::Client(s) => write!(f, "{}", s),
        }
    }
}

impl TranslateError {
    fn is_retryable(&self) -> bool {
        matches!(self, TranslateError::RateLimit | TranslateError::Network)
    }
}

// ═══════════════════════════════════════════════════════════════
//  Request / Response types
// ═══════════════════════════════════════════════════════════════

#[derive(Serialize)]
struct AppInformation {
    os: String,
    os_version: String,
    app_version: String,
    app_build: String,
    instance_id: String,
}

#[derive(Serialize)]
struct OneshotRequest<'a> {
    text: Vec<String>,
    target_lang: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_lang: Option<String>,
    usage_type: String,
    app_information: AppInformation,
}

#[derive(Deserialize)]
struct TranslationItem {
    text: String,
    #[serde(default)]
    detected_source_language: String,
}

#[derive(Deserialize)]
struct OneshotResponse {
    translations: Vec<TranslationItem>,
}

// ── Legacy /translate API types ──

#[derive(Deserialize)]
struct TranslateRequest {
    text: String,
    source_lang: Option<String>,
    target_lang: String,
    #[allow(dead_code)]
    quality: Option<String>,
}

#[derive(Serialize)]
struct TranslateResponse {
    code: u16,
    data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_lang: Option<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    message: String,
}

// ── Official /v2/* API types (mirrors DeepL's public API) ──

#[derive(Deserialize)]
struct OfficialTranslateRequest {
    /// DeepL accepts a single string or an array of strings.
    #[serde(deserialize_with = "deserialize_text_array")]
    text: Vec<String>,
    target_lang: String,
    source_lang: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    formality: Option<String>,
}

/// Accept either a single string or an array of strings for the `text` field,
/// normalizing to `Vec<String>`. This mirrors DeepL's lenient parsing.
fn deserialize_text_array<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StrOrArray {
        Single(String),
        Multiple(Vec<String>),
    }

    match StrOrArray::deserialize(deserializer)? {
        StrOrArray::Single(s) => Ok(vec![s]),
        StrOrArray::Multiple(v) => Ok(v),
    }
}

#[derive(Serialize)]
struct OfficialTranslation {
    detected_source_language: String,
    text: String,
}

#[derive(Serialize)]
struct OfficialTranslateResponse {
    translations: Vec<OfficialTranslation>,
}

#[derive(Serialize)]
struct LanguageEntry {
    language: &'static str,
    name: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    supports_formality: Option<bool>,
}

#[derive(Deserialize)]
struct LanguagesQuery {
    #[serde(rename = "type", default)]
    lang_type: String, // "source" | "target"; default behaves like "target"
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

// ═══════════════════════════════════════════════════════════════
//  Client
// ═══════════════════════════════════════════════════════════════

struct DeepLClient {
    client: Client,
    dl_session: Option<String>,
}

impl DeepLClient {
    async fn new(
        proxy: Option<String>,
        dl_session: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let builder = Client::builder()
            .emulation(Emulation::Chrome130)
            .cookie_store(true)
            .connect_timeout(Duration::from_secs(20))
            .timeout(Duration::from_secs(20));

        let builder = if let Some(ref p) = proxy {
            if !p.is_empty() {
                if p.starts_with("socks5h://") || p.starts_with("socks5://")
                    || p.starts_with("socks4a://") || p.starts_with("socks4://")
                {
                    // SOCKS proxies handle all traffic (HTTP + HTTPS)
                    builder.proxy(wreq::Proxy::all(p)?)
                } else if p.starts_with("http://") {
                    builder.proxy(wreq::Proxy::http(p)?)
                } else if p.starts_with("https://") {
                    builder.proxy(wreq::Proxy::https(p)?)
                } else {
                    // Default to HTTPS proxy for backward compatibility
                    builder.proxy(wreq::Proxy::https(p)?)
                }
            } else {
                builder
            }
        } else {
            builder
        };

        let client = builder.build()?;
        let deepl = DeepLClient {
            client,
            dl_session,
        };

        deepl.warmup_cookies().await?;
        Ok(deepl)
    }

    async fn warmup_cookies(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.client
            .get("https://www.deepl.com/translator")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .send()
            .await?;
        Ok(())
    }

    /// Translate a single chunk of text (≤ MAX_TEXT_LENGTH chars). One upstream
    /// round-trip, no retry. Returns (translated_text, detected_source_lang).
    async fn translate_single(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<(String, Option<String>), TranslateError> {
        if text.trim().is_empty() {
            return Err(TranslateError::Client("text cannot be empty".to_string()));
        }
        if count_chars(text) > MAX_TEXT_LENGTH {
            return Err(TranslateError::Client(format!(
                "text exceeds {} characters",
                MAX_TEXT_LENGTH
            )));
        }

        let target = match resolve_target_lang(target_lang) {
            Ok(t) => t,
            Err(e) => return Err(TranslateError::Client(format!("target_lang: {}", e))),
        };
        let source = match resolve_source_lang(source_lang) {
            Ok(s) => s,
            Err(e) => return Err(TranslateError::Client(format!("source_lang: {}", e))),
        };

        let endpoint = if self.dl_session.is_some() {
            ONESHOT_PRO
        } else {
            ONESHOT_FREE
        };

        let auth_value = match &self.dl_session {
            Some(s) => format!("Bearer {}", s),
            None => "None".to_string(),
        };

        let body = OneshotRequest {
            text: vec![text.to_string()],
            target_lang: target,
            source_lang: source.map(|s| s.to_string()),
            usage_type: "Translate".to_string(),
            app_information: AppInformation {
                os: "brex_macOS".to_string(),
                os_version: "brex_chrome_120.0.0.0".to_string(),
                app_version: "1.86.0".to_string(),
                app_build: "chrome_web_store".to_string(),
                instance_id: INSTANCE_ID.clone(),
            },
        };

        let resp = match self
            .client
            .post(endpoint)
            .header("Authorization", &auth_value)
            .header("Origin", "chrome-extension://cofdbpoegempjloogbagkncekinflcnj")
            .header("Accept", "*/*")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Sec-Fetch-Site", "cross-site")
            .header("Sec-Fetch-Mode", "cors")
            .header("Sec-Fetch-Dest", "empty")
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            // wreq surfaces connect/timeout/reset failures here; classify by
            // category rather than the (version-unstable) error message text.
            Err(_) => return Err(TranslateError::Network),
        };

        let status = resp.status();
        let body_bytes = match resp.bytes().await {
            Ok(b) => b,
            Err(_) => return Err(TranslateError::Network),
        };

        if status == 429 {
            return Err(TranslateError::RateLimit);
        }
        if !status.is_success() {
            let body_str = String::from_utf8_lossy(&body_bytes);
            return Err(TranslateError::Upstream(format!("HTTP {}: {}", status, body_str)));
        }

        let result: OneshotResponse = serde_json::from_slice(&body_bytes)
            .map_err(|e| TranslateError::Upstream(format!("json parse: {}", e)))?;

        if result.translations.is_empty() {
            return Err(TranslateError::Upstream("no translations in response".to_string()));
        }

        let translated = result.translations[0].text.clone();
        let detected = result.translations[0].detected_source_language.clone();
        let detected = if detected.is_empty() { None } else { Some(detected) };

        Ok((translated, detected))
    }

    /// Wrap a single-chunk translation with one automatic retry. On a retryable
    /// error (429 / network), refresh cookies (best-effort) and back off before
    /// the single second attempt. Non-retryable errors are returned immediately.
    async fn translate_with_retry(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<(String, Option<String>), TranslateError> {
        match self.translate_single(text, source_lang, target_lang).await {
            Ok(ok) => Ok(ok),
            Err(err) if err.is_retryable() => {
                let _ = self.warmup_cookies().await;
                tokio::time::sleep(Duration::from_secs(1)).await;
                self.translate_single(text, source_lang, target_lang).await
            }
            Err(err) => Err(err),
        }
    }

    /// Translate `text`, transparently splitting it into chunks when it exceeds
    /// `MAX_TEXT_LENGTH`. Chunks are translated sequentially and stitched back
    /// in order; `detected_source_language` is taken from the first chunk.
    ///
    /// Sequential (rather than concurrent) execution is deliberate: fanning out
    /// N simultaneous requests would multiply the chance of hitting DeepL's 429
    /// limit and work against the per-chunk retry logic. Long inputs are
    /// uncommon, so the small latency cost is an acceptable trade-off.
    async fn translate_chunked(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<(String, Option<String>), TranslateError> {
        if count_chars(text) <= MAX_TEXT_LENGTH {
            return self.translate_with_retry(text, source_lang, target_lang).await;
        }

        let chunks = chunk_text(text, MAX_TEXT_LENGTH);
        if chunks.is_empty() {
            return Err(TranslateError::Client("text cannot be empty".to_string()));
        }

        let mut translated_parts: Vec<String> = Vec::with_capacity(chunks.len());
        let mut detected: Option<String> = None;

        for chunk in &chunks {
            let (part, det) = self.translate_with_retry(chunk, source_lang, target_lang).await?;
            if detected.is_none() {
                detected = det;
            }
            translated_parts.push(part);
        }

        Ok((translated_parts.concat(), detected))
    }

    /// Translate a batch of independent texts. Each entry is chunked
    /// independently and results are returned in input order.
    async fn translate_batch(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<(String, Option<String>)>, TranslateError> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.translate_chunked(text, source_lang, target_lang).await?);
        }
        Ok(results)
    }
}

// ═══════════════════════════════════════════════════════════════
//  Axum handlers
// ═══════════════════════════════════════════════════════════════

#[derive(Clone)]
struct AppState {
    client: Arc<DeepLClient>,
}

/// Map a TranslateError to an HTTP (status, ErrorResponse) tuple.
fn err_to_response(err: TranslateError) -> (StatusCode, Json<ErrorResponse>) {
    let status = match &err {
        TranslateError::RateLimit => StatusCode::TOO_MANY_REQUESTS,
        // Network and generic upstream failures both surface as 502 Bad Gateway.
        TranslateError::Network | TranslateError::Upstream(_) => StatusCode::BAD_GATEWAY,
        TranslateError::Client(s) => {
            if s.contains("exceeds") {
                StatusCode::PAYLOAD_TOO_LARGE
            } else {
                StatusCode::BAD_REQUEST
            }
        }
    };
    (status, Json(ErrorResponse {
        code: status.as_u16(),
        message: err.to_string(),
    }))
}

async fn handle_translate(
    State(state): State<AppState>,
    Json(req): Json<TranslateRequest>,
) -> Result<Json<TranslateResponse>, (StatusCode, Json<ErrorResponse>)> {
    let source = req.source_lang.as_deref().unwrap_or("auto");

    match state
        .client
        .translate_chunked(&req.text, source, &req.target_lang)
        .await
    {
        Ok((data, detected)) => Ok(Json(TranslateResponse {
            code: 200,
            data,
            source_lang: detected,
        })),
        Err(e) => Err(err_to_response(e)),
    }
}

async fn handle_v2_translate(
    State(state): State<AppState>,
    Json(req): Json<OfficialTranslateRequest>,
) -> Result<Json<OfficialTranslateResponse>, (StatusCode, Json<ErrorResponse>)> {
    if req.text.is_empty() {
        return Err(err_to_response(TranslateError::Client(
            "text cannot be empty".to_string(),
        )));
    }
    // DeepL's official limit: at most 50 input texts per request.
    if req.text.len() > 50 {
        return Err(err_to_response(TranslateError::Client(
            "too many texts: maximum 50 per request".to_string(),
        )));
    }

    let source = req.source_lang.as_deref().unwrap_or("auto");

    let results = state
        .client
        .translate_batch(&req.text, source, &req.target_lang)
        .await
        .map_err(err_to_response)?;

    let translations = results
        .into_iter()
        .map(|(text, detected)| OfficialTranslation {
            detected_source_language: detected.unwrap_or_default(),
            text,
        })
        .collect();

    Ok(Json(OfficialTranslateResponse { translations }))
}

async fn handle_v2_languages(
    Query(q): Query<LanguagesQuery>,
) -> Json<Vec<LanguageEntry>> {
    // "source" only omits supports_formality; everything else (including the
    // default and unknown values) behaves like "target".
    let is_source = q.lang_type.eq_ignore_ascii_case("source");

    let entries: Vec<LanguageEntry> = LANGUAGES
        .iter()
        .map(|l| LanguageEntry {
            language: l.code,
            name: l.name,
            supports_formality: if is_source { None } else { Some(l.formality) },
        })
        .collect();

    Json(entries)
}

async fn handle_health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

// ═══════════════════════════════════════════════════════════════
//  Main
// ═══════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv(); // load .env file if present

    let proxy = std::env::var("PROXY_LIST").ok();
    let dl_session = std::env::var("DEEPL_DL_SESSION")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "9000".to_string());
    let addr = format!("{}:{}", host, port);

    let endpoint_label = if dl_session.is_some() { "Pro" } else { "Free" };
    println!("[*] Initializing DeepL client ({} endpoint)...", endpoint_label);
    let client = DeepLClient::new(proxy, dl_session).await?;
    println!("[*] Client ready (cookies warmed)");
    println!("[*] Listening on {}", addr);

    let state = AppState {
        client: Arc::new(client),
    };

    let referrer = SetResponseHeaderLayer::appending(
        HeaderName::from_static("referrer-policy"),
        axum::http::HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // Legacy convenience endpoint (single text, custom response shape).
        .route("/translate", post(handle_translate))
        // DeepL official-API-compatible endpoints.
        .route("/v2/translate", post(handle_v2_translate))
        .route("/v2/languages", get(handle_v2_languages))
        // Liveness probe.
        .route("/health", get(handle_health))
        .layer(referrer)
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// ═══════════════════════════════════════════════════════════════
//  Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn count(s: &str) -> usize {
        s.chars().count()
    }

    #[test]
    fn chunk_short_text_is_single_chunk() {
        let chunks = chunk_text("Hello, world!", 1500);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Hello, world!");
    }

    #[test]
    fn chunk_empty_text_returns_one_empty_chunk() {
        let chunks = chunk_text("", 1500);
        assert_eq!(chunks, vec!["".to_string()]);
    }

    #[test]
    fn chunk_respects_max_chars_exactly() {
        // 3000 'a' chars, no boundaries → hard-cut chunks of 1500.
        let text = "a".repeat(3000);
        let chunks = chunk_text(&text, 1500);
        assert_eq!(chunks.len(), 2);
        for c in &chunks {
            assert_eq!(count(c), 1500);
        }
        assert_eq!(chunks.concat(), text);
    }

    #[test]
    fn chunk_at_exact_boundary_returns_one_chunk() {
        let text = "a".repeat(1500);
        let chunks = chunk_text(&text, 1500);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn chunk_prefers_blank_lines() {
        // Two paragraphs joined by a blank line, split with a small max so the
        // blank line is the chosen boundary.
        let text = format!("{}\n\n{}", "a".repeat(10), "b".repeat(10));
        let chunks = chunk_text(&text, 15);
        assert!(chunks.len() >= 2);
        for c in &chunks {
            assert!(count(c) <= 15);
        }
        assert_eq!(chunks.concat(), text);
    }

    #[test]
    fn chunk_prefers_sentence_punctuation() {
        let text = "Hello world. Another sentence here.";
        let chunks = chunk_text(text, 15);
        for c in &chunks {
            assert!(count(c) <= 15);
        }
        // Reassembly must be lossless.
        assert_eq!(chunks.concat(), text);
    }

    #[test]
    fn chunk_mixed_cjk_and_ascii() {
        // Mix of CJK and ASCII to verify char-count (not byte) boundaries.
        let text = format!("{}。{}。", "你".repeat(800), "a".repeat(800));
        let chunks = chunk_text(&text, 1500);
        for c in &chunks {
            assert!(count(c) <= 1500);
        }
        assert_eq!(chunks.concat(), text);
    }

    #[test]
    fn chunk_no_boundary_hard_cuts_and_reassembles() {
        // 4000 chars of an unusual unicode char with no boundaries.
        let text = "字".repeat(4000);
        let chunks = chunk_text(&text, 1500);
        assert!(chunks.len() >= 3);
        for c in &chunks {
            assert!(count(c) <= 1500);
        }
        assert_eq!(chunks.concat(), text);
    }

    #[test]
    fn lang_aliases_resolve() {
        assert_eq!(resolve_target_lang("EN").unwrap(), "en-US");
        assert_eq!(resolve_target_lang("en").unwrap(), "en-US");
        assert_eq!(resolve_target_lang("PT").unwrap(), "pt-BR");
        assert_eq!(resolve_target_lang("ZH").unwrap(), "zh-Hans");
        assert_eq!(resolve_target_lang("ZH-HANT").unwrap(), "zh-Hant");
        assert_eq!(resolve_target_lang("zh_hans").unwrap(), "zh-Hans");
        assert!(resolve_target_lang("XX").is_err());
    }

    #[test]
    fn source_lang_auto_returns_none() {
        assert_eq!(resolve_source_lang("").unwrap(), None);
        assert_eq!(resolve_source_lang("auto").unwrap(), None);
        assert_eq!(resolve_source_lang("AUTO").unwrap(), None);
        assert_eq!(resolve_source_lang("EN").unwrap(), Some("en-US"));
    }

    #[test]
    fn languages_table_complete() {
        // Sanity: every entry has non-empty fields and unique codes.
        let mut seen = std::collections::HashSet::new();
        for l in LANGUAGES {
            assert!(!l.code.is_empty());
            assert!(!l.name.is_empty());
            assert!(!l.internal.is_empty());
            assert!(seen.insert(l.code), "duplicate code: {}", l.code);
        }
        // The documented formality-supporting languages are flagged.
        let formality_true: Vec<&str> = LANGUAGES
            .iter()
            .filter(|l| l.formality)
            .map(|l| l.code)
            .collect();
        for expected in ["DE", "FR", "IT", "ES", "ES-419", "NL", "PL", "PT-BR", "PT-PT", "RU", "JA"] {
            assert!(
                formality_true.contains(&expected),
                "{} should support formality",
                expected
            );
        }
    }
}
