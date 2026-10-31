use std::time::Instant;

use axum::{
    body::Body,
    extract::connect_info::MockConnectInfo,
    http::{Request, StatusCode, header},
};
use grenadiers_core::{
    game::StormCloudConfig,
    protocol::{Counter, v14::ResumeToken},
    room::{
        BoardSize, CpuDifficulty, LairRoomConfiguration, LairSettings, LairTuning,
        RoomConfiguration, SeatControl,
    },
};
use http_body_util::BodyExt;
use jsonschema::validator_for;
use serde_json::{Value, json};
use tower::ServiceExt;

use super::*;

const ORIGIN: &str = "http://localhost:5177";
const ROOM: &str = "0123456789abcdef0123456789abcdef";
const TOKEN: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn state(enabled: bool) -> AppState {
    let mut state = AppState::without_database([ORIGIN.to_owned()]);
    state.cpu_debug_enabled = enabled;
    state.cpu_debug_gate = cpu_debug::CpuDebugGate::new(Instant::now());
    state
}

fn request(body: &str) -> Request<Body> {
    Request::post("/api/v1/game/cpu-debug")
        .header(header::ORIGIN, ORIGIN)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
        .body(Body::from(body.to_owned()))
        .unwrap()
}

fn live_request(room_id: &str, token: &ResumeToken) -> Request<Body> {
    Request::post("/api/v1/game/cpu-debug")
        .header(header::ORIGIN, ORIGIN)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", token.as_str()))
        .body(Body::from(format!(r#"{{"roomId":"{room_id}"}}"#)))
        .unwrap()
}

async fn send(state: AppState, request: Request<Body>) -> Response {
    app(state, None)
        .layer(MockConnectInfo(std::net::SocketAddr::from((
            [192, 0, 2, 44],
            1234,
        ))))
        .oneshot(request)
        .await
        .unwrap()
}

async fn response_json(response: Response) -> Value {
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}

#[tokio::test]
async fn disabled_route_is_404_before_headers_credentials_or_body_are_processed() {
    let response = send(
        state(false),
        Request::post("/api/v1/game/cpu-debug")
            .body(Body::from(vec![b'x'; 4_096]))
            .unwrap(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
}

#[tokio::test]
async fn enabled_route_fail_closes_origin_authorization_and_body_shape() {
    let missing_origin = Request::post("/api/v1/game/cpu-debug")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(format!(r#"{{"roomId":"{ROOM}"}}"#)))
        .unwrap();
    assert_eq!(
        send(state(true), missing_origin).await.status(),
        StatusCode::FORBIDDEN
    );

    let missing_auth = Request::post("/api/v1/game/cpu-debug")
        .header(header::ORIGIN, ORIGIN)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(format!(r#"{{"roomId":"{ROOM}"}}"#)))
        .unwrap();
    assert_eq!(
        send(state(true), missing_auth).await.status(),
        StatusCode::UNAUTHORIZED
    );

    let unknown = format!(r#"{{"roomId":"{ROOM}","credential":"never-in-body"}}"#);
    assert_eq!(
        send(state(true), request(&unknown)).await.status(),
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn enabled_route_has_independent_fixed_size_and_rate_bounds() {
    let oversized = format!(r#"{{"roomId":"{ROOM}","padding":"{}"}}"#, "x".repeat(256));
    assert_eq!(
        send(state(true), request(&oversized)).await.status(),
        StatusCode::PAYLOAD_TOO_LARGE
    );

    let state = state(true);
    let valid = format!(r#"{{"roomId":"{ROOM}"}}"#);
    assert_eq!(
        send(state.clone(), request(&valid)).await.status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        send(state.clone(), request(&valid)).await.status(),
        StatusCode::NOT_FOUND
    );
    let limited = send(state, request(&valid)).await;
    assert_eq!(limited.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(limited.headers()[header::RETRY_AFTER], "1");
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn connected_forming_room_stays_200_then_publishes_available_cpu_intent() {
    let mut state = state(true);
    state.game_rooms = crate::game::GameRooms::for_cpu_debug_test(Instant::now());
    let binding = crate::game::BindingKey {
        connection_id: crate::game::ConnectionId(701),
        generation: 1,
    };
    let configuration = LairRoomConfiguration::new(
        RoomConfiguration::new(
            &[
                SeatControl::Human,
                SeatControl::Cpu,
                SeatControl::Cpu,
                SeatControl::Cpu,
            ],
            Some(CpuDifficulty::Standard),
            BoardSize::FifteenByThirteen,
        )
        .unwrap(),
        StormCloudConfig::Single,
        LairTuning::new(false, LairSettings::new(10, 1).unwrap()),
    );
    let now = Instant::now();
    let admission = state
        .game_rooms
        .create_live(binding, Counter::new(1).unwrap(), now, configuration)
        .await
        .unwrap();
    let personal: Value = serde_json::from_str(admission.personal_frame()).unwrap();
    let room_id = admission.room_id().clone();
    let token = ResumeToken::new(personal["resumeToken"].as_str().unwrap().to_owned()).unwrap();
    let round_id = personal["room"]["roundId"].as_u64().unwrap();

    let lobby = send(state.clone(), live_request(room_id.as_str(), &token)).await;
    assert_eq!(lobby.status(), StatusCode::OK);
    let lobby = response_json(lobby).await;
    assert_eq!(
        lobby["cpus"][0],
        json!({"seat": "two", "status": "unavailable"})
    );
    assert_eq!(lobby["cpus"].as_array().unwrap().len(), 3);

    state
        .game_rooms
        .action_live(
            &admission,
            binding,
            room_id.clone(),
            round_id,
            crate::game::RosterAction::Ready {
                ready: true,
                settings_revision: 1,
            },
        )
        .await
        .unwrap();

    let mut available = None;
    for _ in 0..(grenadiers_core::room::COUNTDOWN_TICKS + 240) {
        tokio::time::advance(std::time::Duration::from_millis(17)).await;
        tokio::task::yield_now().await;
        state.game_rooms.pump_cpu_debug_test_jobs();
        tokio::task::yield_now().await;
        let snapshot = state
            .game_rooms
            .cpu_debug(&room_id, token.clone())
            .await
            .unwrap();
        if snapshot
            .cpus
            .iter()
            .any(|row| matches!(row, cpu_debug::CpuDebugRow::Available { .. }))
        {
            available = Some(snapshot);
            break;
        }
    }
    assert!(available.is_some(), "CPU intent did not become available");

    let running = send(state, live_request(room_id.as_str(), &token)).await;
    assert_eq!(running.status(), StatusCode::OK);
    assert!(
        response_json(running).await["cpus"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["status"] == "available")
    );
}

#[test]
fn standalone_response_contract_is_closed_and_versioned() {
    let schema: Value =
        serde_json::from_str(include_str!("../../api/cpu-debug.schema.json")).unwrap();
    let validator = validator_for(&schema).unwrap();
    let openapi: Value = serde_json::from_str(include_str!("../../api/openapi.json")).unwrap();
    let mut openapi_schema = json!({
        "$ref": "#/components/schemas/CpuDebugResponse",
        "components": { "schemas": openapi["components"]["schemas"].clone() }
    });
    normalize_openapi_nullable(&mut openapi_schema);
    let openapi_validator = validator_for(&openapi_schema).unwrap();
    let response = json!({
        "protocolVersion": 1, "roomId": ROOM, "roundId": 1, "tick": 0, "revision": 1,
        "youSeat": "one", "cpus": [{"seat": "two", "status": "unavailable"}]
    });
    assert!(validator.is_valid(&response));
    assert!(openapi_validator.is_valid(&response));
    let mut unknown = response.clone();
    unknown
        .as_object_mut()
        .unwrap()
        .insert("privateMemory".into(), json!(true));
    assert!(!validator.is_valid(&unknown));
    assert!(!openapi_validator.is_valid(&unknown));
    let mut wrong_version = response;
    wrong_version["protocolVersion"] = json!(2);
    assert!(!validator.is_valid(&wrong_version));
    assert!(!openapi_validator.is_valid(&wrong_version));

    for objective in [
        cpu_debug::DebugObjective::Wait,
        cpu_debug::DebugObjective::Survive,
    ] {
        let available = serde_json::to_value(cpu_debug::CpuDebugResponse {
            protocol_version: 1,
            room_id: grenadiers_core::protocol::RoomId::new(ROOM).unwrap(),
            round_id: 1,
            tick: 4,
            revision: 2,
            you_seat: cpu_debug::DebugSeat::One,
            cpus: vec![cpu_debug::CpuDebugRow::Available {
                seat: cpu_debug::DebugSeat::Two,
                observed_tick: 4,
                objective,
                target: None,
                approach: None,
                phase: cpu_debug::DebugPhase::Waiting,
                switch_reason: cpu_debug::DebugSwitchReason::NoTargets,
            }],
        })
        .unwrap();
        assert!(
            validator.is_valid(&available),
            "standalone rejected serialized {objective:?}: {available}"
        );
        assert!(
            openapi_validator.is_valid(&available),
            "OpenAPI rejected serialized {objective:?}: {available}"
        );
    }
}

fn normalize_openapi_nullable(value: &mut Value) {
    match value {
        Value::Array(values) => values.iter_mut().for_each(normalize_openapi_nullable),
        Value::Object(object) => {
            let nullable = object.remove("nullable") == Some(Value::Bool(true));
            if nullable && let Some(Value::String(kind)) = object.get_mut("type") {
                let kind = std::mem::take(kind);
                object.insert("type".to_owned(), json!([kind, "null"]));
            }
            object.values_mut().for_each(normalize_openapi_nullable);
        }
        _ => {}
    }
}
