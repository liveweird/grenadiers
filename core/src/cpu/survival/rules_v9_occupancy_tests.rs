use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::policy::{PolicyError, UtilityMemory};
use crate::cpu::{
    CpuRng, MemoryKey, Observation, ObservedActor, ObservedEnvironment, ObservedRespawnLairs,
    WorkBudget,
};
use crate::game::{
    CellId, EntityId, LairStatusView, Position, SeatId, StormCloudConfig, Terrain, lair_layout_v1,
};
use crate::room::{BoardSize, RespawnLairConfig, RespawnLairSettings};

use super::compatibility::{can_admit_ordinary, visible_rivals_allow_segment};

fn observed(cpu: CellId, rival: Option<CellId>) -> Observation {
    let mut parts = crate::cpu::policy::tests::observation(cpu).into_parts();
    parts.rules.version = 9;
    let mut timers = vec![None, Some(0)];
    if let Some(cell) = rival {
        let mut actor = match parts.actors[1].clone() {
            ObservedActor::Visible(actor) => actor,
            ObservedActor::Hidden { .. } => unreachable!(),
        };
        actor.id = EntityId(1);
        actor.seat = SeatId::One;
        actor.position = Position::center(cell);
        parts.actors[0] = ObservedActor::Visible(actor);
        timers[0] = Some(0);
    }
    Observation::new_rules_v9(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, timers),
        None,
    )
    .unwrap()
}

fn with_enabled_lairs(observation: Observation) -> Observation {
    let environment = observation.environment().unwrap().clone();
    let mut parts = observation.into_parts();
    let cells = lair_layout_v1(BoardSize::FifteenByThirteen);
    for cell in cells {
        let index = parts.arena.index(cell).unwrap();
        parts.arena.terrain[index] = Terrain::Wall;
    }
    let lairs = cells
        .into_iter()
        .map(|cell| LairStatusView {
            cell,
            next_spawn_at: Some(parts.tick),
            living_mobs: 0,
        })
        .collect();
    Observation::new_rules_v9(
        parts,
        environment,
        Some(ObservedRespawnLairs::new(
            RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap()),
            lairs,
            Vec::new(),
        )),
    )
    .unwrap()
}

#[test]
fn v9_segment_check_uses_visible_living_footprints_and_not_hidden_positions() {
    let blocked = observed(CellId::new(7, 7), Some(CellId::new(9, 7)));
    assert!(!visible_rivals_allow_segment(
        &blocked,
        Position::center(CellId::new(7, 7)),
        Position::center(CellId::new(9, 7)),
    ));

    let hidden = observed(CellId::new(7, 7), None);
    assert!(visible_rivals_allow_segment(
        &hidden,
        Position::center(CellId::new(7, 7)),
        Position::center(CellId::new(9, 7)),
    ));
}

#[test]
fn unrelated_visible_rival_motion_does_not_starve_ordinary_admission() {
    let basis = with_enabled_lairs(observed(CellId::new(7, 7), Some(CellId::new(11, 7))));
    let current = with_enabled_lairs(observed(CellId::new(7, 7), Some(CellId::new(11, 8))));
    assert!(can_admit_ordinary(&basis, &current));
}

#[test]
fn v9_has_a_distinct_planner_entrypoint_and_keeps_legacy_policy_closed() {
    let observed = observed(CellId::new(7, 7), Some(CellId::new(11, 7)));
    let key = MemoryKey {
        world: observed.parts().world,
        cpu_seat: observed.parts().recipient,
    };
    let memory = UtilityMemory::new(key);
    assert_eq!(
        crate::cpu::policy::plan(
            &observed,
            &memory,
            &mut CpuRng::from_seed(key, [71; 32]),
            &mut WorkBudget::new(65_536),
        ),
        Err(PolicyError::InvalidObservation)
    );
    assert!(
        super::plan_respawn(
            &observed,
            &memory,
            &ObservedHazardMemory::new(key),
            observed.parts().tick,
            &mut CpuRng::from_seed(key, [72; 32]),
            &mut WorkBudget::new(65_536),
        )
        .is_ok()
    );
}

#[test]
fn v9_firing_position_plants_only_with_a_complete_escape_certificate() {
    let observed = observed(CellId::new(9, 7), Some(CellId::new(11, 7)));
    let key = MemoryKey {
        world: observed.parts().world,
        cpu_seat: observed.parts().recipient,
    };
    let planned = super::plan_respawn(
        &observed,
        &UtilityMemory::new(key),
        &ObservedHazardMemory::new(key),
        observed.parts().tick,
        &mut CpuRng::from_seed(key, [73; 32]),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    assert_eq!(
        planned.survival.control.intent.manual,
        Some(crate::cpu::ManualIntent::Bomb)
    );
    assert!(planned.survival.execution.is_some());
}

#[test]
fn newly_blocked_next_segment_invalidates_a_committed_v9_pulse() {
    let basis = with_enabled_lairs(observed(CellId::new(7, 7), Some(CellId::new(11, 7))));
    let key = MemoryKey {
        world: basis.parts().world,
        cpu_seat: basis.parts().recipient,
    };
    let mut work = WorkBudget::new(65_536);
    let hazards = forecast(
        &basis,
        &ObservedHazardMemory::new(key),
        None,
        ForecastLimits::default(),
        &mut work,
    )
    .unwrap();
    let first = TickWindow {
        start: crate::game::Tick(basis.parts().tick.0 + 1),
        end: crate::game::Tick(basis.parts().tick.0 + 2),
    };
    let lease = crate::cpu::routing::certify_respawn_pulse(
        &basis,
        &hazards,
        crate::cpu::routing::PulseRequest {
            movement: crate::cpu::MovementIntent::Direction(crate::game::Direction::East),
            hold_ticks: 12,
            start_window: first,
            stopped_safe_until: crate::game::Tick(basis.parts().tick.0 + 30),
            mode: crate::cpu::routing::RouteMode::CertifiedOnly,
            settled_stop_tick: basis.parts().tick,
        },
        &mut work,
    )
    .unwrap();
    let committed =
        super::CommittedPulse::admit(lease, basis.clone(), &basis, basis.parts().tick).unwrap();
    let environment = basis.environment().unwrap().clone();
    let lairs = basis.respawn_lairs().cloned();
    let mut blocked_parts = basis.clone().into_parts();
    let ObservedActor::Visible(rival) = &mut blocked_parts.actors[0] else {
        unreachable!()
    };
    rival.position = Position {
        x: Position::center(CellId::new(7, 7)).x + 650,
        y: Position::center(CellId::new(7, 7)).y,
    };
    let blocked = Observation::new_rules_v9(blocked_parts, environment, lairs).unwrap();
    assert_eq!(
        committed.validate(&blocked, basis.parts().tick),
        super::PulseStatus::Invalidated
    );
}
