use super::*;

#[tokio::test]
async fn disconnect_reserves_seat_even_when_no_more_commands_arrive() {
    let state = AppState::without_database([ORIGIN.to_owned()]);
    let tokens = [issue_token(&state), issue_token(&state)];
    let (url, server) = start_server(state).await;
    let mut one = connect(&url, &tokens[0]).await;
    let mut two = connect(&url, &tokens[1]).await;
    send(
        &mut one,
        json!({"type":"createRoom", "sequence":1, "configuration":{"controllers":["human","human"],"cpuDifficulty":null,"board":{"width":15,"height":13},"environment":{"stormCloud":true}}}),
    )
    .await;
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
    assert_eq!(update["room"]["seats"][1]["occupied"], true);
    assert_eq!(
        update["room"]["seats"][1]["connection"]["type"],
        "disconnected"
    );
    assert_eq!(update["room"]["revision"], 5);
    server.abort();
}
