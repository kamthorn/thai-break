use std::sync::Arc;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

use thai_break_service::{
    config::Config, create_router, service::ThaiBreakEngine,
};

fn setup_app() -> axum::Router {
    let config = Config {
        host: "127.0.0.1".into(),
        port: 8080,
        words_fst_path: None,
        lines_fst_path: None,
        bigram_path: None,
        max_body_mb: 4,
        concurrency_limit: 100,
        request_timeout_secs: 30,
        log_level: "error".into(),
    };
    let engine = Arc::new(ThaiBreakEngine::new(&config));
    create_router(engine, &config)
}

async fn response_json(app: axum::Router, req: Request<Body>) -> (StatusCode, Value) {
    let response = app.oneshot(req).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
    (status, json)
}

#[tokio::test]
async fn test_health_and_info() {
    let app = setup_app();

    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let (status, res) = response_json(app.clone(), req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(res["status"], "ok");

    let req = Request::builder()
        .uri("/api/v1/info")
        .body(Body::empty())
        .unwrap();
    let (status, res) = response_json(app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(res["name"], "thai-break-service");
    assert_eq!(res["engine"], "thaibreak-rust");
    assert!(res["words_dictionary_words"].as_u64().unwrap() > 20000);
}

#[tokio::test]
async fn test_words_single_and_batch() {
    let app = setup_app();

    // Single text
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/words")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "text": "ฉันรักภาษาไทย"
            })
            .to_string(),
        ))
        .unwrap();
    let (status, res) = response_json(app.clone(), req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(res["success"], true);
    assert_eq!(
        res["data"],
        json!(["ฉัน", "รัก", "ภาษา", "ไทย"])
    );

    // Batch texts
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/words")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "texts": ["สวัสดีครับ", "ราตรีสวัสดิ์"]
            })
            .to_string(),
        ))
        .unwrap();
    let (status, res) = response_json(app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(res["success"], true);
    assert_eq!(res["count"], 2);
    assert_eq!(
        res["data"][0],
        json!(["สวัสดี", "ครับ"])
    );
}

#[tokio::test]
async fn test_lines_with_html_preservation() {
    let app = setup_app();

    // HTML input with custom marker "|"
    let html_input = "<p><b>สวัสดี</b> ประเทศไทย &amp; โลก</p>";
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/lines")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "text": html_input,
                "marker": "|",
                "is_html": true
            })
            .to_string(),
        ))
        .unwrap();

    let (status, res) = response_json(app.clone(), req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(res["success"], true);
    let output = res["data"].as_str().unwrap();

    // Tags and entities must be untouched
    assert!(output.contains("<p><b>"));
    assert!(output.contains("</b>"));
    assert!(output.contains("&amp;"));
    assert!(output.contains("</p>"));
    // Word break marker should be present between words
    assert!(output.contains("ประเทศ|ไทย"));
}

#[tokio::test]
async fn test_wrap() {
    let app = setup_app();

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/wrap")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "text": "ฉันรักภาษาไทยมากที่สุดในโลก",
                "width": 12
            })
            .to_string(),
        ))
        .unwrap();

    let (status, res) = response_json(app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(res["success"], true);
    let wrapped = res["data"].as_str().unwrap();
    assert!(wrapped.contains('\n'));
}

#[tokio::test]
async fn test_unified_break_endpoint() {
    let app = setup_app();

    // Test mode: join
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/break")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "mode": "join",
                "text": "สวัสดีครับคุณลูกค้า",
                "delimiter": "/"
            })
            .to_string(),
        ))
        .unwrap();

    let (status, res) = response_json(app.clone(), req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(res["success"], true);
    assert_eq!(res["data"], "สวัสดี/ครับ/คุณ/ลูกค้า");

    // Test mode: words with HTML extraction
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/break")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "mode": "words",
                "text": "<div>สวัสดี <span>กรุงเทพ</span></div>",
                "is_html": true
            })
            .to_string(),
        ))
        .unwrap();

    let (status, res) = response_json(app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(res["success"], true);
    assert_eq!(
        res["data"],
        json!(["สวัสดี", "กรุงเทพ"])
    );
}

#[tokio::test]
async fn test_validation_error() {
    let app = setup_app();

    // Empty body without text or texts
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/words")
        .header("content-type", "application/json")
        .body(Body::from(json!({}).to_string()))
        .unwrap();

    let (status, res) = response_json(app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(res["success"], false);
    assert!(res["error"].as_str().unwrap().contains("must be provided"));
}

#[tokio::test]
async fn test_compare_endpoint() {
    let app = setup_app();

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/compare")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "text": "เดินทางไปกรมการกงสุลที่เกิดเหตุ"
        }).to_string()))
        .unwrap();

    let (status, res) = response_json(app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(res["success"], true);

    let words = res["data"]["words"].as_array().unwrap();
    assert!(!words.is_empty());

    let lines_break = res["data"]["lines_break"].as_str().unwrap();
    assert!(lines_break.contains('|'));
}

