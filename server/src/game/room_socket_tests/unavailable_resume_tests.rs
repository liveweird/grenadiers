use super::*;

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
