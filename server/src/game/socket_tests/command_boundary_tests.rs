use super::*;

fn sequenced_commands() -> [Value; 9] {
    let room_id = "00000000000000000000000000000001";
    let settings = json!({
        "startingSpeedLevel": 0,
        "turnAssistUnits": 256,
        "winsToFinish": 3,
        "bombFuseTicks": 180,
        "enabledItems": [],
        "environment": {
            "stormCloud": true,
            "lairs": {"enabled": false, "spawnSeconds": 10, "maxLivingMobs": 1}
        }
    });
    [
        json!({"type":"createRoom", "sequence":11, "configuration":{"controllers":["human","human"],"cpuDifficulty":null,"board":{"width":15,"height":13},"environment":{"stormCloud":true,"lairs":{"enabled":false,"spawnSeconds":10,"maxLivingMobs":1}}}}),
        json!({"type":"joinRoom", "sequence":12, "roomCode":"ABCDEFGH"}),
        json!({
            "type":"resumeRoom", "sequence":13, "roomId":room_id,
            "resumeToken":"0000000000000000000000000000000000000000000000000000000000000000"
        }),
        json!({
            "type":"ready", "sequence":14, "roomId":room_id, "roundId":21,
            "ready":true, "settingsRevision":31
        }),
        json!({
            "type":"updateSettings", "sequence":15, "roomId":room_id, "roundId":22,
            "settingsRevision":32, "settings":settings
        }),
        json!({
            "type":"movement", "sequence":16, "roomId":room_id, "roundId":23,
            "direction":"east"
        }),
        json!({
            "type":"manualAction", "sequence":17, "roomId":room_id, "roundId":24,
            "actionId":34, "action":"bomb"
        }),
        json!({
            "type":"leaveRoom", "sequence":18, "roomId":room_id, "roundId":25
        }),
        json!({
            "type":"rematch", "sequence":19, "roomId":room_id, "roundId":26,
            "ready":true
        }),
    ]
}

#[tokio::test]
async fn every_sequenced_command_before_authentication_is_correlated_and_policy_closed() {
    for command in sequenced_commands() {
        let (url, server) = start_server(state()).await;
        let sequence = command["sequence"].clone();
        let mut socket = connect(&url).await;
        send_json(&mut socket, command).await;
        assert_eq!(
            next_json(&mut socket).await,
            json!({
                "type":"error", "code":"notAuthenticated", "sequence":sequence
            })
        );
        let close = within(socket.next()).await.unwrap().unwrap();
        assert!(matches!(close, Message::Close(Some(frame)) if u16::from(frame.code) == 1008));
        server.abort();
    }
}

#[tokio::test]
async fn fresh_unseated_semantic_errors_consume_sequences_before_retry() {
    let state = state();
    let token = issue_token(&state);
    let (url, server) = start_server(state).await;
    let mut socket = connect(&url).await;
    assert_eq!(
        authenticate(&mut socket, &token, 17).await,
        json!({"type":"authenticated", "protocolVersion":17})
    );

    let room_id = "00000000000000000000000000000001";
    send_json(
        &mut socket,
        json!({
            "type":"updateSettings", "sequence":19, "roomId":room_id, "roundId":29,
            "settingsRevision":39, "settings":{
                "startingSpeedLevel":0, "turnAssistUnits":256, "winsToFinish":3,
                "bombFuseTicks":180, "enabledItems":[],
                "environment":{
                    "stormCloud":true,
                    "lairs":{"enabled":false,"spawnSeconds":10,"maxLivingMobs":1}
                }
            }
        }),
    )
    .await;
    assert_eq!(
        next_json(&mut socket).await,
        json!({"type":"error", "code":"notInRoom", "sequence":19})
    );

    for sequence in [19, 18] {
        send_json(
            &mut socket,
            json!({"type":"createRoom", "sequence":sequence, "configuration":{"controllers":["human","human"],"cpuDifficulty":null,"board":{"width":15,"height":13},"environment":{"stormCloud":true,"lairs":{"enabled":false,"spawnSeconds":10,"maxLivingMobs":1}}}}),
        )
        .await;
        assert_eq!(
            next_json(&mut socket).await,
            json!({"type":"error", "code":"staleSequence", "sequence":sequence})
        );
    }

    send_json(
        &mut socket,
        json!({"type":"createRoom", "sequence":20, "configuration":{"controllers":["human","human"],"cpuDifficulty":null,"board":{"width":15,"height":13},"environment":{"stormCloud":true,"lairs":{"enabled":false,"spawnSeconds":10,"maxLivingMobs":1}}}}),
    )
    .await;
    assert_eq!(next_json(&mut socket).await["type"], "admitted");
    server.abort();
}
