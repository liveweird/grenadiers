use super::*;
use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, forecast};
use crate::cpu::routing::RouteMode;
use crate::cpu::routing::tests;
use crate::cpu::{BombKnowledge, ObservedBomb, ObservedBombTrigger, UnknownBombFacts, WorkBudget};
use crate::game::BlastKind;
use crate::game::{MovementSegment, Position};

#[test]
fn origin_grace_is_consumed_on_exit_and_cannot_be_reentered() {
    let origin = PulseOriginGrace {
        cell: CellId::new(2, 1),
        bomb_id: EntityId(7),
    };
    let inside = Position::center(origin.cell);
    let outside = Position::center(CellId::new(3, 1));
    let mut active = true;
    update_origin(
        Some(origin),
        &[MovementSegment::new(inside, outside)],
        &mut active,
    )
    .unwrap();
    assert!(!active);
    assert_eq!(
        update_origin(
            Some(origin),
            &[MovementSegment::new(outside, inside)],
            &mut active,
        ),
        Err(RouteFailure::NoRoute)
    );
}

#[test]
fn origin_precheck_allows_only_named_stationary_foreign_motion_in_conditional_mode() {
    let origin = PulseOriginGrace {
        cell: CellId::new(2, 1),
        bomb_id: EntityId(70),
    };
    let observed = origin_observation(
        CellId::new(3, 1),
        ObservedBombTrigger::Timed {
            remaining_ticks: 180,
            paused: false,
        },
    );
    let forecast = origin_forecast(&observed, &ObservedHazardMemory::new(tests::key()));
    assert!(forecast.stationary_assumptions().contains(&EntityId(71)));

    assert_eq!(
        validate_origin_forecast(&forecast, &observed, origin, RouteMode::CertifiedOnly),
        Err(RouteFailure::NoRoute)
    );
    assert_eq!(
        validate_origin_forecast(
            &forecast,
            &observed,
            origin,
            RouteMode::AllowStationaryAssumptions,
        ),
        Ok(())
    );
}

#[test]
fn origin_precheck_refuses_displaced_foreign_motion_without_an_assumption() {
    let origin = PulseOriginGrace {
        cell: CellId::new(2, 1),
        bomb_id: EntityId(70),
    };
    let mut memory = ObservedHazardMemory::new(tests::key());
    let previous = origin_observation(
        CellId::new(4, 1),
        ObservedBombTrigger::Timed {
            remaining_ticks: 181,
            paused: false,
        },
    );
    memory.observe(&previous).unwrap();
    let mut parts = origin_observation(
        CellId::new(3, 1),
        ObservedBombTrigger::Timed {
            remaining_ticks: 180,
            paused: false,
        },
    )
    .into_parts();
    parts.tick = Tick(1);
    parts.rules.running_updates = 1;
    let observed = crate::cpu::Observation::new(parts).unwrap();
    memory.observe(&observed).unwrap();
    let forecast = origin_forecast(&observed, &memory);
    assert!(!forecast.stationary_assumptions().contains(&EntityId(71)));
    assert_eq!(
        validate_origin_forecast(
            &forecast,
            &observed,
            origin,
            RouteMode::AllowStationaryAssumptions,
        ),
        Err(RouteFailure::NoRoute)
    );
}

#[test]
fn origin_precheck_refuses_paused_and_remote_foreign_bombs() {
    let origin = PulseOriginGrace {
        cell: CellId::new(2, 1),
        bomb_id: EntityId(70),
    };
    for trigger in [
        ObservedBombTrigger::Timed {
            remaining_ticks: 180,
            paused: true,
        },
        ObservedBombTrigger::Remote,
    ] {
        let observed = origin_observation(CellId::new(3, 1), trigger);
        let forecast = origin_forecast(&observed, &ObservedHazardMemory::new(tests::key()));
        assert_eq!(
            validate_origin_forecast(
                &forecast,
                &observed,
                origin,
                RouteMode::AllowStationaryAssumptions,
            ),
            Err(RouteFailure::NoRoute)
        );
    }
}

fn origin_observation(
    foreign_cell: CellId,
    foreign_trigger: ObservedBombTrigger,
) -> crate::cpu::Observation {
    let mut parts = tests::observation(Vec::new(), Vec::new()).into_parts();
    let timed = |id, cell, trigger| ObservedBomb {
        id: EntityId(id),
        placed_at: Tick(0),
        cell,
        owner: EntityId(2),
        range: 2,
        trigger,
        blast: BlastKind::Standard,
        unknown: UnknownBombFacts::ALL,
    };
    parts.bombs = BombKnowledge::Visible(vec![
        timed(
            70,
            CellId::new(2, 1),
            ObservedBombTrigger::Timed {
                remaining_ticks: 180,
                paused: false,
            },
        ),
        timed(71, foreign_cell, foreign_trigger),
    ]);
    crate::cpu::Observation::new(parts).unwrap()
}

fn origin_forecast(
    observed: &crate::cpu::Observation,
    memory: &ObservedHazardMemory,
) -> crate::cpu::forecast::HazardForecast {
    forecast(
        observed,
        memory,
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(65_536),
    )
    .unwrap()
}
