use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::{
    BombKnowledge, MovementIntent, Observation, ObservedActor, ObservedBomb, ObservedBombTrigger,
    ObservedFlame, UnknownBombFacts, WorkBudget,
};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::types::{HALF_FOOTPRINT, UNITS_PER_CELL};
use crate::game::{
    ActorInit, BlastKind, BombTrigger, CellId, Direction, EnabledItems, EntityId, Equipment,
    InputAction, Position, PunchActivationPolicy, RulesConfig, SeatId, SeatInput, SpeedLevel,
    State, Tick, World,
};
use crate::room::RoomSettings;

use super::{
    PulseOriginGrace, PulseRequest, RouteFailure, RouteMode, certify_escape_pulse, certify_pulse,
};

const SOURCE: CellId = CellId::new(5, 5);
const BOMB_ID: EntityId = EntityId(90);

fn tangent() -> Position {
    Position {
        x: i32::from(SOURCE.column) * UNITS_PER_CELL - HALF_FOOTPRINT,
        y: Position::center(SOURCE).y,
    }
}

fn arena() -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if row == 0 || column == 0 || row + 1 == ARENA_HEIGHT || column + 1 == ARENA_WIDTH {
                terrain[usize::from(row * ARENA_WIDTH + column)] = Terrain::Wall;
            }
        }
    }
    Arena::new(
        terrain,
        std::collections::BTreeSet::new(),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap()
}

fn empty_state(start: Position) -> State {
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::PUNCH);
    let mut state = State::new(
        RulesConfig::expanded_v5(settings)
            .with_punch_activation_policy(PunchActivationPolicy::ContactV2),
        World::new(arena()),
        [
            ActorInit {
                id: EntityId(1),
                seat: SeatId::One,
                position: Position::center(CellId::new(13, 11)),
                heading: Direction::West,
                speed: SpeedLevel::Base,
            },
            ActorInit {
                id: EntityId(2),
                seat: SeatId::Two,
                position: start,
                heading: Direction::East,
                speed: SpeedLevel::Base,
            },
        ],
    )
    .unwrap();
    state.replace_test_actor_equipment(SeatId::Two, Equipment::Punch);
    state
}

fn actual_state(start: Position) -> State {
    let mut state = empty_state(start);
    state
        .insert_test_bomb(
            BOMB_ID,
            SOURCE,
            EntityId(1),
            2,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(100),
            },
        )
        .unwrap();
    state
}

fn observed(start: Position, policy: PunchActivationPolicy) -> Observation {
    let mut parts = super::tests::observation(Vec::new(), Vec::new()).into_parts();
    parts.rules.punch_activation_policy = policy;
    parts.rules.settings = parts.rules.settings.with_enabled_items(EnabledItems::PUNCH);
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.position = start;
    actor.equipment = Some(Equipment::Punch);
    parts.bombs = BombKnowledge::Visible(vec![ObservedBomb {
        id: BOMB_ID,
        placed_at: Tick(0),
        cell: SOURCE,
        owner: EntityId(1),
        range: 2,
        trigger: ObservedBombTrigger::Timed {
            remaining_ticks: 100,
            paused: false,
        },
        blast: BlastKind::Standard,
        unknown: UnknownBombFacts::ALL,
    }]);
    Observation::new(parts).unwrap()
}

