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
        axum::serve(listener, app(state, None)).await.unwrap();
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
            "type":"authenticate", "protocolVersion":1, "token":token
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
    assert_eq!(created["room"]["revision"], 1);
    assert_eq!(created["room"]["seats"][0]["occupied"], true);
    let code = created["room"]["roomCode"].clone();
    let room_id = created["room"]["roomId"].clone();

    send(
        &mut two,
        json!({"type":"joinRoom", "sequence":1, "roomCode":code}),
    )
    .await;
    let joined = next(&mut two).await;
    assert_eq!(joined["youSeat"], "two");
    assert_eq!(joined["room"]["revision"], 2);
    assert_eq!(joined["room"]["seats"][0]["occupied"], true);
    assert_eq!(joined["room"]["seats"][1]["occupied"], true);
    let update = next(&mut one).await;
    assert_eq!(update["youSeat"], "one");
    assert_eq!(update["room"], joined["room"]);

    send(
        &mut third,
        json!({"type":"joinRoom", "sequence":1, "roomCode":code}),
    )
    .await;
    assert_eq!(next(&mut third).await["code"], "roomFull");

    send(
        &mut one,
        json!({
            "type":"ready", "sequence":2, "roomId":room_id.clone(),
            "roundId":1, "ready":true
        }),
    )
    .await;
    assert_eq!(next(&mut one).await["type"], "roomState");
    assert_eq!(next(&mut two).await["type"], "roomState");
    send(
        &mut one,
        json!({
            "type":"movement", "sequence":3, "roomId":room_id.clone(),
            "roundId":1, "direction":null
        }),
    )
    .await;
    assert_eq!(next(&mut one).await["code"], "invalidPhase");
    send(
        &mut one,
        json!({
            "type":"placeBomb", "sequence":4, "roomId":room_id,
            "roundId":1, "actionId":1
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
            "roundId":1, "ready":true
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
    assert_eq!(next(&mut owner).await["room"]["revision"], 2);

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
    assert_eq!(next(&mut unbound).await["room"]["revision"], 3);

    send(
        &mut owner,
        json!({"type":"joinRoom", "sequence":4, "roomCode":code}),
    )
    .await;
    assert_eq!(next(&mut owner).await["youSeat"], "one");
    assert_eq!(next(&mut unbound).await["room"]["revision"], 4);
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
    assert_eq!(update["room"]["revision"], 3);
    server.abort();
}
