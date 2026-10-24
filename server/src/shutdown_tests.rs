use std::{future::Future, time::Duration};

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::{net::TcpListener, task::JoinHandle, time::timeout};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{Error as WebSocketError, Message, client::IntoClientRequest},
};
use tower::ServiceExt;

use super::*;
use crate::game::{Admission, BindingKey, ClosingReason, ConnectionId, RoomAction, RoomNotice};

const ORIGIN: &str = "http://localhost:5177";
type ClientSocket = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

async fn within<T>(future: impl Future<Output = T>) -> T {
    timeout(Duration::from_secs(2), future)
        .await
        .expect("test timed out")
}

fn token(state: &AppState) -> String {
    serde_json::to_value(
        state
            .game_sessions
            .issue(std::time::Instant::now())
            .unwrap(),
    )
    .unwrap()["token"]
        .as_str()
        .unwrap()
        .to_owned()
}

async fn server(state: AppState) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        axum::serve(listener, app(state, None)).await.unwrap();
    });
    (format!("ws://{address}"), task)
}

async fn connect(base: &str, path: &str) -> ClientSocket {
    let mut request = format!("{base}{path}").into_client_request().unwrap();
    request
        .headers_mut()
        .insert(axum::http::header::ORIGIN, ORIGIN.parse().unwrap());
    within(connect_async(request)).await.unwrap().0
}

async fn send(socket: &mut ClientSocket, value: Value) {
    within(socket.send(Message::Text(value.to_string().into())))
        .await
        .unwrap();
}

async fn next(socket: &mut ClientSocket) -> Message {
    loop {
        let message = within(socket.next()).await.unwrap().unwrap();
        match message {
            Message::Ping(data) => socket.send(Message::Pong(data)).await.unwrap(),
            message => return message,
        }
    }
}

async fn authenticate(socket: &mut ClientSocket, token: &str) {
    send(
        socket,
        json!({"type":"authenticate", "protocolVersion":11, "token":token}),
    )
    .await;
    let Message::Text(text) = next(socket).await else {
        panic!("expected authentication response");
    };
    assert_eq!(
        serde_json::from_str::<Value>(&text).unwrap()["type"],
        "authenticated"
    );
}

fn binding(connection: u64) -> BindingKey {
    BindingKey {
        connection_id: ConnectionId(connection),
        generation: 1,
    }
}

async fn running_room(state: &AppState) -> (Admission, Admission) {
    let now = std::time::Instant::now();
    let first = state.game_rooms.create(binding(1), now).await.unwrap();
    let second = state
        .game_rooms
        .join(&first.room.room_code, binding(2), now)
        .await
        .unwrap();
    for (binding, admission) in [(binding(1), &first), (binding(2), &second)] {
        state
            .game_rooms
            .action(
                admission,
                binding,
                admission.room_id.clone(),
                admission.round_id,
                RoomAction::Ready(true),
            )
            .await
            .unwrap();
    }
    for _ in 0..180 {
        tokio::time::advance(Duration::from_nanos(16_666_667)).await;
        tokio::task::yield_now().await;
    }
    (first, second)
}

async fn closing(admission: &mut Admission) -> RoomNotice {
    loop {
        let notice = admission.notices.borrow_and_update().clone();
        if matches!(notice, RoomNotice::Closing { .. }) {
            return notice;
        }
        let _ = admission.notices.changed().await;
    }
}

#[tokio::test]
async fn shutdown_delivers_member_finality_and_closes_every_socket_before_drain_completes() {
    let state = AppState::without_database([ORIGIN.to_owned()]);
    let tokens = [token(&state), token(&state)];
    let (base, task) = server(state.clone()).await;
    let mut member = connect(&base, "/api/v1/game/ws").await;
    authenticate(&mut member, &tokens[0]).await;
    send(&mut member, json!({"type":"createRoom", "sequence":1})).await;
    assert!(matches!(next(&mut member).await, Message::Text(_)));
    let mut unseated = connect(&base, "/api/v1/game/ws").await;
    authenticate(&mut unseated, &tokens[1]).await;
    let mut diagnostic = connect(&base, "/api/v1/ws").await;
    assert!(matches!(next(&mut diagnostic).await, Message::Text(_)));

    state.begin_shutdown();
    let Message::Text(final_text) = next(&mut member).await else {
        panic!("member must receive the final room state");
    };
    let final_state: Value = serde_json::from_str(&final_text).unwrap();
    assert_eq!(final_state["type"], "roomState");
    assert_eq!(final_state["room"]["phase"]["type"], "interrupted");
    assert_eq!(final_state["room"]["phase"]["reason"], "serverStopped");
    for socket in [&mut member, &mut unseated, &mut diagnostic] {
        let Message::Close(frame) = next(socket).await else {
            panic!("shutdown must close every socket");
        };
        assert_eq!(
            frame.unwrap().code,
            tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Away
        );
    }
    assert!(state.wait_for_shutdown().await);
    task.abort();
}

