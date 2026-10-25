use std::{fs, path::PathBuf};

use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use http_body_util::BodyExt;
use jsonschema::validator_for;
use serde_json::{Value, json};
use tower::ServiceExt;

use super::*;

const ORIGIN: &str = "http://localhost:5177";

fn state() -> AppState {
    AppState::without_database([ORIGIN.to_owned()])
}

fn problem_schema() -> Value {
    let contract: Value = serde_json::from_str(include_str!("../../api/openapi.json")).unwrap();
    contract["components"]["schemas"]["Problem"].clone()
}

async fn assert_problem(response: Response, status: StatusCode) {
    assert_eq!(response.status(), status);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    let body: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body["status"], status.as_u16());
    let errors = validator_for(&problem_schema())
        .unwrap()
        .iter_errors(&body)
        .map(|error| error.to_string())
        .collect::<Vec<_>>();
    assert!(errors.is_empty(), "schema errors: {errors:?}");
}

#[tokio::test]
async fn known_api_routes_return_problem_405_with_axum_allow_header() {
    let router = app(state(), None);
    for (method, path, allow) in [
        (Method::POST, "/api/v1/health", "GET,HEAD"),
        (Method::POST, "/api/v1/ready", "GET,HEAD"),
        (Method::PUT, "/api/v1/ws", "GET"),
        (Method::GET, "/api/v1/game/session", "POST"),
        (Method::POST, "/api/v1/game/ws", "GET"),
    ] {
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.headers()[header::ALLOW], allow, "{path}");
        assert_problem(response, StatusCode::METHOD_NOT_ALLOWED).await;
    }
}

#[tokio::test]
async fn head_remains_automatic_for_get_routes_and_has_no_body() {
    let router = app(state(), None);
    for path in ["/api/v1/health", "/api/v1/ready"] {
        let response = router
            .clone()
            .oneshot(Request::head(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK, "{path}");
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert!(
            response
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .is_empty()
        );
    }

    for path in ["/api/v1/ws", "/api/v1/game/ws"] {
        let response = router
            .clone()
            .oneshot(Request::head(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED, "{path}");
        assert_eq!(response.headers()[header::ALLOW], "GET", "{path}");
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "application/problem+json"
        );
        assert!(
            response.headers()[header::CONTENT_LENGTH]
                .to_str()
                .unwrap()
                .parse::<usize>()
                .unwrap()
                > 0
        );
        assert!(
            response
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .is_empty()
        );
    }
}

#[tokio::test]
async fn malformed_get_websocket_handshakes_remain_bad_requests() {
    let router = app(state(), None);
    for path in ["/api/v1/ws", "/api/v1/game/ws"] {
        let response = router
            .clone()
            .oneshot(
                Request::get(path)
                    .header(header::ORIGIN, ORIGIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{path}");
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    }
}

fn spa_directory() -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("grenadiers-http-fallback-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    fs::write(directory.join("index.html"), "<main>SPA fallback</main>").unwrap();
    directory
}

#[tokio::test]
async fn unknown_api_routes_return_problem_404_with_and_without_spa_fallback() {
    let directory = spa_directory();
    for web_dist_dir in [None, Some(directory.clone())] {
        for (method, path) in [
            (Method::GET, "/api/v1/missing"),
            (Method::POST, "/api/v1/missing"),
            (Method::DELETE, "/api"),
            (Method::PATCH, "/api/"),
        ] {
            let response = app(state(), web_dist_dir.clone())
                .oneshot(
                    Request::builder()
                        .method(method.clone())
                        .uri(path)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_problem(response, StatusCode::NOT_FOUND).await;
        }
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn openapi_defines_shared_not_found_and_method_not_allowed_problems() {
    let contract: Value = serde_json::from_str(include_str!("../../api/openapi.json")).unwrap();
    assert_eq!(
        contract["x-api-fallback-responses"]["404"]["$ref"],
        "#/components/responses/NotFound"
    );
    assert_eq!(
        contract["x-api-fallback-responses"]["405"]["$ref"],
        "#/components/responses/MethodNotAllowed"
    );
    assert_eq!(
        contract["components"]["responses"]["NotFound"]["content"]["application/problem+json"]["example"],
        json!({
            "type": "about:blank",
            "title": "Not Found",
            "status": 404,
            "detail": "API route does not exist"
        })
    );
    assert_eq!(
        contract["components"]["responses"]["MethodNotAllowed"]["content"]["application/problem+json"]
            ["example"]["status"],
        405
    );
}
