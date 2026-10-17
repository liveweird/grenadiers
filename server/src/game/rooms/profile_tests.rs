use super::super::{publication, types::RoomIdentity};
use super::*;
use grenadiers_core::{
    game::RoundAssignment,
    protocol::{RoomCode, RoomId},
    room::RoomLifecycle,
};

fn running() -> RoomLifecycle {
    let mut lifecycle = RoomLifecycle::new();
    for seat in SeatId::ALL {
        lifecycle.occupy(seat).unwrap();
        lifecycle.set_ready(seat, true).unwrap();
    }
    for _ in 0..180 {
        lifecycle.advance_idle_tick().unwrap();
    }
    assert_eq!(lifecycle.phase(), grenadiers_core::room::Phase::Running);
    lifecycle
}

fn identity(profile: RoomProfile) -> RoomIdentity {
    RoomIdentity {
        id: RoomId::new("00000000000000000000000000000001").unwrap(),
        code: RoomCode::new("ABCDEFGH").unwrap(),
        profile,
    }
}

fn fixed_seed(seed: [u8; 32]) -> RoundSeedSource {
    RoundSeedSource::with_entropy(move |bytes| {
        bytes.copy_from_slice(&seed);
        Ok(())
    })
}

#[test]
fn fixed_profile_constructs_the_rules_it_publishes_without_consuming_entropy() {
    let lifecycle = running();
    let identity = identity(RoomProfile::FixedClassicV1);
    let source = RoundSeedSource::with_entropy(|_| panic!("fixed profile requested entropy"));
    let initial = identity
        .profile
        .initial_round(lifecycle.tick(), &source)
        .unwrap();
    assert!(initial.generation_record.is_none());
    assert_eq!(initial.state.rules().version(), 1);
    assert_eq!(initial.state.tick(), lifecycle.tick());
    assert_eq!(initial.state.running_updates(), 0);
    let (view, encoded) =
        publication::snapshot(&identity, &lifecycle, 1, &initial.state, &[]).unwrap();
    assert_eq!(view.rules_version, RulesVersion::ClassicV1);
    assert_eq!(view.map_id, MapId::FixedV1);
    let value: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(value["room"]["rulesVersion"], 1);
    assert_eq!(value["room"]["mapId"], "fixed_v1");
    assert_eq!(value["state"]["pickups"], serde_json::json!([]));
}

#[test]
fn generated_profile_constructs_classic_v2_with_assignment_spawns_and_private_record() {
    let lifecycle = running();
    let identity = identity(RoomProfile::GeneratedClassicV2);
    let initial = identity
        .profile
        .initial_round(lifecycle.tick(), &fixed_seed([19; 32]))
        .unwrap();
    let record = initial.generation_record.expect("generated record");
    assert_eq!(record.seed(), [19; 32]);
    assert_eq!(initial.state.rules().version(), 2);
    assert_eq!(initial.state.round_assignment(), RoundAssignment::first());
    let assigned = RoundAssignment::first().spawns(initial.state.world().arena());
    for actor in initial.state.actors() {
        assert_eq!(
            actor.position(),
            Position::center(assigned[actor.seat().index()])
        );
    }
    assert_eq!(initial.state.actors()[0].heading(), Direction::East);
    assert_eq!(initial.state.actors()[1].heading(), Direction::West);

    let (view, encoded) =
        publication::snapshot(&identity, &lifecycle, 1, &initial.state, &[]).unwrap();
    assert_eq!(view.rules_version, RulesVersion::ClassicV2);
    assert_eq!(view.map_id, MapId::GeneratedV1);
    let value: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(value["room"]["rulesVersion"], 2);
    assert_eq!(value["room"]["mapId"], "generated_v1");
    assert_eq!(value["state"]["pickups"], serde_json::json!([]));
    assert!(!encoded.contains(&"13".repeat(32)));
    for private in [
        "seed",
        "generationRecord",
        "concealed",
        "pending",
        "assignment",
    ] {
        assert!(!encoded.contains(&format!("\"{private}\"")));
    }
}

#[test]
fn publication_rejects_a_classic_v1_state_for_the_generated_profile() {
    let lifecycle = running();
    let generated_identity = identity(RoomProfile::GeneratedClassicV2);
    let fixed = RoomProfile::FixedClassicV1
        .initial_round(
            lifecycle.tick(),
            &RoundSeedSource::with_entropy(|_| panic!("fixed profile requested entropy")),
        )
        .unwrap();

    assert_eq!(
        publication::snapshot(&generated_identity, &lifecycle, 1, &fixed.state, &[]),
        Err(publication::PublicationError)
    );
}
