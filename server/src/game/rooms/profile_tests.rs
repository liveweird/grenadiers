use super::super::types::RoomIdentity;
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
        .initial_round(lifecycle.tick(), RoundAssignment::first(), &source)
        .unwrap();
    assert!(initial.generation_record.is_none());
    assert_eq!(initial.state.rules().version(), 1);
    assert_eq!(initial.state.tick(), lifecycle.tick());
    assert_eq!(initial.state.running_updates(), 0);
}

#[test]
fn generated_profile_constructs_classic_v2_with_assignment_spawns_and_private_record() {
    let lifecycle = running();
    let identity = identity(RoomProfile::GeneratedClassicV2);
    let initial = identity
        .profile
        .initial_round(
            lifecycle.tick(),
            RoundAssignment::first(),
            &fixed_seed([19; 32]),
        )
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
}

#[test]
fn profile_honors_later_round_assignment_for_spawns_and_facing() {
    let lifecycle = running();
    let assignment = RoundAssignment::new(std::num::NonZeroU32::new(2).unwrap());
    let initial = RoomProfile::FixedClassicV1
        .initial_round(
            lifecycle.tick(),
            assignment,
            &RoundSeedSource::with_entropy(|_| panic!("fixed profile requested entropy")),
        )
        .unwrap();
    let assigned = assignment.spawns(initial.state.world().arena());

    assert_eq!(initial.state.round_assignment(), assignment);
    for actor in initial.state.actors() {
        assert_eq!(
            actor.position(),
            Position::center(assigned[actor.seat().index()])
        );
    }
    assert_eq!(initial.state.actors()[0].heading(), Direction::West);
    assert_eq!(initial.state.actors()[1].heading(), Direction::East);
}