#[tokio::test]
async fn test_compare_endpoint_with_normalize() {
    let app = setup_app();

    let compare = |normalize: bool| {
        let app = app.clone();
        async move {
            let req = Request::builder()
                .method("POST")
                .uri("/api/v1/compare")
                .header("content-type", "application/json")
                .body(Body::from(json!({
                    "text": "ตัวอย่างพนักงานโรงแรมดีมากกก",
                    "normalize": normalize,
                }).to_string()))
                .unwrap();
            response_json(app, req).await
        }
    };

    let (status, plain) = compare(false).await;
    assert_eq!(status, StatusCode::OK);
    let (status, normalized) = compare(true).await;
    assert_eq!(status, StatusCode::OK);

    // Normalization must take effect: elongated "มากกก" collapses, so the
    // two modes produce different token streams.
    let plain_words = plain["data"]["words"].as_array().unwrap();
    let norm_words = normalized["data"]["words"].as_array().unwrap();
    assert_ne!(plain_words, norm_words);
    assert!(norm_words.iter().any(|w| w == "มาก"));
    assert!(!norm_words.iter().any(|w| w.as_str().unwrap().contains("กก")));
}

#[tokio::test]
async fn test_dual_mode_behavior() {
    let app = setup_app();

    // 1. Words mode: "กรมการกงสุล" and "จุฬาลงกรณ์มหาวิทยาลัย" should be kept as 1 semantic token
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/words")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "text": "ไปกรมการกงสุลและจุฬาลงกรณ์มหาวิทยาลัย"
        }).to_string()))
        .unwrap();

    let (status, res) = response_json(app.clone(), req).await;
    assert_eq!(status, StatusCode::OK);
    let words = res["data"].as_array().unwrap();
    let words_str: Vec<&str> = words.iter().map(|w| w.as_str().unwrap()).collect();
    assert!(words_str.contains(&"กรมการกงสุล"), "Words mode must keep proper name intact: {:?}", words_str);
    assert!(words_str.contains(&"จุฬาลงกรณ์มหาวิทยาลัย"), "Words mode must keep institution intact: {:?}", words_str);

    // 2. Lines mode: "กรมการกงสุล" and "จุฬาลงกรณ์มหาวิทยาลัย" should be broken for layout flexibility
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/lines")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "text": "ไปกรมการกงสุลและจุฬาลงกรณ์มหาวิทยาลัย",
            "marker": "|"
        }).to_string()))
        .unwrap();

    let (status, res) = response_json(app, req).await;
    assert_eq!(status, StatusCode::OK);
    let lines_str = res["data"].as_str().unwrap();
    // In lines mode, proper names break into constituent words
    assert!(lines_str.contains("กรมการ|กงสุล") || lines_str.contains("กรม|"), "Lines mode must allow breaking proper names: {}", lines_str);
    // In lines mode, institution names break into constituent words
    assert!(lines_str.contains("จุฬาลงกรณ์|มหาวิทยาลัย"), "Lines mode must allow breaking institution names: {}", lines_str);
}

#[tokio::test]
async fn test_security_headers() {
    let app = setup_app();

    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let nosniff = response
        .headers()
        .get("x-content-type-options")
        .expect("Must include x-content-type-options header");
    assert_eq!(nosniff, "nosniff");
}

#[tokio::test]
async fn test_mathematical_brackets_data_preservation() {
    let app = setup_app();

    let text = "เงื่อนไขคือ x < 5 และ y > 3";
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/words")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "text": text }).to_string()))
        .unwrap();

    let (status, res) = response_json(app.clone(), req).await;
    assert_eq!(status, StatusCode::OK);
    let words = res["data"].as_array().unwrap();
    let words_str: Vec<&str> = words.iter().map(|w| w.as_str().unwrap()).collect();

    // Verify all parts are preserved and "< 5 และ y >" was not stripped
    assert!(words_str.contains(&"<"), "Must preserve '<' token: {:?}", words_str);
    assert!(words_str.contains(&">"), "Must preserve '>' token: {:?}", words_str);
    assert!(words_str.contains(&"5"), "Must preserve '5' token: {:?}", words_str);
    assert!(words_str.contains(&"3"), "Must preserve '3' token: {:?}", words_str);
}