#[tokio::test(start_paused = true)]
async fn shutdown_drain_has_a_five_second_deadline_for_a_hung_connection_permit() {
    let state = AppState::without_database([ORIGIN.to_owned()]);
    let permit = state.connections.clone().acquire_owned().await.unwrap();
    state.begin_shutdown();
    let waiting = tokio::spawn({
        let state = state.clone();
        async move { state.wait_for_shutdown().await }
    });
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(5)).await;
    assert!(!waiting.await.unwrap());
    drop(permit);
}

#[tokio::test(start_paused = true)]
async fn app_shutdown_interrupts_a_running_room_with_its_real_frozen_snapshot() {
    let state = AppState::without_database([ORIGIN.to_owned()]);
    let (mut first, _second) = running_room(&state).await;
    state.begin_shutdown();
    let notice = closing(&mut first).await;
    let RoomNotice::Closing { last, reason } = notice else {
        unreachable!();
    };
    assert_eq!(reason, ClosingReason::ServerStopped);
    assert!(last.frame(grenadiers_core::game::SeatId::One).is_some());
    assert!(matches!(
        last.room.phase,
        grenadiers_core::protocol::v11::RoomPhase::MatchResult {
            result: grenadiers_core::protocol::v11::MatchResult::Interrupted {
                reason: grenadiers_core::protocol::v11::MatchInterruptReason::ServerStopped
            },
            ..
        }
    ));
    assert!(state.wait_for_shutdown().await);
}

#[tokio::test(start_paused = true)]
async fn app_shutdown_preserves_an_already_final_forfeit_outcome() {
    let state = AppState::without_database([ORIGIN.to_owned()]);
    let (mut first, second) = running_room(&state).await;
    let rooms = state.game_rooms.clone();
    let room_id = second.room_id.clone();
    let round_id = second.round_id;
    let leaving = tokio::spawn(async move {
        rooms
            .leave(&second, binding(2), room_id, round_id)
            .await
            .unwrap();
    });
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_nanos(16_666_667)).await;
    leaving.await.unwrap();
    state.begin_shutdown();

    let RoomNotice::Closing { last, .. } = closing(&mut first).await else {
        unreachable!();
    };
    assert!(matches!(
        last.room.phase,
        grenadiers_core::protocol::v11::RoomPhase::MatchResult {
            result: grenadiers_core::protocol::v11::MatchResult::Winner {
                reason: grenadiers_core::protocol::v11::MatchWinnerReason::Forfeit,
                ..
            },
            ..
        }
    ));
    assert!(state.wait_for_shutdown().await);
}

#[tokio::test]
async fn empty_shutdown_drain_completes_and_is_sticky() {
    let state = AppState::without_database([ORIGIN.to_owned()]);
    state.begin_shutdown();
    state.begin_shutdown();
    assert!(state.game_rooms.is_stopping());
    assert!(state.wait_for_shutdown().await);
}

#[tokio::test]
async fn stopping_rejects_readiness_sessions_and_new_websocket_upgrades() {
    let state = AppState::without_database([ORIGIN.to_owned()]);
    state.begin_shutdown();
    let router = app(state.clone(), None);
    let ready = router
        .clone()
        .oneshot(Request::get("/api/v1/ready").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(ready.status(), StatusCode::SERVICE_UNAVAILABLE);
    let session = router
        .oneshot(
            Request::post("/api/v1/game/session")
                .header(axum::http::header::ORIGIN, ORIGIN)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(session.status(), StatusCode::SERVICE_UNAVAILABLE);

    let (base, task) = server(state).await;
    for path in ["/api/v1/game/ws", "/api/v1/ws"] {
        let mut request = format!("{base}{path}").into_client_request().unwrap();
        request
            .headers_mut()
            .insert(axum::http::header::ORIGIN, ORIGIN.parse().unwrap());
        let Err(WebSocketError::Http(response)) = within(connect_async(request)).await else {
            panic!("stopping websocket upgrade must be rejected");
        };
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
    task.abort();
}
