use std::collections::BTreeSet;

use super::super::{
    ActorInit, InputAction, MAX_INPUTS_PER_TICK, SeatInput, State, StateError, StepError,
};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::bombs::{Bomb, BombMotion, BombTrigger};
use crate::game::{
    CellId, Direction, EnabledItems, EntityId, Position, RulesConfig, SeatId, SpeedLevel,
    StepEventKind, Tick, World,
};
use crate::room::RoomSettings;

fn arena(blocks: BTreeSet<CellId>) -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    Arena::new(terrain, blocks, [CellId::new(1, 1), CellId::new(13, 11)]).unwrap()
}

fn actors() -> [ActorInit; 2] {
    [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: Position::center(CellId::new(1, 1)),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        },
        ActorInit {
            id: EntityId(2),
            seat: SeatId::Two,
            position: Position::center(CellId::new(13, 11)),
            heading: Direction::West,
            speed: SpeedLevel::Base,
        },
    ]
}

fn rules(enabled: EnabledItems) -> RulesConfig {
    RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled))
}

fn bomb(trigger: BombTrigger, owner: EntityId) -> Bomb {
    Bomb::new(EntityId(9), CellId::new(5, 5), owner, 2, Tick(0), trigger).unwrap()
}

fn world_with(bomb: Bomb, blocks: BTreeSet<CellId>) -> World {
    let mut world = World::new(arena(blocks));
    world.bombs.insert(bomb.id(), bomb);
    world
}

#[test]
fn unknown_owner_precedes_malformed_kick_history() {
    let mut restored = bomb(
        BombTrigger::Timed {
            detonates_at: Tick(100),
        },
        EntityId(99),
    );
    restored.restore_kick_history(
        Some(BombMotion {
            direction: Direction::East,
            countdown: 10,
        }),
        true,
    );

    assert_eq!(
        State::new(
            RulesConfig::classic_v1(),
            world_with(restored, BTreeSet::new()),
            actors(),
        ),
        Err(StateError::UnknownBombOwner(EntityId(99)))
    );
}

#[test]
fn constructors_reject_every_malformed_kick_history_shape() {
    let cases = [
        (
            EnabledItems::EMPTY,
            BombTrigger::Timed {
                detonates_at: Tick(100),
            },
            None,
            true,
        ),
        (
            EnabledItems::KICK,
            BombTrigger::Timed {
                detonates_at: Tick(100),
            },
            Some(BombMotion {
                direction: Direction::East,
                countdown: 0,
            }),
            false,
        ),
        (
            EnabledItems::KICK,
            BombTrigger::Timed {
                detonates_at: Tick(100),
            },
            Some(BombMotion {
                direction: Direction::East,
                countdown: 10,
            }),
            true,
        ),
        (
            EnabledItems::from_bits(EnabledItems::KICK.bits() | EnabledItems::LANDMINE.bits())
                .unwrap(),
            BombTrigger::Contact,
            Some(BombMotion {
                direction: Direction::East,
                countdown: 0,
            }),
            true,
        ),
        (
            EnabledItems::from_bits(EnabledItems::KICK.bits() | EnabledItems::LANDMINE.bits())
                .unwrap(),
            BombTrigger::Contact,
            None,
            true,
        ),
    ];

    for (enabled, trigger, motion, kicked) in cases {
        let mut restored = bomb(trigger, EntityId(1));
        restored.restore_kick_history(motion, kicked);
        assert_eq!(
            State::new(
                rules(enabled),
                world_with(restored, BTreeSet::new()),
                actors()
            ),
            Err(StateError::UnsupportedBombKickHistory(EntityId(9)))
        );
    }
}

#[test]
fn constructor_accepts_active_and_stopped_kick_history_when_kick_is_enabled() {
    for motion in [
        Some(BombMotion {
            direction: Direction::North,
            countdown: 9,
        }),
        None,
    ] {
        let mut restored = bomb(
            BombTrigger::Timed {
                detonates_at: Tick(100),
            },
            EntityId(1),
        );
        restored.restore_kick_history(motion, true);
        State::new(
            rules(EnabledItems::KICK),
            world_with(restored, BTreeSet::new()),
            actors(),
        )
        .unwrap();
    }
}

