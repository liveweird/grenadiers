use super::super::{publication, types::RoomIdentity};
use super::*;
use grenadiers_core::{
    protocol::{RoomCode, RoomId},
    room::RoomLifecycle,
};

fn running() -> RoomLifecycle {
    let mut lifecycle = RoomLifecycle::new();
    for seat in SeatId::ALL {
        lifecycle.occupy(seat).unwrap();
    }
    for seat in SeatId::ALL {
        lifecycle.set_ready(seat, true).unwrap();
    }
    for _ in 0..180 {
        lifecycle.advance_idle_tick().unwrap();
    }
    assert_eq!(lifecycle.phase(), grenadiers_core::room::Phase::Running);
    lifecycle
}

fn identity() -> RoomIdentity {
    RoomIdentity {
        id: RoomId::new("00000000000000000000000000000001").unwrap(),
        code: RoomCode::new("ABCDEFGH").unwrap(),
        profile: RoomProfile::FixedClassicV1,
    }
}

#[test]
fn fixed_profile_constructs_the_rules_it_publishes_with_empty_v2_items() {
    let lifecycle = running();
    let identity = identity();
    let state = identity.profile.initial_state(lifecycle.tick()).unwrap();
    assert_eq!(state.rules().version(), 1);
    assert_eq!(state.tick(), lifecycle.tick());
    assert_eq!(state.running_updates(), 0);
    let (view, encoded) = publication::snapshot(&identity, &lifecycle, 1, &state, &[]).unwrap();
    assert_eq!(view.rules_version, RulesVersion::ClassicV1);
    assert_eq!(view.map_id, MapId::FixedV1);
    let value: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(value["room"]["rulesVersion"], 1);
    assert_eq!(value["room"]["mapId"], "fixed_v1");
    assert_eq!(value["state"]["pickups"], serde_json::json!([]));
}

#[test]
fn publication_rejects_a_supported_state_that_disagrees_with_the_actual_profile() {
    let lifecycle = running();
    let identity = identity();
    let original = identity.profile.initial_state(lifecycle.tick()).unwrap();
    let actors = std::array::from_fn(|index| {
        let actor = &original.actors()[index];
        ActorInit {
            id: actor.id(),
            seat: actor.seat(),
            position: actor.position(),
            heading: actor.heading(),
            speed: actor.speed(),
        }
    });
    let wrong = State::new_at_tick(
        RulesConfig::classic_v2(),
        original.world().clone(),
        actors,
        lifecycle.tick(),
    )
    .unwrap();
    assert_eq!(
        publication::snapshot(&identity, &lifecycle, 1, &wrong, &[]),
        Err(publication::PublicationError)
    );
}