#[tokio::test]
async fn test_input_validation_limits() {
    let app = setup_app();

    // 1. Marker too long (> 64 chars)
    let long_marker = "A".repeat(65);
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/lines")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "text": "สวัสดีครับ",
            "marker": long_marker
        }).to_string()))
        .unwrap();

    let (status, res) = response_json(app.clone(), req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(res["error"].as_str().unwrap().contains("marker"));

    // 2. Batch too large (> 1,000 items)
    let big_batch: Vec<String> = (0..1001).map(|i| format!("คำที่ {}", i)).collect();
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/words")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "texts": big_batch
        }).to_string()))
        .unwrap();

    let (status, res) = response_json(app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(res["error"].as_str().unwrap().contains("Batch 'texts' exceeds maximum"));
}

#[tokio::test]
async fn test_normalize_endpoint() {
    let app = setup_app();

    // 1. Single text normalization
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/normalize")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "text": "ตัวอย่างพนักงานโรงแรมให้บริการต้อนรับดีมากก"
        }).to_string()))
        .unwrap();

    let (status, res) = response_json(app.clone(), req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(res["success"], true);
    assert_eq!(
        res["data"].as_str().unwrap(),
        "ตัวอย่างพนักงานโรงแรมให้บริการต้อนรับดีมาก"
    );

    // 2. Batch text normalization
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/normalize")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "texts": [
                "เเปลกมากกก",
                "นํ้าดื่มดีีีี",
                "พ ุ่มไม้สวยยยย"
            ]
        }).to_string()))
        .unwrap();

    let (status, res) = response_json(app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(res["success"], true);
    assert_eq!(res["count"], 3);
    assert_eq!(res["data"][0], "แปลกมาก");
    assert_eq!(res["data"][1], "น้ำดื่มดี");
    assert_eq!(res["data"][2], "พุ่มไม้สวย");
}

#[tokio::test]
async fn test_words_with_normalize_flag() {
    let app = setup_app();

    let text = "ตัวอย่างพนักงานโรงแรมให้บริการต้อนรับดีมากก";

    // A. Without normalization (default normalize=false)
    let req_raw = Request::builder()
        .method("POST")
        .uri("/api/v1/words")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "text": text,
            "normalize": false
        }).to_string()))
        .unwrap();

    let (status, res_raw) = response_json(app.clone(), req_raw).await;
    assert_eq!(status, StatusCode::OK);
    let raw_tokens: Vec<String> = res_raw["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert_eq!(raw_tokens.last().unwrap(), "กก");

    // B. With normalization (normalize=true)
    let req_norm = Request::builder()
        .method("POST")
        .uri("/api/v1/words")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "text": text,
            "normalize": true
        }).to_string()))
        .unwrap();

    let (status, res_norm) = response_json(app, req_norm).await;
    assert_eq!(status, StatusCode::OK);
    let norm_tokens: Vec<String> = res_norm["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert_eq!(norm_tokens.last().unwrap(), "มาก");
    assert!(!norm_tokens.contains(&"กก".to_string()));
}

#[tokio::test]
async fn test_bahttext_endpoint() {
    let app = setup_app();

    // 1. POST with numeric amount
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/bahttext")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "amount": 1234.50
        }).to_string()))
        .unwrap();

    let (status, res) = response_json(app.clone(), req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(res["success"], true);
    assert_eq!(res["data"], "หนึ่งพันสองร้อยสามสิบสี่บาทห้าสิบสตางค์");

    // 2. GET with query param text string
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/bahttext?text=100.00")
        .body(Body::empty())
        .unwrap();

    let (status, res) = response_json(app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(res["success"], true);
    assert_eq!(res["data"], "หนึ่งร้อยบาทถ้วน");
}

#[tokio::test]
async fn test_validate_id_endpoint() {
    let app = setup_app();

    // 1. POST valid ID
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/validate-id")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "id": "1234567890121"
        }).to_string()))
        .unwrap();

    let (status, res) = response_json(app.clone(), req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(res["success"], true);
    assert_eq!(res["data"]["is_valid"], true);
    assert_eq!(res["data"]["formatted"], "1-2345-67890-12-1");

    // 2. GET invalid ID with query param
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/validate-id?id=1234567890120")
        .body(Body::empty())
        .unwrap();

    let (status, res) = response_json(app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(res["success"], true);
    assert_eq!(res["data"]["is_valid"], false);
    assert!(res["data"]["error"].as_str().unwrap().contains("Invalid checksum"));
}



