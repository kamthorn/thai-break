use std::time::{SystemTime, UNIX_EPOCH};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::models::{
    BahttextQuery, BahttextRequest, BatchResponse, BreakMode, BreakRequest, BreakResult,
    CompareRequest, CompareResponse, ErrorResponse, HealthResponse, LinesRequest, NormalizeRequest,
    ServiceInfo, SingleResponse, ValidateIdQuery, ValidateIdRequest, WordsRequest, WrapRequest,
};
use crate::service::SharedEngine;

/// Maximum length limits to prevent resource exhaustion / amplification attacks
pub const MAX_MARKER_LEN: usize = 64;
pub const MAX_DELIMITER_LEN: usize = 64;
pub const MAX_BATCH_SIZE: usize = 1_000;
pub const MAX_TEXT_LEN: usize = 500_000;

#[inline]
pub fn validate_text(text: &str) -> Result<(), &'static str> {
    if text.len() > MAX_TEXT_LEN && text.chars().count() > MAX_TEXT_LEN {
        return Err("Input text exceeds maximum allowed limit of 500,000 characters");
    }
    Ok(())
}

#[inline]
pub fn validate_batch(texts: &[String]) -> Result<(), &'static str> {
    if texts.len() > MAX_BATCH_SIZE {
        return Err("Batch 'texts' exceeds maximum allowed limit of 1,000 items per request");
    }
    for t in texts {
        validate_text(t)?;
    }
    Ok(())
}

#[inline]
pub fn validate_marker(marker: Option<&str>) -> Result<(), &'static str> {
    if marker.is_some_and(|m| m.chars().count() > MAX_MARKER_LEN) {
        return Err("The 'marker' parameter exceeds maximum allowed limit of 64 characters");
    }
    Ok(())
}

#[inline]
pub fn validate_delimiter(delim: Option<&str>) -> Result<(), &'static str> {
    if delim.is_some_and(|d| d.chars().count() > MAX_DELIMITER_LEN) {
        return Err("The 'delimiter' parameter exceeds maximum allowed limit of 64 characters");
    }
    Ok(())
}

/// Helper to build error response
pub fn bad_request(msg: impl Into<String>) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            success: false,
            error: msg.into(),
        }),
    )
        .into_response()
}

/// GET /api/v1/info or GET /
pub async fn get_info(State(engine): State<SharedEngine>) -> impl IntoResponse {
    Json(ServiceInfo {
        name: "thai-break-service",
        version: env!("CARGO_PKG_VERSION"),
        engine: "thaibreak-rust",
        base_dictionary_words: engine.base_word_count(),
        words_dictionary_words: engine.words_dict_count(),
        lines_dictionary_words: engine.lines_dict_count(),
        status: "ready",
    })
}

/// GET /health or GET /readyz or GET /livez
pub async fn health_check() -> impl IntoResponse {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    Json(HealthResponse {
        status: "ok",
        timestamp: now,
    })
}

/// POST /api/v1/words or POST /api/v1/tokenize
pub async fn handle_words(
    State(engine): State<SharedEngine>,
    Json(req): Json<WordsRequest>,
) -> Response {
    if let Some(text) = req.text {
        if let Err(e) = validate_text(&text) {
            return bad_request(e);
        }
        let words = engine.words(&text, req.keep_whitespace, req.is_html, req.normalize);
        return Json(SingleResponse {
            success: true,
            data: words,
        })
        .into_response();
    }

    if let Some(texts) = req.texts {
        if let Err(e) = validate_batch(&texts) {
            return bad_request(e);
        }
        let batch: Vec<Vec<String>> = texts
            .into_iter()
            .map(|t| engine.words(&t, req.keep_whitespace, req.is_html, req.normalize))
            .collect();
        let count = batch.len();
        return Json(BatchResponse {
            success: true,
            count,
            data: batch,
        })
        .into_response();
    }

    bad_request("Either 'text' (string) or 'texts' (array of strings) must be provided")
}

