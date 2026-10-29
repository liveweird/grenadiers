use serde_json::{Value, json};

use super::{ClientSocket, next, within};

pub(super) async fn expect_waiting_update(
    socket: &mut ClientSocket,
    room_id: &Value,
    minimum_revision: u64,
    matches: impl Fn(&Value) -> bool,
) -> Value {
    within(async {
        loop {
            let message = next(socket).await;
            let revision = assert_waiting_update(&message, room_id);
            if revision >= minimum_revision && matches(&message["room"]) {
                return message;
            }
        }
    })
    .await
}

pub(super) async fn expect_invalid_phase(
    socket: &mut ClientSocket,
    room_id: &Value,
    sequence: u64,
    minimum_revision: u64,
) {
    within(async {
        loop {
            let message = next(socket).await;
            if inspect(&message, room_id, sequence, minimum_revision) {
                return;
            }
        }
    })
    .await;
}

fn inspect(message: &Value, room_id: &Value, sequence: u64, minimum_revision: u64) -> bool {
    if message["type"] == "error" {
        assert_eq!(
            *message,
            json!({"type":"error", "code":"invalidPhase", "sequence":sequence})
        );
        return true;
    }
    let revision = assert_waiting_update(message, room_id);
    assert!(
        revision >= minimum_revision,
        "stale waiting-room update: {message}"
    );
    false
}

fn assert_waiting_update(message: &Value, room_id: &Value) -> u64 {
    assert_eq!(message["type"], "roomState", "unexpected frame: {message}");
    assert!(
        message.get("youSeat").is_none(),
        "private seat leaked: {message}"
    );
    assert!(
        message.get("resumeToken").is_none(),
        "private credential leaked: {message}"
    );
    assert_eq!(message["room"]["roomId"], *room_id);
    assert_eq!(message["room"]["phase"]["type"], "waiting");
    message["room"]["revision"]
        .as_u64()
        .unwrap_or_else(|| panic!("invalid waiting-room revision: {message}"))
}

#[test]
fn queued_waiting_updates_are_strictly_validated_before_the_correlated_error() {
    let room_id = json!("00000000-0000-0000-0000-000000000001");
    let frames = [
        json!({
            "type":"roomState",
            "room": {
                "roomId":room_id, "revision":7, "settingsRevision":2,
                "phase":{"type":"waiting"}
            }
        }),
        json!({"type":"error", "code":"invalidPhase", "sequence":4}),
    ];
    let terminal = frames
        .iter()
        .position(|frame| inspect(frame, &room_id, 4, 7));
    assert_eq!(terminal, Some(1));
}