fn certify(observation: &Observation) -> Result<super::PulseLease, RouteFailure> {
    let forecast = forecast(
        observation,
        &ObservedHazardMemory::new(super::tests::key()),
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    certify_pulse(
        observation,
        &forecast,
        PulseRequest {
            movement: MovementIntent::Direction(Direction::East),
            hold_ticks: 1,
            start_window: TickWindow {
                start: Tick(1),
                end: Tick(2),
            },
            stopped_safe_until: Tick(23),
            mode: RouteMode::AllowStationaryAssumptions,
            settled_stop_tick: Tick(0),
        },
        &mut WorkBudget::new(65_536),
    )
}

fn certify_departure(observation: &Observation) -> Result<super::PulseLease, RouteFailure> {
    let forecast = forecast(
        observation,
        &ObservedHazardMemory::new(super::tests::key()),
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    certify_escape_pulse(
        observation,
        &forecast,
        PulseRequest {
            movement: MovementIntent::Direction(Direction::East),
            hold_ticks: 1,
            start_window: TickWindow {
                start: Tick(1),
                end: Tick(2),
            },
            stopped_safe_until: Tick(23),
            mode: RouteMode::AllowStationaryAssumptions,
            settled_stop_tick: Tick(0),
        },
        PulseOriginGrace {
            cell: SOURCE,
            bomb_id: BOMB_ID,
        },
        &mut WorkBudget::new(65_536),
    )
}

#[test]
fn state_proves_the_archived_tangent_would_relocate_under_contact_v2() {
    let start = Position {
        x: tangent().x - 51,
        ..tangent()
    };
    assert!(certify(&observed(start, PunchActivationPolicy::ManualV1)).is_ok());
    assert_eq!(
        certify(&observed(start, PunchActivationPolicy::ContactV2)),
        Err(RouteFailure::NoRoute)
    );

    let mut state = actual_state(start);
    state
        .step(&[SeatInput {
            seat: SeatId::Two,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
    assert_eq!(state.actor(SeatId::Two).unwrap().position(), tangent());
    assert_eq!(state.world().bombs()[&BOMB_ID].cell(), CellId::new(8, 5));
}

#[test]
fn state_and_guard_keep_one_unit_short_useful() {
    let start = Position {
        x: tangent().x - 52,
        ..tangent()
    };
    assert!(certify(&observed(start, PunchActivationPolicy::ContactV2)).is_ok());

    let mut state = actual_state(start);
    state
        .step(&[SeatInput {
            seat: SeatId::Two,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
    assert_eq!(
        state.actor(SeatId::Two).unwrap().position().x,
        tangent().x - 1
    );
    assert_eq!(state.world().bombs()[&BOMB_ID].cell(), SOURCE);
}

#[test]
fn chain_target_remains_guarded_on_its_pre_explosion_removal_tick() {
    let start = Position {
        x: tangent().x - 51,
        ..tangent()
    };
    let chain = CellId::new(5, 3);
    let observation = observed(start, PunchActivationPolicy::ContactV2)
        .map_parts(|parts| {
            let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                unreachable!()
            };
            bombs.push(ObservedBomb {
                id: EntityId(91),
                placed_at: Tick(0),
                cell: chain,
                owner: EntityId(1),
                range: 2,
                trigger: ObservedBombTrigger::Timed {
                    remaining_ticks: 1,
                    paused: false,
                },
                blast: BlastKind::Standard,
                unknown: UnknownBombFacts::ALL,
            });
        })
        .unwrap();
    assert_eq!(certify(&observation), Err(RouteFailure::NoRoute));

    let mut state = actual_state(start);
    state
        .insert_test_bomb(
            EntityId(91),
            chain,
            EntityId(1),
            2,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(1),
            },
        )
        .unwrap();
    state
        .step(&[SeatInput {
            seat: SeatId::Two,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
    assert_eq!(state.world().bombs()[&BOMB_ID].cell(), CellId::new(8, 5));
}

#[test]
fn expiring_source_flame_reopens_contact_eligibility_before_next_movement() {
    let start = Position {
        x: tangent().x - 51,
        ..tangent()
    };
    let observation = observed(start, PunchActivationPolicy::ContactV2)
        .map_parts(|parts| {
            parts.flames.push(ObservedFlame {
                cell: SOURCE,
                remaining_ticks: 1,
            });
        })
        .unwrap();
    assert_eq!(certify(&observation), Err(RouteFailure::NoRoute));

    let mut state = actual_state(start);
    state.insert_test_flame(SOURCE, Tick(1));
    state
        .step(&[SeatInput {
            seat: SeatId::Two,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
    assert_eq!(state.world().bombs()[&BOMB_ID].cell(), CellId::new(8, 5));
}

#[test]
fn state_and_origin_guard_keep_own_bomb_grace_departure_useful() {
    let observation = observed(Position::center(SOURCE), PunchActivationPolicy::ContactV2)
        .map_parts(|parts| {
            let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
                unreachable!()
            };
            actor.live_bomb_count = 1;
            let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                unreachable!()
            };
            bombs[0].owner = actor.id;
        })
        .unwrap();
    assert!(certify_departure(&observation).is_ok());

    let mut state = empty_state(Position::center(SOURCE));
    state
        .step(&[SeatInput {
            seat: SeatId::Two,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    let id = *state.world().bombs().keys().next().unwrap();
    state
        .step(&[SeatInput {
            seat: SeatId::Two,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
    assert_eq!(state.world().bombs()[&id].cell(), SOURCE);
    assert!(state.actor(SeatId::Two).unwrap().position().x > Position::center(SOURCE).x);
}
