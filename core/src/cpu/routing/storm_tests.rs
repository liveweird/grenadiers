use std::collections::{BTreeMap, BTreeSet};

use crate::cpu::forecast::TickWindow;
use crate::cpu::{
    MovementIntent, Observation, ObservedActor, ObservedEffect, ObservedEnvironment, WorkBudget,
};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    ActorInit, CellId, Direction, EntityId, InputAction, Position, RoundAssignment, RulesConfig,
    SeatId, SeatInput, SimulationSeed, SpeedLevel, State, StormCloudConfig, Tick, World,
};

use super::certify_pulse;
use super::motion::{Motion, advance};
use super::tests::{forecast_for, observation, request};
use super::validation::validate;

fn environment_observation(config: StormCloudConfig) -> Observation {
    let mut parts = observation(Vec::new(), Vec::new()).into_parts();
    parts.rules.version = 7;
    let cloud = (config == StormCloudConfig::Single).then_some(crate::game::StormCloudView {
        position: Position::center(CellId::new(7, 6)),
        direction: Direction::East,
        leg_remaining_ticks: 128,
    });
    Observation::new_rules_v7(
        parts,
        ObservedEnvironment::new(config, cloud, vec![Some(0), Some(0)]),
    )
    .unwrap()
}

pub(crate) fn custom_environment_observation(
    cloud: crate::game::StormCloudView,
    actor_position: Position,
    remaining: u16,
) -> Observation {
    let mut parts = observation(Vec::new(), Vec::new()).into_parts();
    parts.rules.version = 7;
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.position = actor_position;
    Observation::new_rules_v7(
        parts,
        ObservedEnvironment::new(
            StormCloudConfig::Single,
            Some(cloud),
            vec![Some(0), Some(remaining)],
        ),
    )
    .unwrap()
}

#[test]
fn rules_seven_requires_valid_overlay_and_map_parts_preserves_it() {
    let observed = environment_observation(StormCloudConfig::Disabled);
    assert!(Observation::new(observed.clone().into_parts()).is_err());

    let mapped = observed
        .map_parts(|parts| parts.rules.remaining_ticks -= 1)
        .unwrap();
    assert_eq!(
        mapped.environment().unwrap().storm_cloud_config(),
        StormCloudConfig::Disabled
    );

    let mut parts = mapped.into_parts();
    parts.rules.remaining_ticks -= 1;
    assert!(
        Observation::new_rules_v7(
            parts,
            ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(1), Some(0)]),
        )
        .is_err()
    );
}

#[test]
fn disabled_environment_keeps_legacy_motion_and_work_accounting() {
    let mut legacy_parts = observation(Vec::new(), Vec::new()).into_parts();
    legacy_parts.rules.version = 6;
    let legacy = Observation::new(legacy_parts).unwrap();
    let disabled = environment_observation(StormCloudConfig::Disabled);
    let mut legacy_work = WorkBudget::new(65_536);
    let mut disabled_work = WorkBudget::new(65_536);
    let legacy_lease = certify_pulse(
        &legacy,
        &forecast_for(&legacy),
        request(MovementIntent::Direction(Direction::East), 12),
        &mut legacy_work,
    )
    .unwrap();
    let disabled_lease = certify_pulse(
        &disabled,
        &forecast_for(&disabled),
        request(MovementIntent::Direction(Direction::East), 12),
        &mut disabled_work,
    )
    .unwrap();
    assert_eq!(legacy_work.spent(), disabled_work.spent());
    assert_eq!(legacy_work.spent(), 24);
    assert_eq!(legacy_lease.pose_after(12), disabled_lease.pose_after(12));
}

#[test]
fn current_overlap_refresh_and_speed_match_the_domain_oracle() {
    let center = Position::center(CellId::new(ARENA_WIDTH / 2, ARENA_HEIGHT / 2));
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if row == 0 || column == 0 || row + 1 == ARENA_HEIGHT || column + 1 == ARENA_WIDTH {
                terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    let arena = Arena::new(
        terrain,
        BTreeSet::new(),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap();
    let actors = [
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
            position: center,
            heading: Direction::East,
            speed: SpeedLevel::Base,
        },
    ];
    let world =
        World::with_pickups(arena, BTreeMap::new(), BTreeMap::new(), BTreeMap::new()).unwrap();
    let rules =
        RulesConfig::environment_v7(crate::room::RoomSettings::DEFAULT, StormCloudConfig::Single);
    let mut state = State::new_at_tick_with_assignment_and_seed(
        rules,
        world,
        actors,
        Tick(0),
        RoundAssignment::first(),
        SimulationSeed::new([9; 32]),
    )
    .unwrap();

    let mut parts = observation(Vec::new(), Vec::new()).into_parts();
    parts.rules.version = 7;
    let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
        unreachable!()
    };
    cpu.position = center;
    let observed = Observation::new_rules_v7(
        parts,
        ObservedEnvironment::new(
            StormCloudConfig::Single,
            state.storm_cloud(),
            vec![Some(0), Some(0)],
        ),
    )
    .unwrap();
    let forecast = forecast_for(&observed);
    let context = validate(
        &observed,
        &forecast,
        TickWindow {
            start: Tick(1),
            end: Tick(2),
        },
        Tick(20),
    )
    .unwrap();
    let mut motion = Motion::stopped(&context, Tick(1));
    for tick in 1..=12 {
        advance(
            &context,
            &mut motion,
            MovementIntent::Direction(Direction::East),
            Tick(tick),
        )
        .unwrap();
        state
            .step(&[SeatInput {
                seat: SeatId::Two,
                action: InputAction::Press(Direction::East),
            }])
            .unwrap();
        assert_eq!(
            motion.position,
            state.actors()[SeatId::Two.index()].position(),
            "tick {tick}"
        );
        if tick <= 3 {
            assert_eq!(
                motion.position.x - center.x,
                [51, 76, 102][usize::try_from(tick - 1).unwrap()],
                "fractional speed residual at tick {tick}"
            );
        }
    }
    assert_eq!(
        motion.storm_deadlines().unwrap().lower,
        state.actors()[SeatId::Two.index()]
            .storm_slow_until()
            .unwrap()
    );
}

