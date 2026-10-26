use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH};
use crate::game::{
    CellId, Direction, EntityId, Position, SeatId, SpeedLevel, Terrain, Tick, generate_arena,
};
use crate::room::RoomSettings;

use super::*;

fn world(round: u64) -> WorldId {
    WorldId {
        room_instance: 7,
        physical_round: round,
    }
}

fn decision(world: WorldId, epoch: u64, tick: u64, serial: u64) -> DecisionId {
    DecisionId {
        world,
        command_epoch: epoch,
        observation_tick: Tick(tick),
        checked_request_serial: serial,
    }
}

fn context(id: DecisionId) -> AdmissionContext {
    AdmissionContext {
        world: id.world,
        command_epoch: id.command_epoch,
        observation_tick: id.observation_tick,
        checked_request_serial: id.checked_request_serial,
    }
}

fn visible(seat: SeatId, id: u32) -> ObservedActor {
    ObservedActor::Visible(VisibleActor {
        id: EntityId(id),
        seat,
        position: Position::center(CellId::new(1, 1)),
        heading: Direction::East,
        speed: SpeedLevel::Base,
        alive: true,
        bomb_capacity: 1,
        bomb_range: 2,
        live_bomb_count: 0,
        equipment: None,
        weapon: None,
        effects: Vec::new(),
        damage_grace_remaining_ticks: 0,
        voluntary_actions_frozen: false,
        last_teleport_tick: None,
        unknown: UnknownActorFacts::ALL,
    })
}

fn observation(bombs: BombKnowledge) -> Observation {
    Observation::new(ObservationParts {
        world: world(1),
        tick: Tick(4),
        recipient: SeatId::Two,
        rules: ObservedRules {
            version: 5,
            tick_rate: 60,
            running_updates: 4,
            remaining_ticks: 100,
            settings: RoomSettings::DEFAULT,
        },
        arena: ObservedArena {
            width: ARENA_WIDTH,
            height: ARENA_HEIGHT,
            terrain: vec![Terrain::Floor; ARENA_CELL_COUNT],
            blocks: Vec::new(),
        },
        actors: [
            ObservedActor::Hidden {
                id: EntityId(1),
                seat: SeatId::One,
                alive: true,
            },
            visible(SeatId::Two, 2),
        ],
        bombs,
        flames: Vec::new(),
        oil: Vec::new(),
        pickups: Vec::new(),
    })
    .unwrap()
}

/// Deliberately tiny test fixture, not a production policy or search budget.
fn fixture_policy(observation: &Observation, rng: &mut CpuRng, work: &mut WorkBudget) -> Intent {
    if work.try_take(1).is_err() {
        return Intent::STOP;
    }
    let actor_alive = observation
        .parts()
        .actors
        .iter()
        .find(|actor| actor.seat() == observation.parts().recipient)
        .is_some_and(|actor| matches!(actor, ObservedActor::Visible(actor) if actor.alive));
    if !actor_alive {
        return Intent::STOP;
    }
    let directions = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];
    let index = rng.bounded_u32(directions.len() as u32).unwrap() as usize;
    Intent::moving(directions[index])
}

#[test]
fn hidden_actor_exposes_only_public_identity_and_liveness() {
    let observation = observation(BombKnowledge::Visible(Vec::new()));
    assert_eq!(
        observation.parts().actors[0],
        ObservedActor::Hidden {
            id: EntityId(1),
            seat: SeatId::One,
            alive: true,
        }
    );
    let ObservedActor::Visible(recipient) = &observation.parts().actors[1] else {
        panic!("recipient must be visible")
    };
    assert_eq!(recipient.unknown, UnknownActorFacts::ALL);
}

#[test]
fn unavailable_bombs_are_distinct_from_a_visible_empty_layer() {
    let unavailable = observation(BombKnowledge::Unavailable);
    let empty = observation(BombKnowledge::Visible(Vec::new()));
    assert_ne!(unavailable, empty);
}

#[test]
fn observation_rejects_structural_overflow_and_hidden_recipient() {
    let bomb = ObservedBomb {
        id: EntityId(9),
        cell: CellId::new(1, 1),
        owner: EntityId(2),
        range: 2,
        trigger: ObservedBombTrigger::Remote,
        blast: crate::game::BlastKind::Standard,
        unknown: UnknownBombFacts::ALL,
    };
    let mut parts = observation(BombKnowledge::Visible(Vec::new())).into_parts();
    parts.bombs = BombKnowledge::Visible(vec![bomb; MAX_OBSERVED_BOMBS + 1]);
    assert_eq!(
        Observation::new(parts).unwrap_err(),
        ObservationError::InvalidBombs
    );

    let mut parts = observation(BombKnowledge::Unavailable).into_parts();
    parts.actors[1] = ObservedActor::Hidden {
        id: EntityId(2),
        seat: SeatId::Two,
        alive: true,
    };
    assert_eq!(
        Observation::new(parts).unwrap_err(),
        ObservationError::InvalidActors
    );
}

#[test]
fn fixture_policy_is_deterministic_without_hidden_authoritative_facts() {
    let observation = observation(BombKnowledge::Unavailable);
    let key = MemoryKey {
        world: world(1),
        cpu_seat: SeatId::Two,
    };
    let mut first_rng = CpuRng::from_seed(key, [3; 32]);
    let mut second_rng = CpuRng::from_seed(key, [3; 32]);
    let first = fixture_policy(&observation, &mut first_rng, &mut WorkBudget::new(1));
    let second = fixture_policy(&observation, &mut second_rng, &mut WorkBudget::new(1));
    assert_eq!(first, second);
    assert_eq!(first_rng.draws(), 1);
    assert_eq!(second_rng.draws(), 1);
}

