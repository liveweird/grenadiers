use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use axum::{
    body::Body,
    extract::connect_info::MockConnectInfo,
    http::{Request, StatusCode, header},
};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::net::TcpListener;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, client::IntoClientRequest},
};
use tower::ServiceExt;
use tracing::{Event, Subscriber, field::Visit, span::Attributes};
use tracing_subscriber::{Layer, layer::SubscriberExt};

use crate::{
    AppState,
    admission::{DenialOperation, DenialReason, DenialTelemetry},
    app, http_request_span,
    peer_identity::TrustedProxyIps,
};

const ORIGIN: &str = "http://localhost:5177";

#[derive(Clone)]
struct CapturedFields(Arc<Mutex<Vec<String>>>);

impl<Registry: Subscriber> Layer<Registry> for CapturedFields {
    fn on_new_span(
        &self,
        attributes: &Attributes<'_>,
        _id: &tracing::span::Id,
        _context: tracing_subscriber::layer::Context<'_, Registry>,
    ) {
        attributes.record(&mut FieldVisitor(Arc::clone(&self.0)));
    }

    fn on_event(
        &self,
        event: &Event<'_>,
        _context: tracing_subscriber::layer::Context<'_, Registry>,
    ) {
        event.record(&mut FieldVisitor(Arc::clone(&self.0)));
    }
}

struct FieldVisitor(Arc<Mutex<Vec<String>>>);

impl Visit for FieldVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0
            .lock()
            .unwrap()
            .push(format!("{}={value:?}", field.name()));
    }
}

#[test]
fn denial_summary_is_visible_under_documented_deployment_filter() {
    let fields = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            "grenadiers_server=info,tower_http=info",
        ))
        .with(CapturedFields(Arc::clone(&fields)));
    let started = Instant::now();
    let telemetry = DenialTelemetry::new(started);
    tracing::subscriber::with_default(subscriber, || {
        telemetry.record(DenialOperation::Mint, DenialReason::PeerRate, started);
        telemetry.record(
            DenialOperation::Mint,
            DenialReason::PeerRate,
            started + Duration::from_secs(61),
        );
    });

    let captured = fields.lock().unwrap().join(" ");
    assert!(captured.contains("admission denials"));
    assert!(captured.contains("Mint"));
    assert!(captured.contains("PeerRate"));
}

#[test]
fn trace_spans_exclude_query_credentials() {
    const CREDENTIAL: &str = "private-query-credential";
    let fields = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::registry().with(CapturedFields(Arc::clone(&fields)));
    let request = Request::post(format!("/api/v1/game/session?token={CREDENTIAL}"))
        .body(Body::empty())
        .unwrap();
    tracing::subscriber::with_default(subscriber, || drop(http_request_span(&request)));

    let captured = fields.lock().unwrap().join(" ");
    assert!(captured.contains("/api/v1/game/session"));
    assert!(!captured.contains(CREDENTIAL));
    assert!(!captured.contains("token="));
}

#[tokio::test(flavor = "current_thread")]
async fn denial_events_do_not_capture_invalid_proxy_header_values() {
    const SENTINEL: &str = "sentinel-private-forwarding-value";
    let fields = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::registry().with(CapturedFields(Arc::clone(&fields)));
    let _guard = tracing::subscriber::set_default(subscriber);
    let started = Instant::now() - Duration::from_secs(61);
    let telemetry = Arc::new(DenialTelemetry::new(started));
    telemetry.record(
        DenialOperation::Mint,
        DenialReason::InvalidProxyIdentity,
        started,
    );
    let mut state = AppState::without_database([ORIGIN.to_owned()]);
    state.trusted_proxies =
        Arc::new(TrustedProxyIps::parse(Some("192.0.2.1")).expect("trusted test transport"));
    state.denial_telemetry = telemetry;
    let request = Request::post("/api/v1/game/session")
        .header(header::ORIGIN, ORIGIN)
        .header("x-forwarded-for", SENTINEL)
        .body(Body::empty())
        .unwrap();
    let router = app(state, None).layer(MockConnectInfo(std::net::SocketAddr::from((
        [192, 0, 2, 1],
        1234,
    ))));
    assert_eq!(
        router.oneshot(request).await.unwrap().status(),
        StatusCode::BAD_REQUEST
    );

    let captured = fields.lock().unwrap().join(" ");
    assert!(captured.contains("admission denials"));
    assert!(captured.contains("Mint"));
    assert!(captured.contains("InvalidProxyIdentity"));
    assert!(!captured.contains(SENTINEL));
}

#[tokio::test(flavor = "current_thread")]
async fn authentication_denial_events_do_not_capture_frame_credentials() {
    const SENTINEL_TOKEN: &str = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
    let fields = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::registry().with(CapturedFields(Arc::clone(&fields)));
    let _guard = tracing::subscriber::set_default(subscriber);
    let started = Instant::now() - Duration::from_secs(61);
    let telemetry = Arc::new(DenialTelemetry::new(started));
    telemetry.record(
        DenialOperation::GameplayUpgrade,
        DenialReason::AuthenticationFailed,
        started,
    );
    let mut state = AppState::without_database([ORIGIN.to_owned()]);
    state.denial_telemetry = telemetry;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            app(state, None).into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let mut request = format!("ws://{address}/api/v1/game/ws")
        .into_client_request()
        .unwrap();
    request
        .headers_mut()
        .insert(header::ORIGIN, ORIGIN.parse().unwrap());
    let (mut socket, _) = connect_async(request).await.unwrap();
    socket
        .send(Message::Text(
            json!({
                "type":"authenticate",
                "protocolVersion":12,
                "token":SENTINEL_TOKEN
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();
    let _ = socket.next().await;
    server.abort();

    let captured = fields.lock().unwrap().join(" ");
    assert!(captured.contains("GameplayUpgrade"));
    assert!(captured.contains("AuthenticationFailed"));
    assert!(!captured.contains(SENTINEL_TOKEN));
}
