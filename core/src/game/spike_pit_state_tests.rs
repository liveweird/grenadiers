use std::collections::{BTreeMap, BTreeSet};

use super::{ActorInit, RoundActorInit, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    CellId, DamagePreventionSource, Direction, EffectKind, EnabledItems, EntityId, Equipment,
    PickupKind, Position, RoundAssignment, RulesConfig, SeatId, SpeedLevel, SpikePitClockError,
    SpikePitPhase, StepError, StepEventKind, StormCloudConfig, Tick, World,
};
use crate::room::{
    RespawnLairConfig, RoomSettings, SpikePitConfig, SpikePitSettings,
};

pub(super) const PIT: CellId = CellId::new(6, 5);
pub(super) const SAFE: CellId = CellId::new(1, 1);

pub(super) fn arena(blocks: BTreeSet<CellId>) -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if row == 0 || row == ARENA_HEIGHT - 1 || column == 0 || column == ARENA_WIDTH - 1 {
                terrain[usize::from(row * ARENA_WIDTH + column)] = Terrain::Wall;
            }
        }
    }
    Arena::new(
        terrain,
        blocks,
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap()
}

pub(super) fn rules(items: EnabledItems, pits: SpikePitConfig) -> RulesConfig {
    RulesConfig::environment_v10(
        RoomSettings::DEFAULT.with_enabled_items(items),
        StormCloudConfig::Disabled,
        RespawnLairConfig::Disabled,
        pits,
    )
}

pub(super) fn actors(one: CellId, two: CellId) -> [ActorInit; 2] {
    [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: Position::center(one),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        },
        ActorInit {
            id: EntityId(2),
            seat: SeatId::Two,
            position: Position::center(two),
            heading: Direction::West,
            speed: SpeedLevel::Base,
        },
    ]
}

pub(super) fn state_at(
    tick: Tick,
    items: EnabledItems,
    pits: SpikePitConfig,
    one: CellId,
    two: CellId,
) -> State {
    State::new_at_tick(
        rules(items, pits),
        World::new(arena(BTreeSet::new())),
        actors(one, two),
        tick,
    )
    .unwrap()
}

pub(super) fn enabled() -> SpikePitConfig {
    SpikePitConfig::Enabled(SpikePitSettings::new(6, 60).unwrap())
}

pub(super) fn step_to(game: &mut State, tick: u64) {
    while game.tick().0 < tick {
        game.step(&[]).unwrap();
    }
}

#[test]
fn rise_resolves_before_movement_and_terminal_ticks_freeze_the_view() {
    let mut game = state_at(Tick(0), EnabledItems::EMPTY, enabled(), PIT, SAFE);
    assert_eq!(game.spike_pits().unwrap()[0].cell, PIT);
    game.step(&[]).unwrap();
    assert_eq!(game.spike_pits().unwrap()[0].phase, SpikePitPhase::Warning);
    step_to(&mut game, 6);

    let start = game.actors()[0].position();
    let report = game
        .step(&[crate::game::SeatInput {
            seat: SeatId::One,
            action: crate::game::InputAction::Press(Direction::East),
        }])
        .unwrap();
    assert!(!game.actors()[0].alive());
    assert_eq!(game.actors()[0].position(), start);
    assert!(report.traces[0].is_empty());
    assert_eq!(game.spike_pits().unwrap()[0].phase, SpikePitPhase::Raised);

    let frozen = game.spike_pits();
    game.step(&[]).unwrap();
    assert_eq!(game.spike_pits(), frozen);
}

#[test]
fn shield_then_heart_use_normal_damage_priority_and_grace() {
    let items = EnabledItems::HEART_AND_SHIELD;
    let mut game = state_at(Tick(0), items, enabled(), PIT, SAFE);
    game.replace_test_actor_equipment(SeatId::One, Equipment::Heart);
    game.apply_test_effect(SeatId::One, EffectKind::Shield, Tick(1));
    step_to(&mut game, 6);

    let shield = game.step(&[]).unwrap();
    assert!(game.actors()[0].alive());
    assert_eq!(game.actors()[0].inventory().equipment(), Some(Equipment::Heart));
    assert!(shield.events.iter().any(|event| matches!(
        event.kind,
        StepEventKind::DamagePrevented {
            source: DamagePreventionSource::Shield,
            ..
        }
    )));

    game.clear_test_effects(SeatId::One);
    let heart = game.step(&[]).unwrap();
    assert!(game.actors()[0].alive());
    assert_eq!(game.actors()[0].inventory().equipment(), None);
    assert!(heart.events.iter().any(|event| matches!(
        event.kind,
        StepEventKind::DamagePrevented {
            source: DamagePreventionSource::Heart,
            ..
        }
    )));
    game.step(&[]).unwrap();
    assert!(game.actors()[0].alive(), "heart grace covers repeated raised hits");
}