/// POST /api/v1/lines
pub async fn handle_lines(
    State(engine): State<SharedEngine>,
    Json(req): Json<LinesRequest>,
) -> Response {
    if let Err(e) = validate_marker(req.marker.as_deref()) {
        return bad_request(e);
    }
    let marker = req.marker.as_deref();

    if let Some(text) = req.text {
        if let Err(e) = validate_text(&text) {
            return bad_request(e);
        }
        let result = engine.lines(&text, marker, req.is_html, req.normalize);
        return Json(SingleResponse {
            success: true,
            data: result,
        })
        .into_response();
    }

    if let Some(texts) = req.texts {
        if let Err(e) = validate_batch(&texts) {
            return bad_request(e);
        }
        let batch: Vec<String> = texts
            .into_iter()
            .map(|t| engine.lines(&t, marker, req.is_html, req.normalize))
            .collect();
        let count = batch.len();
        return Json(BatchResponse {
            success: true,
            count,
            data: batch,
        })
        .into_response();
    }

    bad_request("Either 'text' (string) or 'texts' (array of strings) must be provided")
}

/// POST /api/v1/wrap
pub async fn handle_wrap(
    State(engine): State<SharedEngine>,
    Json(req): Json<WrapRequest>,
) -> Response {
    if let Some(text) = req.text {
        if let Err(e) = validate_text(&text) {
            return bad_request(e);
        }
        let result = engine.wrap(&text, req.width, req.is_html, req.normalize);
        return Json(SingleResponse {
            success: true,
            data: result,
        })
        .into_response();
    }

    if let Some(texts) = req.texts {
        if let Err(e) = validate_batch(&texts) {
            return bad_request(e);
        }
        let batch: Vec<String> = texts
            .into_iter()
            .map(|t| engine.wrap(&t, req.width, req.is_html, req.normalize))
            .collect();
        let count = batch.len();
        return Json(BatchResponse {
            success: true,
            count,
            data: batch,
        })
        .into_response();
    }

    bad_request("Either 'text' (string) or 'texts' (array of strings) must be provided")
}

/// POST /api/v1/normalize
pub async fn handle_normalize(
    State(engine): State<SharedEngine>,
    Json(req): Json<NormalizeRequest>,
) -> Response {
    if let Some(text) = req.text {
        if let Err(e) = validate_text(&text) {
            return bad_request(e);
        }
        let result = engine.normalize(&text);
        return Json(SingleResponse {
            success: true,
            data: result,
        })
        .into_response();
    }

    if let Some(texts) = req.texts {
        if let Err(e) = validate_batch(&texts) {
            return bad_request(e);
        }
        let batch: Vec<String> = texts
            .into_iter()
            .map(|t| engine.normalize(&t))
            .collect();
        let count = batch.len();
        return Json(BatchResponse {
            success: true,
            count,
            data: batch,
        })
        .into_response();
    }

    bad_request("Either 'text' (string) or 'texts' (array of strings) must be provided")
}

