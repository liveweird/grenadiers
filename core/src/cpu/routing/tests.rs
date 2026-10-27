use std::collections::{BTreeMap, BTreeSet};

use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::*;
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    ActorInit, CellId, Direction, EntityId, InputAction, Position, RulesConfig, SeatId, SeatInput,
    SpeedLevel, State, Tick, World,
};
use crate::room::RoomSettings;

use super::*;

pub(super) fn key() -> MemoryKey {
    MemoryKey {
        world: WorldId {
            room_instance: 8,
            physical_round: 3,
        },
        cpu_seat: SeatId::Two,
    }
}

fn actor(seat: SeatId, id: u32, cell: CellId) -> ObservedActor {
    ObservedActor::Visible(VisibleActor {
        id: EntityId(id),
        seat,
        position: Position::center(cell),
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

fn terrain() -> Vec<Terrain> {
    let mut value = vec![Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if row == 0 || column == 0 || row + 1 == ARENA_HEIGHT || column + 1 == ARENA_WIDTH {
                value[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    value
}

pub(crate) fn observation(flames: Vec<ObservedFlame>, pickups: Vec<ObservedPickup>) -> Observation {
    Observation::new(ObservationParts {
        world: key().world,
        tick: Tick(0),
        recipient: SeatId::Two,
        rules: ObservedRules {
            version: 5,
            tick_rate: 60,
            running_updates: 0,
            remaining_ticks: 1_000,
            settings: RoomSettings::DEFAULT,
        },
        arena: ObservedArena {
            width: ARENA_WIDTH,
            height: ARENA_HEIGHT,
            terrain: terrain(),
            blocks: Vec::new(),
        },
        actors: [
            actor(SeatId::One, 1, CellId::new(13, 11)),
            actor(SeatId::Two, 2, CellId::new(2, 1)),
        ],
        bombs: BombKnowledge::Visible(Vec::new()),
        flames,
        oil: Vec::new(),
        pickups,
    })
    .unwrap()
}

pub(crate) fn forecast_for(observation: &Observation) -> crate::cpu::forecast::HazardForecast {
    forecast(
        observation,
        &ObservedHazardMemory::new(key()),
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(65_536),
    )
    .unwrap()
}

pub(crate) fn with_bomb(observation: Observation, trigger: ObservedBombTrigger) -> Observation {
    let mut parts = observation.into_parts();
    parts.bombs = BombKnowledge::Visible(vec![ObservedBomb {
        id: EntityId(90),
        placed_at: crate::game::Tick(0),
        cell: CellId::new(5, 1),
        owner: EntityId(1),
        range: 3,
        trigger,
        blast: crate::game::BlastKind::Standard,
        unknown: UnknownBombFacts::ALL,
    }]);
    Observation::new(parts).unwrap()
}

pub(super) fn request(movement: MovementIntent, hold_ticks: u8) -> PulseRequest {
    PulseRequest {
        movement,
        hold_ticks,
        start_window: TickWindow {
            start: Tick(1),
            end: Tick(2),
        },
        stopped_safe_until: Tick(23),
        mode: RouteMode::CertifiedOnly,
        settled_stop_tick: Tick(0),
    }
}

pub(super) fn actual_state(
    position: Position,
    heading: Direction,
    speed: SpeedLevel,
    settings: RoomSettings,
) -> State {
    actual_state_with_pickup(position, heading, speed, settings, None)
}

pub(super) fn actual_state_with_pickup(
    position: Position,
    heading: Direction,
    speed: SpeedLevel,
    settings: RoomSettings,
    pickup: Option<crate::game::PickupKind>,
) -> State {
    let mut arena_terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    arena_terrain.copy_from_slice(&terrain());
    let arena = Arena::new(
        arena_terrain,
        BTreeSet::new(),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap();
    let actors = [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: Position::center(CellId::new(13, 11)),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        },
        ActorInit {
            id: EntityId(2),
            seat: SeatId::Two,
            position,
            heading,
            speed,
        },
    ];
    let exposed = pickup
        .map(|kind| std::collections::BTreeMap::from([(position.cell().unwrap(), kind)]))
        .unwrap_or_default();
    let world = World::with_pickups(arena, BTreeMap::new(), BTreeMap::new(), exposed).unwrap();
    State::new(RulesConfig::expanded_v5(settings), world, actors).unwrap()
}

#[test]
fn certified_pulse_pose_matches_actual_state_updates() {
    let observed = observation(Vec::new(), Vec::new());
    let hazards = forecast_for(&observed);
    let lease = certify_pulse(
        &observed,
        &hazards,
        request(MovementIntent::Direction(Direction::East), 12),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();

    let mut state = actual_state(
        Position::center(CellId::new(2, 1)),
        Direction::East,
        SpeedLevel::Base,
        RoomSettings::DEFAULT,
    );
    for completed in 1..=12 {
        state
            .step(&[SeatInput {
                seat: SeatId::Two,
                action: InputAction::Press(Direction::East),
            }])
            .unwrap();
        let actual = &state.actors()[SeatId::Two.index()];
        assert!(
            lease
                .pose_after(completed)
                .contains(actual.position(), actual.heading())
        );
    }
    assert_eq!(lease.pose_after(12).min(), lease.pose_after(12).max());
    assert_eq!(lease.safe_until(), Tick(23));
}

#[test]
fn fastest_narrow_turn_pose_matches_actual_state() {
    let mut parts = observation(Vec::new(), Vec::new()).into_parts();
    let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
        unreachable!()
    };
    cpu.position.y += 128;
    cpu.heading = Direction::North;
    cpu.speed = SpeedLevel::Four;
    let position = cpu.position;
    let observed = Observation::new(parts).unwrap();
    let hazards = forecast_for(&observed);
    let lease = certify_pulse(
        &observed,
        &hazards,
        request(MovementIntent::Direction(Direction::East), 3),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    let mut state = actual_state(
        position,
        Direction::North,
        SpeedLevel::Four,
        RoomSettings::DEFAULT,
    );
    for completed in 1..=3 {
        state
            .step(&[SeatInput {
                seat: SeatId::Two,
                action: InputAction::Press(Direction::East),
            }])
            .unwrap();
        let actual = &state.actors()[SeatId::Two.index()];
        assert!(
            lease
                .pose_after(completed)
                .contains(actual.position(), actual.heading())
        );
    }
}

#[test]
fn waiting_is_certifiable_while_immediate_long_movement_crosses_fire() {
    let observed = observation(
        vec![ObservedFlame {
            cell: CellId::new(3, 1),
            remaining_ticks: 5,
        }],
        Vec::new(),
    );
    let hazards = forecast_for(&observed);
    assert_eq!(
        certify_pulse(
            &observed,
            &hazards,
            request(MovementIntent::Direction(Direction::East), 12),
            &mut WorkBudget::new(65_536)
        ),
        Err(RouteFailure::NoRoute)
    );
    let waiting = certify_pulse(
        &observed,
        &hazards,
        request(MovementIntent::Stop, 1),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    assert_eq!(waiting.movement(), MovementIntent::Stop);
    assert!(matches!(waiting.assurance(), RouteAssurance::Certified));
}

#[test]
fn pulse_rejects_pickup_sweeps_bad_identity_and_insufficient_work() {
    let observed = observation(
        Vec::new(),
        vec![ObservedPickup {
            cell: CellId::new(3, 1),
            kind: crate::game::PickupKind::MovementSpeed,
        }],
    );
    let hazards = forecast_for(&observed);
    assert_eq!(
        certify_pulse(
            &observed,
            &hazards,
            request(MovementIntent::Direction(Direction::East), 12),
            &mut WorkBudget::new(65_536)
        ),
        Err(RouteFailure::Unsupported)
    );
    assert_eq!(
        certify_pulse(
            &observed,
            &hazards,
            request(MovementIntent::Stop, 1),
            &mut WorkBudget::new(0)
        ),
        Err(RouteFailure::WorkExhausted)
    );

    let mut wrong = observation(Vec::new(), Vec::new()).into_parts();
    wrong.world.physical_round += 1;
    let wrong = Observation::new(wrong).unwrap();
    assert_eq!(
        certify_pulse(
            &wrong,
            &hazards,
            request(MovementIntent::Stop, 1),
            &mut WorkBudget::new(65_536)
        ),
        Err(RouteFailure::IdentityMismatch)
    );
}

#[test]
fn conditional_lease_carries_motion_id_but_never_waives_trigger_uncertainty() {
    let timed = with_bomb(
        observation(Vec::new(), Vec::new()),
        ObservedBombTrigger::Timed {
            remaining_ticks: 100,
            paused: false,
        },
    );
    let timed_forecast = forecast_for(&timed);
    let mut conditional = request(MovementIntent::Stop, 1);
    conditional.mode = RouteMode::AllowStationaryAssumptions;
    let lease = certify_pulse(
        &timed,
        &timed_forecast,
        conditional,
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    assert_eq!(
        lease.assurance(),
        &RouteAssurance::ConditionalStationary {
            bombs: vec![EntityId(90)]
        }
    );

    let remote = with_bomb(
        observation(Vec::new(), Vec::new()),
        ObservedBombTrigger::Remote,
    );
    let remote_forecast = forecast_for(&remote);
    assert_eq!(
        certify_pulse(
            &remote,
            &remote_forecast,
            conditional,
            &mut WorkBudget::new(65_536),
        ),
        Err(RouteFailure::NoRoute)
    );
}