#[test]
fn terminal_steps_still_validate_history_after_input_and_tick_preflights() {
    let restored = bomb(
        BombTrigger::Timed {
            detonates_at: Tick(100),
        },
        EntityId(1),
    );
    let mut state = State::new(
        rules(EnabledItems::KICK),
        world_with(restored, BTreeSet::new()),
        actors(),
    )
    .unwrap();
    state.result = Some(super::super::RoundResult::Winner(SeatId::One));
    state
        .world
        .bombs
        .get_mut(&EntityId(9))
        .unwrap()
        .restore_kick_history(
            Some(BombMotion {
                direction: Direction::East,
                countdown: 10,
            }),
            true,
        );

    assert_eq!(
        state.step(
            &[SeatInput {
                seat: SeatId::One,
                action: InputAction::Clear,
            }; MAX_INPUTS_PER_TICK + 1]
        ),
        Err(StepError::TooManyInputs {
            count: MAX_INPUTS_PER_TICK + 1,
            maximum: MAX_INPUTS_PER_TICK,
        })
    );
    state.tick = Tick(u64::MAX);
    assert_eq!(state.step(&[]), Err(StepError::TickOverflow));
    state.tick = Tick(0);
    assert_eq!(
        state.step(&[]),
        Err(StepError::UnsupportedBombKickHistory(EntityId(9)))
    );
}

#[test]
fn remote_hop_trigger_event_keeps_exact_active_private_provenance() {
    let mut restored = bomb(BombTrigger::Remote, EntityId(1));
    restored.restore_kick_history(
        Some(BombMotion {
            direction: Direction::East,
            countdown: 0,
        }),
        true,
    );
    let enabled =
        EnabledItems::from_bits(EnabledItems::KICK.bits() | EnabledItems::REMOTE_BOMB.bits())
            .unwrap();
    let mut state = State::new(
        rules(enabled),
        world_with(restored, BTreeSet::new()),
        actors(),
    )
    .unwrap();

    let report = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    let event = report
        .events
        .iter()
        .find(|event| matches!(event.kind, StepEventKind::BombDetonated(_)))
        .unwrap();
    assert!(matches!(
        event.kind,
        StepEventKind::BombDetonated(view) if view.cell == CellId::new(6, 5)
    ));
    assert_eq!(
        event.bomb_motion,
        Some(BombMotion {
            direction: Direction::East,
            countdown: 9,
        })
    );
    assert!(event.bomb_has_been_kicked);
    assert!(state.world.bombs.is_empty());
}

#[test]
fn stopped_remote_event_keeps_kick_history_without_motion() {
    let mut restored = bomb(BombTrigger::Remote, EntityId(1));
    restored.restore_kick_history(
        Some(BombMotion {
            direction: Direction::East,
            countdown: 0,
        }),
        true,
    );
    let enabled =
        EnabledItems::from_bits(EnabledItems::KICK.bits() | EnabledItems::REMOTE_BOMB.bits())
            .unwrap();
    let mut state = State::new(
        rules(enabled),
        world_with(restored, BTreeSet::from([CellId::new(6, 5)])),
        actors(),
    )
    .unwrap();
    state.step(&[]).unwrap();
    let stopped = state.world.bombs[&EntityId(9)];
    assert_eq!(stopped.motion(), None);
    assert!(stopped.has_been_kicked());

    let report = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    let event = report
        .events
        .iter()
        .find(|event| matches!(event.kind, StepEventKind::BombDetonated(_)))
        .unwrap();
    assert_eq!(event.bomb_motion, None);
    assert!(event.bomb_has_been_kicked);
    assert!(state.world.bombs.is_empty());
}