/// POST /api/v1/break (Unified endpoint for words, lines, wrap, join)
pub async fn handle_break(
    State(engine): State<SharedEngine>,
    Json(req): Json<BreakRequest>,
) -> Response {
    if let Err(e) = validate_marker(req.options.marker.as_deref()) {
        return bad_request(e);
    }
    if let Err(e) = validate_delimiter(req.options.delimiter.as_deref()) {
        return bad_request(e);
    }

    let process_item = |text: &str| -> BreakResult {
        match req.mode {
            BreakMode::Words => {
                let w = engine.words(
                    text,
                    req.options.keep_whitespace,
                    req.options.is_html,
                    req.options.normalize,
                );
                BreakResult::Words(w)
            }
            BreakMode::Lines => {
                let l = engine.lines(
                    text,
                    req.options.marker.as_deref(),
                    req.options.is_html,
                    req.options.normalize,
                );
                BreakResult::Text(l)
            }
            BreakMode::Wrap => {
                let w = engine.wrap(
                    text,
                    req.options.width.unwrap_or(80),
                    req.options.is_html,
                    req.options.normalize,
                );
                BreakResult::Text(w)
            }
            BreakMode::Join => {
                let sep = req.options.delimiter.as_deref().unwrap_or("|");
                let j = engine.join(text, sep, req.options.is_html, req.options.normalize);
                BreakResult::Text(j)
            }
        }
    };

    if let Some(text) = &req.text {
        if let Err(e) = validate_text(text) {
            return bad_request(e);
        }
        let result = process_item(text);
        return Json(SingleResponse {
            success: true,
            data: result,
        })
        .into_response();
    }

    if let Some(texts) = &req.texts {
        if let Err(e) = validate_batch(texts) {
            return bad_request(e);
        }
        let batch: Vec<BreakResult> = texts.iter().map(|t| process_item(t)).collect();
        let count = batch.len();
        return Json(BatchResponse {
            success: true,
            count,
            data: batch,
        })
        .into_response();
    }

    bad_request("Either 'text' (string) or 'texts' (array of strings) must be provided")
}

/// POST /api/v1/compare
pub async fn handle_compare(
    State(engine): State<SharedEngine>,
    Json(req): Json<CompareRequest>,
) -> Response {
    if req.text.is_empty() {
        return bad_request("The 'text' field is required and cannot be empty.");
    }
    if let Err(e) = validate_text(&req.text) {
        return bad_request(e);
    }

    let words = engine.words(&req.text, false, req.is_html, req.normalize);
    let lines_break = engine.lines(&req.text, Some("|"), req.is_html, req.normalize);

    Json(SingleResponse {
        success: true,
        data: CompareResponse {
            text: req.text,
            words,
            lines_break,
            explanation: "Words mode retains proper names, compounds, and institutions intact for semantic completeness. Lines mode splits them into constituent units for flexible line wrapping.",
        },
    })
    .into_response()
}

/// POST /api/v1/bahttext
pub async fn handle_bahttext_post(
    Json(req): Json<BahttextRequest>,
) -> Response {
    if let Some(text) = req.text {
        match crate::utils::bahttext_str(&text) {
            Ok(result) => Json(SingleResponse { success: true, data: result }).into_response(),
            Err(e) => bad_request(e),
        }
    } else if let Some(amount) = req.amount {
        let result = crate::utils::bahttext(amount);
        Json(SingleResponse { success: true, data: result }).into_response()
    } else {
        bad_request("Either 'amount' (number) or 'text' (string) must be provided")
    }
}

/// GET /api/v1/bahttext?amount=... or ?text=...
pub async fn handle_bahttext_get(
    Query(query): Query<BahttextQuery>,
) -> Response {
    if let Some(text) = query.text {
        match crate::utils::bahttext_str(&text) {
            Ok(result) => Json(SingleResponse { success: true, data: result }).into_response(),
            Err(e) => bad_request(e),
        }
    } else if let Some(amount) = query.amount {
        let result = crate::utils::bahttext(amount);
        Json(SingleResponse { success: true, data: result }).into_response()
    } else {
        bad_request("Query parameter 'amount' (number) or 'text' (string) must be provided")
    }
}

/// POST /api/v1/validate-id
pub async fn handle_validate_id_post(
    Json(req): Json<ValidateIdRequest>,
) -> Response {
    let result = crate::utils::validate_thai_id(&req.id);
    Json(SingleResponse { success: true, data: result }).into_response()
}

/// GET /api/v1/validate-id?id=...
pub async fn handle_validate_id_get(
    Query(query): Query<ValidateIdQuery>,
) -> Response {
    if let Some(id) = query.id {
        let result = crate::utils::validate_thai_id(&id);
        Json(SingleResponse { success: true, data: result }).into_response()
    } else {
        bad_request("Query parameter 'id' (13-digit Thai National ID) is required")
    }
}
