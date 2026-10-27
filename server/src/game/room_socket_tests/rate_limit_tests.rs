use std::time::{Duration, Instant as StdInstant};

use grenadiers_core::protocol::{Counter, RoomId, v13::ResumeToken};
use tokio_tungstenite::tungstenite::Message;

use super::*;
use crate::game::{BindingKey, ConnectionId};

async fn deplete_resume_budget(state: &AppState) -> StdInstant {
    let depleted_at = StdInstant::now() + Duration::from_secs(60);
    let missing_room = RoomId::new("ffffffffffffffffffffffffffffffff").unwrap();
    let missing_token = ResumeToken::new("f".repeat(64)).unwrap();
    for connection in 10..42 {
        assert!(matches!(
            state
                .game_rooms
                .resume(
                    missing_room.clone(),
                    BindingKey {
                        connection_id: ConnectionId(connection),
                        generation: 1,
                    },
                    Counter::new(1).unwrap(),
                    missing_token.clone(),
                    depleted_at,
                )
                .await,
            Err(crate::game::rooms::RoomError::ResumeRejected)
        ));
    }
    depleted_at
}

#[tokio::test]
async fn throttled_resume_is_correlated_and_retryable_on_the_same_socket() {
    let state = AppState::without_database([ORIGIN.to_owned()]);
    let tokens = [issue_token(&state), issue_token(&state)];
    let (url, server) = start_server(state.clone()).await;
    let mut original = connect(&url, &tokens[0]).await;
    send(
        &mut original,
        json!({"type":"createRoom", "sequence":1, "mode":{"type":"humanDuel"}}),
    )
    .await;
    let created = next(&mut original).await;
    let room_id = created["room"]["roomId"].clone();
    let resume_token = created["resumeToken"].clone();
    let depleted_at = deplete_resume_budget(&state).await;

    let mut replacement = connect(&url, &tokens[1]).await;
    let request = json!({
        "type":"resumeRoom", "sequence":1,
        "roomId":room_id, "resumeToken":resume_token
    });
    send(&mut replacement, request.clone()).await;
    assert_eq!(
        next(&mut replacement).await,
        json!({"type":"error", "code":"rateLimited", "sequence":1})
    );
    assert!(
        tokio::time::timeout(Duration::from_millis(20), original.next())
            .await
            .is_err()
    );

    send(&mut replacement, request.clone()).await;
    assert_eq!(
        next(&mut replacement).await,
        json!({"type":"error", "code":"staleSequence", "sequence":1})
    );

    assert!(matches!(
        state
            .game_rooms
            .resume(
                RoomId::new("ffffffffffffffffffffffffffffffff").unwrap(),
                BindingKey {
                    connection_id: ConnectionId(50),
                    generation: 1,
                },
                Counter::new(1).unwrap(),
                ResumeToken::new("f".repeat(64)).unwrap(),
                depleted_at + Duration::from_millis(250),
            )
            .await,
        Err(crate::game::rooms::RoomError::ResumeRejected)
    ));
    let mut retry = request;
    retry["sequence"] = json!(2);
    send(&mut replacement, retry).await;
    let admitted = next(&mut replacement).await;
    assert_eq!(admitted["type"], "admitted");
    assert_eq!(admitted["youSeat"], created["youSeat"]);
    assert_eq!(admitted["room"]["roomId"], created["room"]["roomId"]);
    assert_eq!(admitted["resumeToken"], created["resumeToken"]);

    assert_eq!(
        next(&mut original).await,
        json!({"type":"seatReplaced", "roomId":room_id})
    );
    let close = within(original.next()).await.unwrap().unwrap();
    assert!(matches!(close, Message::Close(Some(frame)) if u16::from(frame.code) == 4001));
    server.abort();
}

#[tokio::test]
async fn throttled_unseated_socket_still_closes_and_drains_on_shutdown() {
    let state = AppState::without_database([ORIGIN.to_owned()]);
    let token = issue_token(&state);
    let _ = deplete_resume_budget(&state).await;
    let (url, server) = start_server(state.clone()).await;
    let mut socket = connect(&url, &token).await;

    send(
        &mut socket,
        json!({
            "type":"resumeRoom", "sequence":1,
            "roomId":"ffffffffffffffffffffffffffffffff",
            "resumeToken":"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
        }),
    )
    .await;
    assert_eq!(
        next(&mut socket).await,
        json!({"type":"error", "code":"rateLimited", "sequence":1})
    );

    state.begin_shutdown();
    let close = within(socket.next()).await.unwrap().unwrap();
    assert!(matches!(close, Message::Close(Some(frame)) if u16::from(frame.code) == 1001));
    assert!(state.wait_for_shutdown().await);
    server.abort();
}

#[tokio::test]
async fn peer_room_throttle_is_correlated_and_keeps_the_socket_open() {
    let state = AppState::without_database([ORIGIN.to_owned()]);
    let token = issue_token(&state);
    let (url, server) = start_server(state).await;
    let mut socket = connect(&url, &token).await;

    for sequence in 1..=8 {
        send(
            &mut socket,
            json!({
                "type":"resumeRoom", "sequence":sequence,
                "roomId":"ffffffffffffffffffffffffffffffff",
                "resumeToken":"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
            }),
        )
        .await;
        assert_eq!(next(&mut socket).await["code"], "resumeRejected");
    }
    send(
        &mut socket,
        json!({
            "type":"resumeRoom", "sequence":9,
            "roomId":"ffffffffffffffffffffffffffffffff",
            "resumeToken":"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
        }),
    )
    .await;
    assert_eq!(
        next(&mut socket).await,
        json!({"type":"error", "code":"rateLimited", "sequence":9})
    );
    send(
        &mut socket,
        json!({"type":"createRoom", "sequence":10, "mode":{"type":"humanDuel"}}),
    )
    .await;
    assert_eq!(next(&mut socket).await["type"], "admitted");
    server.abort();
}

#[tokio::test]
async fn inbound_flood_keeps_policy_close_and_releases_connection_permit() {
    const FLOOD_FRAMES: usize = 151;

    let state = AppState::without_database([ORIGIN.to_owned()]);
    let token = issue_token(&state);
    let (url, server) = start_server(state.clone()).await;
    let mut socket = connect(&url, &token).await;

    let (rate_limited, close) = within(async {
        // The two-second observation window can grant at most the 30-token burst
        // plus 120 refills. Buffer one more frame and flush the flood as one write
        // so client scheduling cannot turn it into an allowed 60-frame-per-second stream.
        for _ in 0..FLOOD_FRAMES {
            socket.feed(Message::Ping(Vec::new().into())).await.unwrap();
        }
        socket.flush().await.unwrap();

        let mut rate_limited = false;
        loop {
            match socket.next().await.unwrap().unwrap() {
                Message::Pong(_) => {}
                Message::Text(text) => {
                    let message: Value = serde_json::from_str(&text).unwrap();
                    if message["type"] == "error" && message["code"] == "rateLimited" {
                        rate_limited = true;
                    }
                }
                Message::Close(frame) => break (rate_limited, frame),
                other => panic!("unexpected flood response: {other:?}"),
            }
        }
    })
    .await;
    assert!(rate_limited);
    assert!(matches!(close, Some(frame) if u16::from(frame.code) == 1008));
    drop(socket);
    within(async {
        while state.connections.available_permits() != crate::MAX_CONNECTIONS {
            tokio::task::yield_now().await;
        }
    })
    .await;
    server.abort();
}