#[test]
fn known_expiry_and_unknown_contact_are_certified_without_guessing_speed() {
    let far_cloud = crate::game::StormCloudView {
        position: Position::center(CellId::new(7, 6)),
        direction: Direction::East,
        leg_remaining_ticks: 128,
    };
    let actor_position = Position::center(CellId::new(2, 1));
    let expiry = custom_environment_observation(far_cloud, actor_position, 1);
    let forecast = forecast_for(&expiry);
    let context = validate(
        &expiry,
        &forecast,
        TickWindow {
            start: Tick(1),
            end: Tick(2),
        },
        Tick(10),
    )
    .unwrap();
    let mut motion = Motion::stopped(&context, Tick(1));
    advance(
        &context,
        &mut motion,
        MovementIntent::Direction(Direction::East),
        Tick(1),
    )
    .unwrap();
    assert_eq!(motion.position.x - actor_position.x, 51);

    let uncertain_cloud = crate::game::StormCloudView {
        position: actor_position,
        direction: Direction::North,
        leg_remaining_ticks: 0,
    };
    let uncertain = custom_environment_observation(uncertain_cloud, actor_position, 2);
    let forecast = forecast_for(&uncertain);
    let context = validate(
        &uncertain,
        &forecast,
        TickWindow {
            start: Tick(1),
            end: Tick(2),
        },
        Tick(10),
    )
    .unwrap();
    let mut motion = Motion::stopped(&context, Tick(1));
    advance(
        &context,
        &mut motion,
        MovementIntent::Direction(Direction::East),
        Tick(1),
    )
    .unwrap();
    assert!(
        advance(
            &context,
            &mut motion,
            MovementIntent::Direction(Direction::East),
            Tick(2),
        )
        .is_err()
    );
}

#[test]
fn storm_slow_is_independent_of_disabled_slow_foot_item() {
    let cloud = crate::game::StormCloudView {
        position: Position::center(CellId::new(7, 6)),
        direction: Direction::East,
        leg_remaining_ticks: 128,
    };
    let actor_position = Position::center(CellId::new(2, 1));
    let basis = custom_environment_observation(cloud, actor_position, 20);
    let environment = basis.environment().unwrap().clone();
    let mut parts = basis.into_parts();
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.effects.push(ObservedEffect {
        kind: crate::game::EffectKind::SlowFoot,
        remaining_ticks: 20,
    });
    let observed = Observation::new_rules_v7(parts, environment).unwrap();
    let forecast = forecast_for(&observed);
    let context = validate(
        &observed,
        &forecast,
        TickWindow {
            start: Tick(1),
            end: Tick(2),
        },
        Tick(10),
    )
    .unwrap();
    let mut motion = Motion::stopped(&context, Tick(1));
    advance(
        &context,
        &mut motion,
        MovementIntent::Direction(Direction::East),
        Tick(1),
    )
    .unwrap();
    assert_eq!(motion.position.x - actor_position.x, 25);
}

#[test]
fn unknown_cloud_certifies_safe_separation_and_guaranteed_slow_only() {
    let actor_position = Position::center(CellId::new(2, 1));
    let separated = custom_environment_observation(
        crate::game::StormCloudView {
            position: Position::center(CellId::new(13, 11)),
            direction: Direction::West,
            leg_remaining_ticks: 0,
        },
        actor_position,
        0,
    );
    assert!(
        certify_pulse(
            &separated,
            &forecast_for(&separated),
            request(MovementIntent::Direction(Direction::East), 12),
            &mut WorkBudget::new(65_536),
        )
        .is_ok()
    );

    let uncertain_cloud = crate::game::StormCloudView {
        position: actor_position,
        direction: Direction::North,
        leg_remaining_ticks: 0,
    };
    let guaranteed = custom_environment_observation(uncertain_cloud, actor_position, 180);
    assert!(
        certify_pulse(
            &guaranteed,
            &forecast_for(&guaranteed),
            request(MovementIntent::Direction(Direction::East), 12),
            &mut WorkBudget::new(65_536),
        )
        .is_ok()
    );
    let uncertain = custom_environment_observation(uncertain_cloud, actor_position, 2);
    assert!(
        certify_pulse(
            &uncertain,
            &forecast_for(&uncertain),
            request(MovementIntent::Direction(Direction::East), 12),
            &mut WorkBudget::new(65_536),
        )
        .is_err()
    );
}
