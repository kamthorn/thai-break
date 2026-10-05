use serde::{Deserialize, Serialize};

/// Mode for unified segmentation endpoint
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BreakMode {
    /// Tokenize into an array of words
    #[default]
    Words,
    /// Insert break marker (e.g. ZWSP or <wbr>) for typographic line breaking
    Lines,
    /// Hard-wrap text into lines based on visual display width
    Wrap,
    /// Join tokenized words with a custom delimiter
    Join,
}

/// Common request options for tokenization and line breaking
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BreakOptions {
    /// Whether to treat input as HTML/XML, preserving tags and entities untouched
    #[serde(default)]
    pub is_html: bool,

    /// Break marker to insert in "lines" mode (default: "\u{200B}" Zero-Width Space)
    #[serde(default)]
    pub marker: Option<String>,

    /// Maximum visual column width in "wrap" mode (default: 80)
    #[serde(default)]
    pub width: Option<usize>,

    /// Delimiter string used in "join" mode (default: "|")
    #[serde(default)]
    pub delimiter: Option<String>,

    /// Whether to keep whitespace tokens when tokenizing (default: false)
    #[serde(default)]
    pub keep_whitespace: bool,

    /// Whether to normalize Thai text before processing (default: false)
    #[serde(default)]
    pub normalize: bool,
}

impl Default for BreakOptions {
    fn default() -> Self {
        Self {
            is_html: false,
            marker: None,
            width: Some(80),
            delimiter: Some("|".to_string()),
            keep_whitespace: false,
            normalize: false,
        }
    }
}

/// Request payload supporting either a single `text` or multiple `texts` (batch)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BreakRequest {
    /// Processing mode: "words", "lines", "wrap", or "join"
    #[serde(default)]
    pub mode: BreakMode,

    /// Single text to process
    pub text: Option<String>,

    /// Batch texts to process
    pub texts: Option<Vec<String>>,

    /// Options for segmentation and line breaking
    #[serde(flatten)]
    pub options: BreakOptions,
}

/// Dedicated words/tokenize request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WordsRequest {
    /// Single text
    pub text: Option<String>,

    /// Batch texts
    pub texts: Option<Vec<String>>,

    /// Whether to treat input as HTML (only tokenizing text outside tags)
    #[serde(default)]
    pub is_html: bool,

    /// Whether to keep whitespace tokens (default: false)
    #[serde(default)]
    pub keep_whitespace: bool,

    /// Whether to normalize Thai text before tokenization (default: false)
    #[serde(default)]
    pub normalize: bool,
}

/// Dedicated lines request (typographic line breaker for HTML / PDF / Web)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinesRequest {
    /// Single text
    pub text: Option<String>,

    /// Batch texts
    pub texts: Option<Vec<String>>,

    /// Break marker (default: "\u{200B}" Zero-Width Space, or "<wbr>", "&shy;", etc.)
    #[serde(default)]
    pub marker: Option<String>,

    /// Whether to preserve HTML/XML tags and entities
    #[serde(default)]
    pub is_html: bool,

    /// Whether to normalize Thai text before line breaking (default: false)
    #[serde(default)]
    pub normalize: bool,
}

/// Dedicated wrap request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WrapRequest {
    /// Single text
    pub text: Option<String>,

    /// Batch texts
    pub texts: Option<Vec<String>>,

    /// Target display width per line (default: 80)
    #[serde(default = "default_width")]
    pub width: usize,

    /// Whether to preserve HTML/XML tags
    #[serde(default)]
    pub is_html: bool,

    /// Whether to normalize Thai text before wrapping (default: false)
    #[serde(default)]
    pub normalize: bool,
}

/// Dedicated normalize request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NormalizeRequest {
    /// Single text
    pub text: Option<String>,

    /// Batch texts
    pub texts: Option<Vec<String>>,
}

fn default_width() -> usize {
    80
}

/// Single item result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BreakResult {
    Words(Vec<String>),
    Text(String),
}

/// Response payload for single text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleResponse<T> {
    pub success: bool,
    pub data: T,
}

/// Response payload for batch texts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse<T> {
    pub success: bool,
    pub count: usize,
    pub data: Vec<T>,
}

/// General service status / info response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub engine: &'static str,
    pub base_dictionary_words: usize,
    pub words_dictionary_words: usize,
    pub lines_dictionary_words: usize,
    pub status: &'static str,
}

/// Request payload for comparing Words vs Lines segmentation
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompareRequest {
    pub text: String,
    #[serde(default)]
    pub is_html: bool,
    /// Whether to normalize Thai text before comparing (default: false)
    #[serde(default)]
    pub normalize: bool,
}

/// Response payload for comparing Words vs Lines segmentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareResponse {
    pub text: String,
    pub words: Vec<String>,
    pub lines_break: String,
    pub explanation: &'static str,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub timestamp: i64,
}

/// Error response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
}

/// Request payload for BahtText conversion
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BahttextRequest {
    pub amount: Option<f64>,
    pub text: Option<String>,
}

/// Query parameters for BahtText GET endpoint
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BahttextQuery {
    pub amount: Option<f64>,
    pub text: Option<String>,
}

/// Request payload for Thai ID validation
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidateIdRequest {
    pub id: String,
}

/// Query parameters for Thai ID validation GET endpoint
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidateIdQuery {
    pub id: Option<String>,
}
