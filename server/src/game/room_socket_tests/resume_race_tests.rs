use tokio_tungstenite::tungstenite::Message;

use super::*;

async fn expect_replaced(socket: &mut ClientSocket, room_id: &Value) {
    for _ in 0..3 {
        let message = next(socket).await;
        if message["type"] == "seatReplaced" {
            assert_eq!(message["roomId"], *room_id);
            let close = within(socket.next()).await.unwrap().unwrap();
            assert!(matches!(close, Message::Close(Some(frame)) if u16::from(frame.code) == 4001));
            return;
        }
        assert_eq!(message["type"], "roomState");
    }
    panic!("replacement notice was not delivered");
}

async fn stop_server(state: &AppState, final_socket: &mut ClientSocket, server: JoinHandle<()>) {
    state.begin_shutdown();
    loop {
        let Some(message) = within(final_socket.next()).await else {
            break;
        };
        if matches!(message.unwrap(), Message::Close(_)) {
            break;
        }
    }
    assert!(state.wait_for_shutdown().await);
    server.abort();
    assert!(server.await.unwrap_err().is_cancelled());
}

#[tokio::test]
async fn concurrent_wire_resumes_commit_serially_and_the_last_admission_owns_the_seat() {
    let state = AppState::without_database([ORIGIN.to_owned()]);
    let tokens = [
        issue_token(&state),
        issue_token(&state),
        issue_token(&state),
    ];
    let (url, server) = start_server(state.clone()).await;
    let mut original = connect(&url, &tokens[0]).await;
    let mut replacement_a = connect(&url, &tokens[1]).await;
    let mut replacement_b = connect(&url, &tokens[2]).await;
    send(
        &mut original,
        json!({"type":"createRoom", "sequence":1, "configuration":{"controllers":["human","human"],"cpuDifficulty":null,"board":{"width":15,"height":13},"environment":{"stormCloud":true,"lairs":{"enabled":false,"spawnSeconds":10,"maxLivingMobs":1},"spikePits":{"enabled":true,"warningTicks":36,"raisedTicks":180}}}}),
    )
    .await;
    let created = next(&mut original).await;
    let room_id = created["room"]["roomId"].clone();
    let request = json!({
        "type":"resumeRoom", "sequence":1,
        "roomId":room_id, "resumeToken":created["resumeToken"]
    });

    send(&mut replacement_a, request.clone()).await;
    send(&mut replacement_b, request).await;
    let admitted_a = next(&mut replacement_a).await;
    let admitted_b = next(&mut replacement_b).await;
    assert_eq!(admitted_a["type"], "admitted");
    assert_eq!(admitted_b["type"], "admitted");
    assert_eq!(admitted_a["youSeat"], created["youSeat"]);
    assert_eq!(admitted_b["youSeat"], created["youSeat"]);
    let revision_a = admitted_a["room"]["revision"].as_u64().unwrap();
    let revision_b = admitted_b["room"]["revision"].as_u64().unwrap();
    assert_ne!(revision_a, revision_b);

    expect_replaced(&mut original, &room_id).await;
    let (earlier, final_socket, final_admission) = if revision_a < revision_b {
        (&mut replacement_a, &mut replacement_b, &admitted_b)
    } else {
        (&mut replacement_b, &mut replacement_a, &admitted_a)
    };
    expect_replaced(earlier, &room_id).await;

    send(
        final_socket,
        json!({
            "type":"ready", "sequence":2, "roomId":room_id,
            "roundId":final_admission["room"]["roundId"], "ready":true,
            "settingsRevision":final_admission["room"]["settingsRevision"]
        }),
    )
    .await;
    let ready = next(final_socket).await;
    assert_eq!(ready["type"], "roomState");
    assert_eq!(ready["room"]["seats"][0]["ready"], true);

    stop_server(&state, final_socket, server).await;
}
