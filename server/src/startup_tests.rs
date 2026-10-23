use std::path::PathBuf;

use grenadiers_core::protocol::v10::{MapId, RulesVersion};

use crate::{
    AppState, StartupError,
    game::{BindingKey, ConnectionId},
};

#[tokio::test]
async fn default_configuration_creates_expanded_v5_rooms() {
    let state = AppState::from_configuration(None, None, None).unwrap();

    let admission = state
        .game_rooms
        .create(
            BindingKey {
                connection_id: ConnectionId(1),
                generation: 1,
            },
            std::time::Instant::now(),
        )
        .await
        .unwrap();

    assert_eq!(admission.room.rules_version, RulesVersion::ExpandedV5);
    assert_eq!(admission.room.map_id, MapId::GeneratedV2);
}

#[test]
fn seed_configuration_failure_is_redacted_and_distinct_from_database_failure() {
    let secret_path = PathBuf::from(format!(
        "/tmp/grenadiers-secret-missing-seed-{}",
        std::process::id()
    ));
    let result = AppState::from_configuration(None, None, Some(secret_path.clone()));
    let Err(error) = result else {
        panic!("missing seed configuration should fail");
    };

    assert_eq!(error, StartupError::RoundSeed);
    let diagnostics = format!("{error:?} {error}");
    assert!(!diagnostics.contains(secret_path.to_string_lossy().as_ref()));

    let result = AppState::from_configuration(None, Some("not a postgres URL".to_owned()), None);
    assert!(matches!(result, Err(StartupError::Database)));
}
