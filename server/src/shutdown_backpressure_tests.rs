use std::{future::Future, sync::Arc, time::Duration};

use axum::serve::ListenerExt;
use futures_util::{FutureExt, SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::{
    net::TcpStream,
    sync::oneshot,
    task::JoinHandle,
    time::{Instant, sleep, timeout},
};
use tokio_tungstenite::{
    WebSocketStream, client_async,
    tungstenite::{Message, client::IntoClientRequest},
};

use super::{AppState, MAX_CONNECTIONS, ShutdownOutcome, app, run_with_shutdown};
use crate::shutdown_backpressure_test_support::{
    ObservedListener, REQUESTED_BUFFER_BYTES, WriteObservations, constrained_client,
};

const ORIGIN: &str = "http://localhost:5177";
const WATCHDOG: Duration = Duration::from_secs(30);
type Client = WebSocketStream<TcpStream>;

async fn within<T>(future: impl Future<Output = T>) -> T {
    timeout(WATCHDOG, future)
        .await
        .expect("backpressure fixture watchdog elapsed")
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

async fn connect(address: std::net::SocketAddr) -> (Client, u32) {
    let (stream, receive_buffer) = constrained_client(address).await.unwrap();
    let mut request = format!("ws://{address}/api/v1/game/ws")
        .into_client_request()
        .unwrap();
    request
        .headers_mut()
        .insert(axum::http::header::ORIGIN, ORIGIN.parse().unwrap());
    let (socket, response) = within(client_async(request, stream)).await.unwrap();
    assert_eq!(
        response.status(),
        axum::http::StatusCode::SWITCHING_PROTOCOLS
    );
    (socket, receive_buffer)
}

async fn send(socket: &mut Client, value: Value) {
    within(socket.send(Message::Text(value.to_string().into())))
        .await
        .unwrap();
}

async fn next_json(socket: &mut Client) -> Value {
    loop {
        match within(socket.next()).await.unwrap().unwrap() {
            Message::Text(text) => return serde_json::from_str(&text).unwrap(),
            Message::Ping(data) => socket.send(Message::Pong(data)).await.unwrap(),
            other => panic!("expected JSON text, got {other:?}"),
        }
    }
}

async fn authenticate(socket: &mut Client, token: &str) {
    send(
        socket,
        json!({"type":"authenticate", "protocolVersion":18, "token":token}),
    )
    .await;
    assert_eq!(next_json(socket).await["type"], "authenticated");
}

struct Server {
    address: std::net::SocketAddr,
    state: AppState,
    signal: oneshot::Sender<()>,
    task: JoinHandle<(Instant, std::io::Result<ShutdownOutcome>)>,
    observations: Arc<[Arc<WriteObservations>]>,
    effective_listener_send_buffer: u32,
}

async fn server(connection_count: usize) -> Server {
    let (listener, address, effective_listener_send_buffer, observations) =
        ObservedListener::constrained(connection_count).unwrap();
    let state = AppState::without_database([ORIGIN.to_owned()]);
    let (signal, signal_rx) = oneshot::channel();
    let task = tokio::spawn({
        let task_state = state.clone();
        async move {
            let outcome = run_with_shutdown(
                task_state.clone(),
                async move {
                    signal_rx.await.unwrap();
                },
                move |graceful| {
                    axum::serve(
                        listener.tap_io(|_| {}),
                        app(task_state, None)
                            .into_make_service_with_connect_info::<std::net::SocketAddr>(),
                    )
                    .with_graceful_shutdown(async move {
                        let _ = graceful.await;
                    })
                },
            )
            .await;
            (Instant::now(), outcome)
        }
    });
    Server {
        address,
        state,
        signal,
        task,
        observations,
        effective_listener_send_buffer,
    }
}

struct RunningFixture {
    server: Server,
    blocked: Client,
    healthy: Client,
    blocked_receive_buffer: u32,
    initial_tick: u64,
}

async fn running_fixture() -> RunningFixture {
    let server = server(2).await;
    let (mut blocked, blocked_receive_buffer) = connect(server.address).await;
    let (mut healthy, _) = connect(server.address).await;
    authenticate(&mut blocked, &token(&server.state)).await;
    authenticate(&mut healthy, &token(&server.state)).await;
    send(
        &mut blocked,
        json!({"type":"createRoom", "sequence":1, "configuration":{"controllers":["human","human"],"cpuDifficulty":null,"board":{"width":15,"height":13},"environment":{"stormCloud":true,"lairs":{"enabled":false,"spawnSeconds":10,"maxLivingMobs":1},"spikePits":{"enabled":true,"warningTicks":36,"raisedTicks":180}}}}),
    )
    .await;
    let created = next_json(&mut blocked).await;
    let room_id = created["room"]["roomId"].clone();
    let room_code = created["room"]["roomCode"].clone();
    send(
        &mut healthy,
        json!({"type":"joinRoom", "sequence":1, "roomCode":room_code}),
    )
    .await;
    assert_eq!(next_json(&mut healthy).await["type"], "admitted");
    assert_eq!(next_json(&mut blocked).await["type"], "roomState");
    for (socket, sequence) in [(&mut blocked, 2), (&mut healthy, 2)] {
        send(
            socket,
            json!({
                "type":"ready", "sequence":sequence, "roomId":room_id.clone(),
                "roundId":1, "ready":true, "settingsRevision":1
            }),
        )
        .await;
    }
    let initial_tick = running_snapshot(&mut blocked).await;
    let _ = running_snapshot(&mut healthy).await;
    server.observations[0].arm();
    server.observations[1].arm();
    RunningFixture {
        server,
        blocked,
        healthy,
        blocked_receive_buffer,
        initial_tick,
    }
}

async fn running_snapshot(socket: &mut Client) -> u64 {
    loop {
        let message = next_json(socket).await;
        if message["type"] == "snapshot" {
            return message["state"]["physicalTick"].as_u64().unwrap();
        }
    }
}

async fn resumed_snapshot_without_advancing_time(
    socket: &mut Client,
    observation: &WriteObservations,
    initial_tick: u64,
    calls_at_pressure: usize,
) -> u64 {
    let mut newest_tick = initial_tick;
    for _ in 0..200_000 {
        match socket.next().now_or_never() {
            Some(Some(Ok(Message::Text(text)))) => {
                let value: Value = serde_json::from_str(&text).unwrap();
                if value["type"] == "snapshot" {
                    newest_tick = newest_tick.max(value["state"]["physicalTick"].as_u64().unwrap());
                }
            }
            Some(Some(Ok(Message::Ping(_)))) => {
                panic!("unexpected server ping in recovery fixture")
            }
            Some(Some(Ok(other))) => panic!("unexpected recovery frame {other:?}"),
            Some(Some(Err(error))) => panic!("recovery socket failed: {error}"),
            Some(None) => panic!("recovery socket closed"),
            None => tokio::task::yield_now().await,
        }
        if newest_tick > initial_tick
            && WriteObservations::load(&observation.calls) > calls_at_pressure
            && !observation.is_pressured()
        {
            return newest_tick;
        }
    }
    panic!("socket did not make writability-driven progress while virtual time was held");
}

async fn wait_for_pressure(observation: &WriteObservations) {
    let reached = timeout(WATCHDOG, async {
        loop {
            let bytes = WriteObservations::load(&observation.bytes);
            if bytes > 0 && observation.is_pressured() {
                return;
            }
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await;
    if reached.is_err() {
        panic!(
            "real outbound pressure was not reached: bytes={}, calls={}, syscall_would_block={}, pending={}",
            WriteObservations::load(&observation.bytes),
            WriteObservations::load(&observation.calls),
            WriteObservations::load(&observation.syscall_would_block),
            WriteObservations::load(&observation.pending),
        );
    }
}

async fn healthy_reader(mut socket: Client) -> (usize, bool, bool) {
    let mut snapshots = 0;
    let mut finality = false;
    while let Some(message) = socket.next().await {
        match message {
            Ok(Message::Text(text)) => {
                let value: Value = serde_json::from_str(&text).unwrap();
                snapshots += usize::from(value["type"] == "snapshot");
                finality |= value["room"]["phase"]["type"] == "matchResult"
                    && value["room"]["phase"]["result"]["type"] == "interrupted"
                    && value["room"]["phase"]["result"]["reason"] == "serverStopped";
            }
            Ok(Message::Ping(data)) => socket.send(Message::Pong(data)).await.unwrap(),
            Ok(Message::Close(frame)) => {
                let away = frame.is_some_and(|value| u16::from(value.code) == 1001);
                return (snapshots, finality, away);
            }
            Err(_) => return (snapshots, finality, false),
            _ => {}
        }
    }
    (snapshots, finality, false)
}

async fn assert_cleanup(state: &AppState, observations: &[Arc<WriteObservations>]) {
    within(async {
        state.game_rooms.wait_empty().await;
        while state.connections.available_permits() != MAX_CONNECTIONS
            || observations
                .iter()
                .any(|value| WriteObservations::load(&value.dropped) == 0)
        {
            tokio::task::yield_now().await;
        }
    })
    .await;
}

#[tokio::test]
async fn idle_non_reader_does_not_manufacture_outbound_pressure() {
    let server = server(1).await;
    let (mut client, effective_receive_buffer) = connect(server.address).await;
    authenticate(&mut client, &token(&server.state)).await;
    server.observations[0].arm();
    sleep(Duration::from_millis(250)).await;
    assert_eq!(WriteObservations::load(&server.observations[0].bytes), 0);
    assert_eq!(
        WriteObservations::load(&server.observations[0].syscall_would_block),
        0
    );
    assert!(server.effective_listener_send_buffer >= REQUESTED_BUFFER_BYTES);
    assert!(effective_receive_buffer >= REQUESTED_BUFFER_BYTES);
    server.signal.send(()).unwrap();
    while !matches!(within(client.next()).await, Some(Ok(Message::Close(_)))) {}
    let (_, outcome) = within(server.task).await.unwrap();
    assert_eq!(outcome.unwrap(), ShutdownOutcome::Completed);
    drop(client);
    assert_cleanup(&server.state, &server.observations).await;
}

#[tokio::test(flavor = "current_thread")]
async fn a_real_blocked_websocket_recovers_when_the_client_resumes_reads() {
    let mut fixture = running_fixture().await;
    let healthy = tokio::spawn(healthy_reader(fixture.healthy));
    wait_for_pressure(&fixture.server.observations[0]).await;
    let calls_at_pressure = WriteObservations::load(&fixture.server.observations[0].calls);
    tokio::time::pause();
    let held = Instant::now();
    let resumed_tick = resumed_snapshot_without_advancing_time(
        &mut fixture.blocked,
        &fixture.server.observations[0],
        fixture.initial_tick,
        calls_at_pressure,
    )
    .await;
    assert!(resumed_tick > fixture.initial_tick);
    assert_eq!(Instant::now(), held);
    tokio::time::resume();
    assert!(fixture.server.effective_listener_send_buffer >= REQUESTED_BUFFER_BYTES);
    assert!(fixture.blocked_receive_buffer >= REQUESTED_BUFFER_BYTES);
    fixture.server.signal.send(()).unwrap();
    while !matches!(
        within(fixture.blocked.next()).await,
        Some(Ok(Message::Close(_)))
    ) {}
    let (_, outcome) = within(fixture.server.task).await.unwrap();
    assert_eq!(outcome.unwrap(), ShutdownOutcome::Completed);
    drop(fixture.blocked);
    let (snapshots, finality, closed) = within(healthy).await.unwrap();
    assert!(snapshots > 0 && finality && closed);
    assert_cleanup(&fixture.server.state, &fixture.server.observations).await;
}

#[tokio::test(flavor = "current_thread")]
async fn shutdown_during_a_real_pending_send_obeys_the_shared_deadline() {
    let fixture = running_fixture().await;
    let healthy = tokio::spawn(healthy_reader(fixture.healthy));
    wait_for_pressure(&fixture.server.observations[0]).await;
    assert!(WriteObservations::load(&fixture.server.observations[0].calls) > 0);
    assert!(WriteObservations::load(&fixture.server.observations[0].syscall_would_block) > 0);

    tokio::time::pause();
    let signal_instant = Instant::now();
    fixture.server.signal.send(()).unwrap();
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(5)).await;
    tokio::task::yield_now().await;
    for _ in 0..10_000 {
        if fixture.server.task.is_finished() {
            break;
        }
        tokio::task::yield_now().await;
    }
    assert!(fixture.server.task.is_finished());
    let (completed_at, outcome) = fixture.server.task.await.unwrap();
    let outcome = outcome.unwrap();
    assert!(completed_at <= signal_instant + Duration::from_secs(5));
    assert!(matches!(
        outcome,
        ShutdownOutcome::Completed | ShutdownOutcome::DeadlineElapsed
    ));
    tokio::time::resume();

    drop(fixture.blocked);
    let (snapshots, finality, closed) = within(healthy).await.unwrap();
    assert!(snapshots > 0 && finality && closed);
    assert_cleanup(&fixture.server.state, &fixture.server.observations).await;
}