#[test]
fn reconnect_epoch_preserves_memory_but_new_world_reset_clears_it() {
    let key = MemoryKey {
        world: world(1),
        cpu_seat: SeatId::Two,
    };
    let mut memory = CpuMemory::new(key, 2).unwrap();
    let first = decision(key.world, 4, 10, 8);
    assert_eq!(
        admit_decision(&mut memory, context(first), first, &mut WorkBudget::new(1)),
        DecisionAdmission::Accepted
    );
    memory.remember(first, Intent::STOP).unwrap();

    let reconnected = decision(key.world, 5, 10, 0);
    assert_eq!(
        admit_decision(
            &mut memory,
            context(reconnected),
            reconnected,
            &mut WorkBudget::new(1)
        ),
        DecisionAdmission::Accepted
    );
    assert_eq!(memory.history().len(), 1);

    memory.reset_for(MemoryKey {
        world: world(2),
        cpu_seat: SeatId::Two,
    });
    assert!(memory.history().is_empty());
    assert_eq!(memory.last_admitted(), None);
}

#[test]
fn history_eviction_is_explicit_and_bounded() {
    let key = MemoryKey {
        world: world(1),
        cpu_seat: SeatId::Two,
    };
    let mut memory = CpuMemory::new(key, 1).unwrap();
    let first = decision(key.world, 1, 1, 1);
    admit_decision(&mut memory, context(first), first, &mut WorkBudget::new(1));
    assert_eq!(
        memory.remember(first, Intent::STOP).unwrap(),
        MemoryUpdate::Added
    );
    assert_eq!(
        memory.remember(first, Intent::STOP),
        Err(MemoryError::AlreadyRemembered)
    );
    assert_eq!(memory.history().len(), 1);
    let second = decision(key.world, 1, 2, 2);
    admit_decision(
        &mut memory,
        context(second),
        second,
        &mut WorkBudget::new(1),
    );
    assert_eq!(
        memory.remember(second, Intent::moving(Direction::North)),
        Ok(MemoryUpdate::Evicted(MemoryEntry {
            decision: first,
            intent: Intent::STOP,
        }))
    );
    assert_eq!(memory.history().len(), 1);
}

#[test]
fn stale_duplicate_wrong_world_and_exhausted_decisions_fail_closed() {
    let key = MemoryKey {
        world: world(1),
        cpu_seat: SeatId::Two,
    };
    let mut memory = CpuMemory::new(key, 2).unwrap();
    let accepted = decision(key.world, 2, 9, 3);
    admit_decision(
        &mut memory,
        context(accepted),
        accepted,
        &mut WorkBudget::new(1),
    );

    for (candidate, reason) in [
        (accepted, RejectionReason::DuplicateRequest),
        (
            decision(key.world, 2, 8, 4),
            RejectionReason::StaleObservation,
        ),
        (decision(world(2), 2, 10, 4), RejectionReason::WrongWorld),
    ] {
        let rejected = admit_decision(
            &mut memory,
            context(candidate),
            candidate,
            &mut WorkBudget::new(1),
        );
        assert_eq!(rejected, DecisionAdmission::Rejected(reason));
        assert_eq!(rejected.fallback_intent(), Some(Intent::STOP));
    }

    let next = decision(key.world, 2, 10, 4);
    let exhausted = admit_decision(&mut memory, context(next), next, &mut WorkBudget::new(0));
    assert_eq!(
        exhausted,
        DecisionAdmission::Rejected(RejectionReason::WorkExhausted)
    );
}

#[test]
fn completed_decision_must_match_caller_current_identity() {
    let key = MemoryKey {
        world: world(1),
        cpu_seat: SeatId::Two,
    };
    let mut memory = CpuMemory::new(key, 2).unwrap();
    let current = decision(key.world, 7, 20, 12);
    let stale = decision(key.world, 6, 20, 12);
    let rejected = admit_decision(
        &mut memory,
        context(current),
        stale,
        &mut WorkBudget::new(1),
    );
    assert_eq!(
        rejected,
        DecisionAdmission::Rejected(RejectionReason::StaleCommandEpoch)
    );
    assert_eq!(memory.last_admitted(), None);
}

#[test]
fn work_overflow_is_rejected_without_changing_accounting() {
    let mut work = WorkBudget::new(u32::MAX);
    work.try_take(u32::MAX).unwrap();
    assert_eq!(work.try_take(1), Err(WorkError::Exhausted));
    assert_eq!(work.spent(), u32::MAX);
    assert!(work.exhausted());
}

#[test]
fn cpu_rng_has_fixed_draws_redacted_debug_and_no_arena_stream_effect() {
    let arena_seed = [7; 32];
    let before = generate_arena(arena_seed);
    let key = MemoryKey {
        world: world(1),
        cpu_seat: SeatId::Two,
    };
    let mut cpu_rng = CpuRng::from_seed(key, [9; 32]);
    assert!(format!("{cpu_rng:?}").contains("<redacted>"));
    assert_eq!(cpu_rng.bounded_u32(0), Err(RandomError::EmptyRange));
    assert_eq!(cpu_rng.draws(), 0);
    for upper in [1, 2, 17, u32::MAX] {
        assert!(cpu_rng.bounded_u32(upper).unwrap() < upper);
    }
    assert_eq!(cpu_rng.draws(), 4);
    assert_eq!(before, generate_arena(arena_seed));

    let reset_key = MemoryKey {
        world: world(2),
        cpu_seat: SeatId::Two,
    };
    cpu_rng.reset_for(reset_key, [9; 32]);
    assert_eq!(cpu_rng.key(), reset_key);
    assert_eq!(cpu_rng.draws(), 0);
    let mut fresh = CpuRng::from_seed(reset_key, [9; 32]);
    assert_eq!(cpu_rng.bounded_u32(17), fresh.bounded_u32(17));
}
