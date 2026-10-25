use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use axum::{
    body::Body,
    http::{Method, Request, Response, StatusCode, header},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

use super::*;

const ORIGIN: &str = "http://localhost:5177";
static FIXTURE_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

fn state() -> AppState {
    AppState::without_database([ORIGIN.to_owned()])
}

fn fixture_directory() -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "grenadiers-browser-response-{}-{}",
        std::process::id(),
        FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    let assets = directory.join("assets");
    fs::create_dir_all(&assets).unwrap();
    fs::write(directory.join("index.html"), "<main>fresh index</main>").unwrap();
    fs::write(
        assets.join("Geometry-DKID-A9Y.js"),
        "export const hashed = true;",
    )
    .unwrap();
    fs::write(assets.join("app.js"), "export const unhashed = true;").unwrap();
    fs::write(
        assets.join("page-C4t9aB2_.html"),
        "<main>hashed html</main>",
    )
    .unwrap();
    directory
}

fn assert_browser_headers(response: &Response<Body>) {
    assert_eq!(response.headers()["x-content-type-options"], "nosniff");
    assert_eq!(response.headers()["x-frame-options"], "DENY");
    assert_eq!(response.headers()["referrer-policy"], "no-referrer");
    assert_eq!(
        response.headers()[header::CONTENT_SECURITY_POLICY],
        concat!(
            "default-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'; ",
            "connect-src 'self'; img-src 'self' data:; font-src 'none'; media-src 'none'; ",
            "worker-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'; ",
            "frame-ancestors 'none'"
        )
    );
    assert!(
        !response
            .headers()
            .contains_key("content-security-policy-report-only")
    );
}

async fn body_text(response: Response<Body>) -> String {
    String::from_utf8(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap()
}

#[tokio::test]
async fn html_and_spa_fallbacks_revalidate_instead_of_becoming_immutable() {
    let directory = fixture_directory();
    let router = app(state(), Some(directory.clone()));
    for (path, status) in [
        ("/", StatusCode::OK),
        ("/rooms/demo", StatusCode::OK),
        ("/assets/page-C4t9aB2_.html", StatusCode::OK),
    ] {
        let response = router
            .clone()
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), status, "{path}");
        assert_browser_headers(&response);
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "no-cache",
            "{path}"
        );
        assert!(
            response.headers()[header::CONTENT_TYPE]
                .to_str()
                .unwrap()
                .starts_with("text/html"),
            "{path}"
        );
        assert!(body_text(response).await.contains("<main>"), "{path}");
    }

    let missing = router
        .oneshot(
            Request::get("/assets/missing-C4t9aB2_.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::OK);
    assert_browser_headers(&missing);
    assert_eq!(missing.headers()[header::CACHE_CONTROL], "no-cache");
    assert!(
        missing.headers()[header::CONTENT_TYPE]
            .to_str()
            .unwrap()
            .starts_with("text/html")
    );
    assert!(body_text(missing).await.contains("fresh index"));
    fs::remove_dir_all(directory).unwrap();
}

#[tokio::test]
async fn only_successful_content_hashed_non_html_assets_are_immutable() {
    let directory = fixture_directory();
    let router = app(state(), Some(directory.clone()));
    for (path, cache) in [
        (
            "/assets/Geometry-DKID-A9Y.js",
            "public, max-age=31536000, immutable",
        ),
        ("/assets/app.js", "no-cache"),
    ] {
        let response = router
            .clone()
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK, "{path}");
        assert_browser_headers(&response);
        assert_eq!(response.headers()[header::CACHE_CONTROL], cache, "{path}");
        assert!(
            !response.headers()[header::CONTENT_TYPE]
                .to_str()
                .unwrap()
                .starts_with("text/html"),
            "{path}"
        );
    }

    let partial = router
        .clone()
        .oneshot(
            Request::get("/assets/Geometry-DKID-A9Y.js")
                .header(header::RANGE, "bytes=0-5")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(partial.status(), StatusCode::PARTIAL_CONTENT);
    assert_browser_headers(&partial);
    assert_eq!(
        partial.headers()[header::CACHE_CONTROL],
        "public, max-age=31536000, immutable"
    );

    let head = router
        .oneshot(
            Request::head("/assets/Geometry-DKID-A9Y.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(head.status(), StatusCode::OK);
    assert_browser_headers(&head);
    assert_eq!(
        head.headers()[header::CACHE_CONTROL],
        "public, max-age=31536000, immutable"
    );
    assert!(
        head.into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .is_empty()
    );
    fs::remove_dir_all(directory).unwrap();
}

#[tokio::test]
async fn html_conditional_revalidation_keeps_security_and_no_cache_headers() {
    let directory = fixture_directory();
    let router = app(state(), Some(directory.clone()));
    let initial = router
        .clone()
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let last_modified = initial.headers()[header::LAST_MODIFIED].clone();

    let response = router
        .oneshot(
            Request::get("/")
                .header(header::IF_MODIFIED_SINCE, last_modified)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
    assert_browser_headers(&response);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-cache");
    assert!(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .is_empty()
    );
    fs::remove_dir_all(directory).unwrap();
}

#[tokio::test]
async fn html_head_and_fallback_policy_does_not_trust_request_sources() {
    let directory = fixture_directory();
    let router = app(state(), Some(directory.clone()));
    for path in ["/", "/rooms/demo"] {
        for method in [Method::GET, Method::HEAD] {
            let response = router
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method.clone())
                        .uri(path)
                        .header(header::HOST, "untrusted.example:9999")
                        .header(header::ORIGIN, "https://untrusted.example")
                        .header("forwarded", "host=untrusted.example;proto=https")
                        .header("x-forwarded-host", "untrusted.example")
                        .header("x-forwarded-proto", "https")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            assert_browser_headers(&response);
            assert_eq!(response.headers()[header::CACHE_CONTROL], "no-cache");
            if method == Method::HEAD {
                assert!(body_text(response).await.is_empty());
            } else {
                assert!(body_text(response).await.contains("fresh index"));
            }
        }
    }
    fs::remove_dir_all(directory).unwrap();
}

#[tokio::test]
async fn api_and_websocket_responses_keep_no_store_and_gain_browser_headers() {
    let router = app(state(), None);
    for (request, status, allow) in [
        (
            Request::get("/api/v1/health").body(Body::empty()).unwrap(),
            StatusCode::OK,
            None,
        ),
        (
            Request::get("/api/v1/missing").body(Body::empty()).unwrap(),
            StatusCode::NOT_FOUND,
            None,
        ),
        (
            Request::post("/api/v1/health").body(Body::empty()).unwrap(),
            StatusCode::METHOD_NOT_ALLOWED,
            Some("GET,HEAD"),
        ),
        (
            Request::get("/api/v1/ws")
                .header(header::ORIGIN, ORIGIN)
                .body(Body::empty())
                .unwrap(),
            StatusCode::BAD_REQUEST,
            None,
        ),
    ] {
        let response = router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), status);
        assert_browser_headers(&response);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        if let Some(allow) = allow {
            assert_eq!(response.headers()[header::ALLOW], allow);
        }
    }
}