#[test]
fn simultaneous_occupants_all_receive_the_raised_hit() {
    let mut game = state_at(Tick(0), EnabledItems::EMPTY, enabled(), PIT, SAFE);
    game.set_test_actor_pose(SeatId::Two, Position::center(PIT), Direction::West);
    game.step(&[]).unwrap();
    step_to(&mut game, 7);

    assert!(game.actors().iter().all(|actor| !actor.alive()));
    assert_eq!(
        game.result(),
        Some(crate::game::RoundResult::Draw(
            crate::game::DrawReason::MutualElimination
        ))
    );
}

#[test]
fn disabled_profiles_are_dormant_and_enabled_can_have_an_empty_layout() {
    let ninth = State::new(
        RulesConfig::environment_v9(
            RoomSettings::DEFAULT,
            StormCloudConfig::Disabled,
            RespawnLairConfig::Disabled,
        ),
        World::new(arena(BTreeSet::new())),
        actors(PIT, SAFE),
    )
    .unwrap();
    assert!(ninth.spike_pits().is_none());
    assert!(
        state_at(Tick(0), EnabledItems::EMPTY, SpikePitConfig::Disabled, PIT, SAFE)
            .spike_pits()
            .is_none()
    );

    let mut blocks = (1..ARENA_HEIGHT - 1)
        .flat_map(|row| (1..ARENA_WIDTH - 1).map(move |column| CellId::new(column, row)))
        .collect::<BTreeSet<_>>();
    for cell in [CellId::new(1, 1), CellId::new(2, 1), CellId::new(1, 2)] {
        blocks.remove(&cell);
    }
    for cell in [
        CellId::new(13, 11),
        CellId::new(12, 11),
        CellId::new(13, 10),
    ] {
        blocks.remove(&cell);
    }
    let empty = State::new(
        rules(EnabledItems::EMPTY, enabled()),
        World::new(arena(blocks)),
        actors(SAFE, CellId::new(13, 11)),
    )
    .unwrap();
    assert_eq!(empty.spike_pits(), Some(Vec::new()));
}

#[test]
fn clock_overflow_and_later_pickup_failure_rollback_the_entire_v10_tick() {
    let mut horizon = state_at(
        Tick(u64::MAX - 1),
        EnabledItems::EMPTY,
        enabled(),
        PIT,
        SAFE,
    );
    let before = horizon.clone();
    assert_eq!(
        horizon.step(&[]),
        Err(StepError::SpikePitClock(SpikePitClockError::DeadlineOverflow))
    );
    assert_eq!(horizon, before);

    let items = EnabledItems::SHIELD;
    let world = World::with_pickups(
        arena(BTreeSet::new()),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(PIT, PickupKind::Shield)]),
    )
    .unwrap();
    let mut late = State::new_at_tick(
        rules(items, enabled()),
        world,
        actors(PIT, SAFE),
        Tick(u64::MAX - 100),
    )
    .unwrap();
    let before = late.clone();
    assert!(matches!(
        late.step(&[]),
        Err(StepError::EffectExpiryOverflow(EffectKind::Shield))
    ));
    assert_eq!(late, before);
}

#[test]
fn new_round_rebuilds_idle_clocks() {
    let rules = rules(EnabledItems::EMPTY, enabled());
    let mut game = state_at(Tick(0), EnabledItems::EMPTY, enabled(), PIT, SAFE);
    game.step(&[]).unwrap();
    assert_eq!(game.spike_pits().unwrap()[0].phase, SpikePitPhase::Warning);

    let reset = State::new_round(
        rules,
        World::new(arena(BTreeSet::new())),
        actors(PIT, SAFE).map(|actor| RoundActorInit {
            actor,
            participating: true,
        }),
        Tick(20),
        RoundAssignment::first(),
        None,
    )
    .unwrap();
    assert!(
        reset
            .spike_pits()
            .unwrap()
            .iter()
            .all(|pit| pit.phase == SpikePitPhase::Idle && pit.remaining_ticks == 0)
    );
}
