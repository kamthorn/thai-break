use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "High-performance Thai Word Segmentation Web API Service")]
pub struct Config {
    /// Host address to bind to
    #[arg(long, env = "HOST", default_value = "0.0.0.0")]
    pub host: String,

    /// Port number to listen on
    #[arg(short, long, env = "PORT", default_value_t = 8080)]
    pub port: u16,

    /// Optional path to words FST dictionary (falls back to embedded words.fst)
    #[arg(long, env = "THAIBREAK_WORDS_FST_PATH", alias = "dict-path")]
    pub words_fst_path: Option<String>,

    /// Optional path to lines FST dictionary (falls back to embedded lines.fst)
    #[arg(long, env = "THAIBREAK_LINES_FST_PATH")]
    pub lines_fst_path: Option<String>,

    /// Optional path to bigrams.tsv file
    #[arg(long, env = "THAIBREAK_BIGRAM_PATH")]
    pub bigram_path: Option<String>,

    /// Maximum request body size in Megabytes (default: 4 MB)
    #[arg(long, env = "MAX_BODY_MB", default_value_t = 4)]
    pub max_body_mb: usize,

    /// Maximum concurrent requests processed simultaneously (default: 200)
    #[arg(long, env = "CONCURRENCY_LIMIT", default_value_t = 200)]
    pub concurrency_limit: usize,

    /// Request timeout in seconds (default: 30)
    #[arg(long, env = "REQUEST_TIMEOUT_SECS", default_value_t = 30)]
    pub request_timeout_secs: u64,

    /// Tracing / log level filter (e.g. "info", "debug", "warn")
    #[arg(long, env = "RUST_LOG", default_value = "info")]
    pub log_level: String,
}
