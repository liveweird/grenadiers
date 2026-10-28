use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::*;
use crate::game::{
    CellId, Direction, EffectKind, EnabledItems, InputAction, Position, SeatId, SeatInput,
    SpeedLevel, Terrain, Tick,
};
use crate::room::RoomSettings;

use super::tests::{actual_state, forecast_for, key, observation, request, with_bomb};
use super::*;

fn small_observation() -> Observation {
    let mut parts = observation(Vec::new(), Vec::new()).into_parts();
    parts.rules.version = 6;
    parts.arena.width = 11;
    parts.arena.height = 9;
    parts.arena.terrain = vec![Terrain::Floor; 99];
    for row in 0..9 {
        for column in 0..11 {
            if row == 0 || column == 0 || row == 8 || column == 10 {
                parts.arena.terrain[row * 11 + column] = Terrain::Wall;
            }
        }
    }
    let ObservedActor::Visible(rival) = &mut parts.actors[0] else {
        unreachable!()
    };
    rival.position = Position::center(CellId::new(9, 7));
    Observation::new(parts).unwrap()
}

#[test]
fn small_board_route_uses_its_bounds_and_rejects_cross_dimension_forecast() {
    let large = observation(Vec::new(), Vec::new());
    let small = small_observation();
    let small_hazards = forecast_for(&small);
    let pulse = request(MovementIntent::Direction(Direction::East), 3);
    assert!(certify_pulse(&small, &small_hazards, pulse, &mut WorkBudget::new(65_536),).is_ok());

    let large_hazards = forecast_for(&large);
    assert_eq!(
        certify_pulse(&small, &large_hazards, pulse, &mut WorkBudget::new(65_536),),
        Err(RouteFailure::IdentityMismatch)
    );
}

#[test]
fn multistart_slow_and_reverse_expiry_matches_actual_state() {
    let enabled = EnabledItems::from_bits(
        EnabledItems::SLOW_FOOT.bits() | EnabledItems::REVERSE_CONTROLS.bits(),
    )
    .unwrap();
    let settings = RoomSettings::DEFAULT.with_enabled_items(enabled);
    let mut parts = observation(Vec::new(), Vec::new()).into_parts();
    parts.tick = Tick(598);
    parts.rules.running_updates = 598;
    parts.rules.settings = settings;
    let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
        unreachable!()
    };
    cpu.speed = SpeedLevel::Four;
    cpu.effects = vec![
        ObservedEffect {
            kind: EffectKind::SlowFoot,
            remaining_ticks: 2,
        },
        ObservedEffect {
            kind: EffectKind::ReverseControls,
            remaining_ticks: 2,
        },
    ];
    let position = cpu.position;
    let observed = Observation::new(parts).unwrap();
    let hazards = forecast_for(&observed);
    let mut pulse = request(MovementIntent::Direction(Direction::East), 3);
    pulse.start_window = TickWindow {
        start: Tick(599),
        end: Tick(601),
    };
    pulse.stopped_safe_until = Tick(613);
    pulse.settled_stop_tick = Tick(598);
    let lease = certify_pulse(&observed, &hazards, pulse, &mut WorkBudget::new(65_536)).unwrap();

    let mut basis = actual_state(position, Direction::East, SpeedLevel::Four, settings);
    basis.apply_test_effect(SeatId::Two, EffectKind::SlowFoot, Tick(0));
    basis.apply_test_effect(SeatId::Two, EffectKind::ReverseControls, Tick(0));
    for _ in 0..598 {
        basis.step(&[]).unwrap();
    }
    for start in [599_u64, 600] {
        let mut state = basis.clone();
        for _ in 599..start {
            state.step(&[]).unwrap();
        }
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
}

#[test]
fn stopped_hazards_respect_stop_and_exclusive_lease_boundaries() {
    for (detonation, expected) in [(2, false), (19, false), (20, true)] {
        let observed = with_bomb(
            observation(Vec::new(), Vec::new()),
            ObservedBombTrigger::Timed {
                remaining_ticks: detonation,
                paused: false,
            },
        );
        let hazards = forecast_for(&observed);
        let mut pulse = request(MovementIntent::Stop, 1);
        pulse.mode = RouteMode::AllowStationaryAssumptions;
        pulse.stopped_safe_until = Tick(20);
        let result = certify_pulse(&observed, &hazards, pulse, &mut WorkBudget::new(65_536));
        assert_eq!(result.is_ok(), expected, "detonation {detonation}");
    }
}

#[test]
fn conditional_mode_rejects_pause_and_sticky_displaced_motion() {
    let paused = with_bomb(
        observation(Vec::new(), Vec::new()),
        ObservedBombTrigger::Timed {
            remaining_ticks: 2,
            paused: true,
        },
    );
    let paused_forecast = forecast_for(&paused);
    let mut conditional = request(MovementIntent::Stop, 1);
    conditional.mode = RouteMode::AllowStationaryAssumptions;
    assert_eq!(
        certify_pulse(
            &paused,
            &paused_forecast,
            conditional,
            &mut WorkBudget::new(65_536),
        ),
        Err(RouteFailure::NoRoute)
    );

    let first = with_bomb(
        observation(Vec::new(), Vec::new()),
        ObservedBombTrigger::Timed {
            remaining_ticks: 100,
            paused: false,
        },
    );
    let mut memory = ObservedHazardMemory::new(key());
    memory.observe(&first).unwrap();
    let mut second_parts = first.into_parts();
    second_parts.tick = Tick(1);
    second_parts.rules.running_updates = 1;
    let BombKnowledge::Visible(bombs) = &mut second_parts.bombs else {
        unreachable!()
    };
    bombs[0].cell.column += 1;
    bombs[0].trigger = ObservedBombTrigger::Timed {
        remaining_ticks: 99,
        paused: false,
    };
    let second = Observation::new(second_parts).unwrap();
    memory.observe(&second).unwrap();
    let displaced = forecast(
        &second,
        &memory,
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    conditional.start_window = TickWindow {
        start: Tick(2),
        end: Tick(3),
    };
    conditional.stopped_safe_until = Tick(13);
    conditional.settled_stop_tick = Tick(1);
    assert_eq!(
        certify_pulse(
            &second,
            &displaced,
            conditional,
            &mut WorkBudget::new(65_536),
        ),
        Err(RouteFailure::NoRoute)
    );
}

#[test]
fn partial_scenario_work_exhaustion_returns_no_lease() {
    let observed = observation(Vec::new(), Vec::new());
    let hazards = forecast_for(&observed);
    let mut pulse = request(MovementIntent::Direction(Direction::East), 12);
    pulse.start_window = TickWindow {
        start: Tick(1),
        end: Tick(8),
    };
    pulse.stopped_safe_until = Tick(29);
    let mut work = WorkBudget::new(5);
    let result = certify_pulse(&observed, &hazards, pulse, &mut work);
    assert_eq!(result, Err(RouteFailure::WorkExhausted));
    assert!(work.spent() > 0);
    assert_eq!(work.remaining(), 0);
}
