use std::{future::Future, time::Duration};

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::{net::TcpListener, task::JoinHandle, time::timeout};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{Message, client::IntoClientRequest},
};

use crate::{AppState, app};

const ORIGIN: &str = "http://localhost:5177";
const TEST_TIMEOUT: Duration = Duration::from_secs(2);
type ClientSocket = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

async fn within<T>(future: impl Future<Output = T>) -> T {
    timeout(TEST_TIMEOUT, future).await.expect("test timed out")
}

fn issue_token(state: &AppState) -> String {
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

async fn start_server(state: AppState) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        axum::serve(
            listener,
            app(state, None).into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    (format!("ws://{address}/api/v1/game/ws"), task)
}

async fn connect(url: &str, token: &str) -> ClientSocket {
    let mut request = url.into_client_request().unwrap();
    request
        .headers_mut()
        .insert(axum::http::header::ORIGIN, ORIGIN.parse().unwrap());
    let (mut socket, _) = within(connect_async(request)).await.unwrap();
    send(
        &mut socket,
        json!({
            "type":"authenticate", "protocolVersion":11, "token":token
        }),
    )
    .await;
    assert_eq!(next(&mut socket).await["type"], "authenticated");
    socket
}

async fn send(socket: &mut ClientSocket, value: Value) {
    within(socket.send(Message::Text(value.to_string().into())))
        .await
        .unwrap();
}

async fn next(socket: &mut ClientSocket) -> Value {
    loop {
        let message = within(socket.next()).await.unwrap().unwrap();
        match message {
            Message::Text(text) => return serde_json::from_str(&text).unwrap(),
            Message::Ping(data) => socket.send(Message::Pong(data)).await.unwrap(),
            other => panic!("expected JSON text, got {other:?}"),
        }
    }
}

#[tokio::test]
async fn two_members_receive_personalized_admission_and_live_waiting_updates() {
    let state = AppState::without_database([ORIGIN.to_owned()]);
    let tokens = [
        issue_token(&state),
        issue_token(&state),
        issue_token(&state),
    ];
    let (url, server) = start_server(state).await;
    let mut one = connect(&url, &tokens[0]).await;
    let mut two = connect(&url, &tokens[1]).await;
    let mut third = connect(&url, &tokens[2]).await;

    send(&mut one, json!({"type":"createRoom", "sequence":1})).await;
    let created = next(&mut one).await;
    assert_eq!(created["youSeat"], "one");
    assert_eq!(created["room"]["revision"], 2);
    assert_eq!(created["room"]["roundNumber"], 0);
    assert_eq!(created["room"]["scores"], json!({"one":0,"two":0}));
    assert!(created["room"]["lastRound"].is_null());
    assert_eq!(created["room"]["phase"]["type"], "waiting");
    assert_eq!(created["room"]["seats"][0]["occupied"], true);
    assert_eq!(created["room"]["rulesVersion"], 5);
    assert_eq!(created["room"]["settingsRevision"], 1);
    assert_eq!(created["room"]["settingsEditor"], "one");
    assert_eq!(
        created["room"]["settings"],
        json!({
            "startingSpeedLevel":0, "turnAssistUnits":256,
            "winsToFinish":3, "bombFuseTicks":180, "enabledItems":["heart"]
        })
    );
    assert_eq!(
        created["room"]["availableItems"],
        json!([
            "kick",
            "punch",
            "bracelet",
            "wallPass",
            "heart",
            "remoteBomb",
            "piercingBomb",
            "powerBomb",
            "landmine",
            "shield",
            "stopwatch",
            "invisibilityCloak",
            "goldenFlame",
            "shortFuse",
            "diarrhea",
            "slowFoot",
            "shortFlame",
            "longFuse",
            "reverseControls",
            "invisibleBombs",
            "oilBarrel",
            "teleporter"
        ])
    );
    let code = created["room"]["roomCode"].clone();
    let room_id = created["room"]["roomId"].clone();

    send(
        &mut two,
        json!({"type":"joinRoom", "sequence":1, "roomCode":code}),
    )
    .await;
    let joined = next(&mut two).await;
    assert_eq!(joined["youSeat"], "two");
    assert_eq!(joined["room"]["revision"], 3);
    assert_eq!(joined["room"]["seats"][0]["occupied"], true);
    assert_eq!(joined["room"]["seats"][1]["occupied"], true);
    let update = next(&mut one).await;
    assert!(update.get("youSeat").is_none());
    assert_eq!(update["room"], joined["room"]);

    let settings = json!({
        "startingSpeedLevel":2, "turnAssistUnits":64,
        "winsToFinish":2, "bombFuseTicks":90, "enabledItems":[]
    });
    send(
        &mut two,
        json!({
            "type":"updateSettings", "sequence":2, "roomId":room_id.clone(),
            "roundId":1, "settingsRevision":1, "settings":settings.clone()
        }),
    )
    .await;
    assert_eq!(next(&mut two).await["code"], "settingsLocked");
    send(
        &mut two,
        json!({
            "type":"ready", "sequence":3, "roomId":room_id.clone(),
            "roundId":1, "ready":true, "settingsRevision":1
        }),
    )
    .await;
    assert_eq!(next(&mut two).await["type"], "roomState");
    next(&mut one).await;
    send(
        &mut one,
        json!({
            "type":"updateSettings", "sequence":2, "roomId":room_id.clone(),
            "roundId":1, "settingsRevision":1, "settings":settings
        }),
    )
    .await;
    let configured = next(&mut one).await;
    assert_eq!(configured["room"]["settingsRevision"], 2);
    assert_eq!(configured["room"]["seats"][1]["ready"], false);
    assert_eq!(next(&mut two).await["room"], configured["room"]);

    send(
        &mut third,
        json!({"type":"joinRoom", "sequence":1, "roomCode":code}),
    )
    .await;
    assert_eq!(next(&mut third).await["code"], "roomFull");

    send(
        &mut one,
        json!({
            "type":"ready", "sequence":3, "roomId":room_id.clone(),
            "roundId":1, "ready":true, "settingsRevision":2
        }),
    )
    .await;
    assert_eq!(next(&mut one).await["type"], "roomState");
    assert_eq!(next(&mut two).await["type"], "roomState");
    send(
        &mut one,
        json!({
            "type":"movement", "sequence":4, "roomId":room_id.clone(),
            "roundId":1, "direction":null
        }),
    )
    .await;
    assert_eq!(next(&mut one).await["code"], "invalidPhase");
    send(
        &mut one,
        json!({
            "type":"manualAction", "sequence":5, "roomId":room_id.clone(),
            "roundId":1, "actionId":1, "action":"equipment"
        }),
    )
    .await;
    assert_eq!(next(&mut one).await["code"], "invalidPhase");
    send(
        &mut one,
        json!({
            "type":"manualAction", "sequence":6, "roomId":room_id,
            "roundId":1, "actionId":1, "action":"bomb"
        }),
    )
    .await;
    assert_eq!(next(&mut one).await["code"], "invalidPhase");
    send(
        &mut one,
        json!({
            "type":"rematch", "sequence":7, "roomId":room_id,
            "roundId":1, "ready":true
        }),
    )
    .await;
    assert_eq!(next(&mut one).await["code"], "invalidPhase");
    server.abort();
}

#[tokio::test]
async fn binding_checks_precede_placeholder_phase_and_leave_allows_rejoin() {
    let state = AppState::without_database([ORIGIN.to_owned()]);
    let tokens = [issue_token(&state), issue_token(&state)];
    let (url, server) = start_server(state).await;
    let mut owner = connect(&url, &tokens[0]).await;
    let mut unbound = connect(&url, &tokens[1]).await;

    send(&mut owner, json!({"type":"createRoom", "sequence":1})).await;
    let created = next(&mut owner).await;
    let room_id = created["room"]["roomId"].clone();
    let code = created["room"]["roomCode"].clone();
    send(
        &mut unbound,
        json!({
            "type":"ready", "sequence":1, "roomId":room_id,
            "roundId":1, "ready":true, "settingsRevision":1
        }),
    )
    .await;
    assert_eq!(next(&mut unbound).await["code"], "notInRoom");

    send(
        &mut unbound,
        json!({"type":"joinRoom", "sequence":2, "roomCode":code}),
    )
    .await;
    assert_eq!(next(&mut unbound).await["youSeat"], "two");
    assert_eq!(next(&mut owner).await["room"]["revision"], 3);

    send(
        &mut owner,
        json!({
            "type":"movement", "sequence":2, "roomId":room_id,
            "roundId":2, "direction":"east"
        }),
    )
    .await;
    assert_eq!(next(&mut owner).await["code"], "staleRoom");
    send(
        &mut owner,
        json!({
            "type":"leaveRoom", "sequence":3, "roomId":room_id, "roundId":1
        }),
    )
    .await;
    assert_eq!(next(&mut owner).await["type"], "leftRoom");
    assert_eq!(next(&mut unbound).await["room"]["revision"], 4);

    send(
        &mut owner,
        json!({"type":"joinRoom", "sequence":4, "roomCode":code}),
    )
    .await;
    assert_eq!(next(&mut owner).await["youSeat"], "one");
    assert_eq!(next(&mut unbound).await["room"]["revision"], 5);
    send(&mut owner, json!({"type":"createRoom", "sequence":5})).await;
    assert_eq!(next(&mut owner).await["code"], "alreadyInRoom");
    server.abort();
}

#[tokio::test]
async fn disconnect_removes_seat_even_when_no_more_commands_arrive() {
    let state = AppState::without_database([ORIGIN.to_owned()]);
    let tokens = [issue_token(&state), issue_token(&state)];
    let (url, server) = start_server(state).await;
    let mut one = connect(&url, &tokens[0]).await;
    let mut two = connect(&url, &tokens[1]).await;
    send(&mut one, json!({"type":"createRoom", "sequence":1})).await;
    let created = next(&mut one).await;
    send(
        &mut two,
        json!({
            "type":"joinRoom", "sequence":1, "roomCode":created["room"]["roomCode"]
        }),
    )
    .await;
    next(&mut two).await;
    next(&mut one).await;

    two.close(None).await.unwrap();
    let update = next(&mut one).await;
    assert_eq!(update["room"]["seats"][1]["occupied"], false);
    assert_eq!(update["room"]["revision"], 4);
    server.abort();
}

#[tokio::test]
async fn connected_resume_replaces_old_socket_with_private_admission_and_close_4001() {
    let state = AppState::without_database([ORIGIN.to_owned()]);
    let tokens = [issue_token(&state), issue_token(&state)];
    let (url, server) = start_server(state).await;
    let mut original = connect(&url, &tokens[0]).await;
    send(&mut original, json!({"type":"createRoom", "sequence":1})).await;
    let created = next(&mut original).await;
    let room_id = created["room"]["roomId"].clone();
    let resume_token = created["resumeToken"].clone();

    let mut replacement = connect(&url, &tokens[1]).await;
    send(
        &mut replacement,
        json!({
            "type":"resumeRoom", "sequence":1,
            "roomId":room_id, "resumeToken":resume_token
        }),
    )
    .await;
    let admitted = next(&mut replacement).await;
    assert_eq!(admitted["type"], "admitted");
    assert_eq!(admitted["youSeat"], "one");
    assert_eq!(admitted["resumeToken"], created["resumeToken"]);

    let replaced = next(&mut original).await;
    assert_eq!(replaced, json!({"type":"seatReplaced", "roomId":room_id}));
    let close = within(original.next()).await.unwrap().unwrap();
    assert!(matches!(close, Message::Close(Some(frame)) if u16::from(frame.code) == 4001));
    server.abort();
}

#[tokio::test]
async fn unavailable_resume_is_uniformly_rejected_on_the_wire() {
    let state = AppState::without_database([ORIGIN.to_owned()]);
    let token = issue_token(&state);
    let (url, server) = start_server(state).await;
    let mut socket = connect(&url, &token).await;
    send(
        &mut socket,
        json!({
            "type":"resumeRoom", "sequence":1,
            "roomId":"00000000000000000000000000000001",
            "resumeToken":"0000000000000000000000000000000000000000000000000000000000000000"
        }),
    )
    .await;

    let rejected = next(&mut socket).await;
    assert_eq!(rejected["type"], "error");
    assert_eq!(rejected["code"], "resumeRejected");
    assert_eq!(rejected["sequence"], 1);
    server.abort();
}

#[cfg(test)]
#[path = "room_socket_tests/rate_limit_tests.rs"]
mod rate_limit_tests;
