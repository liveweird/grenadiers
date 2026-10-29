use super::*;

#[tokio::test]
async fn binding_checks_precede_phase_and_leave_allows_fenced_replacement() {
    let state = AppState::without_database([ORIGIN.to_owned()]);
    let tokens = [issue_token(&state), issue_token(&state)];
    let (url, server) = start_server(state).await;
    let mut owner = connect(&url, &tokens[0]).await;
    let mut unbound = connect(&url, &tokens[1]).await;

    send(
        &mut owner,
        json!({"type":"createRoom", "sequence":1, "configuration":{"controllers":["human","human"],"cpuDifficulty":null,"board":{"width":15,"height":13},"environment":{"stormCloud":true}}}),
    )
    .await;
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
    let joined = next(&mut unbound).await;
    assert_eq!(joined["youSeat"], "two");
    let joined_revision = joined["room"]["revision"].as_u64().unwrap();
    waiting_reply::expect_waiting_update(&mut owner, &room_id, joined_revision, |room| {
        room["seats"][1]["occupied"] == true
    })
    .await;

    send(
        &mut owner,
        json!({
            "type":"movement", "sequence":2, "roomId":room_id,
            "roundId":2, "direction":"east"
        }),
    )
    .await;
    waiting_reply::expect_error(&mut owner, &room_id, "staleRoom", 2, joined_revision).await;
    send(
        &mut owner,
        json!({
            "type":"leaveRoom", "sequence":3, "roomId":room_id, "roundId":1
        }),
    )
    .await;
    waiting_reply::expect_reply(
        &mut owner,
        &room_id,
        joined_revision,
        json!({"type":"leftRoom", "roomId":room_id, "sequence":3}),
    )
    .await;
    let vacant =
        waiting_reply::expect_waiting_update(&mut unbound, &room_id, joined_revision, |room| {
            room["seats"][0]["occupied"] == false
        })
        .await;
    let vacant_revision = vacant["room"]["revision"].as_u64().unwrap();

    send(
        &mut owner,
        json!({"type":"joinRoom", "sequence":4, "roomCode":code}),
    )
    .await;
    let replacement = next(&mut owner).await;
    assert_eq!(replacement["youSeat"], "one");
    assert_ne!(replacement["resumeToken"], created["resumeToken"]);
    assert_eq!(replacement["room"]["settingsEditor"], Value::Null);
    let replacement_revision = replacement["room"]["revision"].as_u64().unwrap();
    assert!(replacement_revision > vacant_revision);
    send(
        &mut owner,
        json!({"type":"createRoom", "sequence":5, "configuration":{"controllers":["human","human"],"cpuDifficulty":null,"board":{"width":15,"height":13},"environment":{"stormCloud":true}}}),
    )
    .await;
    waiting_reply::expect_error(
        &mut owner,
        &room_id,
        "alreadyInRoom",
        5,
        replacement_revision,
    )
    .await;
    server.abort();
}
