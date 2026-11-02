use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::routing::{
    PulseRequest, RouteLimits, RouteMode, RouteRequest, certify_respawn_pulse, plan_respawn_route,
};
use crate::cpu::{
    BombKnowledge, Observation, ObservedActor, ObservedArena, ObservedEnvironment, ObservedRules,
    ObservedSpikePits, VisibleActor, WorkBudget,
};
use crate::game::{
    CellId, Direction, EntityId, Position, SeatId, SpeedLevel, SpikePitPhase, SpikePitView,
    StormCloudConfig, Tick, fixed_arena, spike_pit_layout_v1,
};
use crate::room::{RoomSettings, SpikePitConfig, SpikePitSettings};

use super::{CommittedPulse, CommittedRoute, PulseStatus, RouteStatus};

pub(super) fn observed(tick: u64, phase: SpikePitPhase, remaining_ticks: u64) -> Observation {
    observed_with_settings(
        tick,
        phase,
        remaining_ticks,
        SpikePitSettings::new(6, 60).unwrap(),
    )
}

fn observed_with_settings(
    tick: u64,
    phase: SpikePitPhase,
    remaining_ticks: u64,
    settings: SpikePitSettings,
) -> Observation {
    let arena = fixed_arena();
    let pit_cells = spike_pit_layout_v1(&arena, []);
    let target_pit = pit_cells[0];
    let arena_ref = &arena;
    let terrain = (0..arena.height())
        .flat_map(|row| {
            (0..arena_ref.width())
                .map(move |column| arena_ref.terrain(CellId::new(column, row)).unwrap())
        })
        .collect();
    let parts = crate::cpu::ObservationParts {
        world: crate::cpu::WorldId {
            room_instance: 83,
            physical_round: 2,
        },
        tick: Tick(tick),
        recipient: SeatId::Two,
        rules: ObservedRules {
            version: 10,
            punch_activation_policy: crate::game::PunchActivationPolicy::ManualV1,
            tick_rate: 60,
            running_updates: u16::try_from(tick).unwrap(),
            remaining_ticks: u16::try_from(1_000 - tick).unwrap(),
            settings: RoomSettings::DEFAULT,
        },
        arena: ObservedArena {
            width: arena.width(),
            height: arena.height(),
            terrain,
            blocks: arena.blocks().iter().copied().collect(),
        },
        actors: vec![
            actor(SeatId::One, 1, CellId::new(13, 11)),
            actor(SeatId::Two, 2, CellId::new(1, 1)),
        ],
        bombs: BombKnowledge::Visible(Vec::new()),
        flames: Vec::new(),
        oil: Vec::new(),
        pickups: Vec::new(),
    };
    Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0); 2]),
        None,
        Some(ObservedSpikePits::new(
            SpikePitConfig::Enabled(settings),
            pit_cells
                .into_iter()
                .map(|cell| SpikePitView {
                    cell,
                    phase: if cell == target_pit {
                        phase
                    } else {
                        SpikePitPhase::Idle
                    },
                    remaining_ticks: if cell == target_pit {
                        remaining_ticks
                    } else {
                        0
                    },
                })
                .collect(),
        )),
    )
    .unwrap()
}

fn committed_route(
    basis: &Observation,
    admission_tick: Tick,
    current: &Observation,
) -> Result<CommittedRoute, crate::cpu::policy::PolicyError> {
    let key = crate::cpu::MemoryKey {
        world: basis.parts().world,
        cpu_seat: basis.parts().recipient,
    };
    let hazards = forecast(
        basis,
        &ObservedHazardMemory::new(key),
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    let route = plan_respawn_route(
        basis,
        &hazards,
        RouteRequest {
            start_window: TickWindow {
                start: Tick(1),
                end: Tick(8),
            },
            settled_stop_tick: Tick(0),
            survive_until: Tick(20),
            mode: RouteMode::CertifiedOnly,
            limits: RouteLimits::default(),
        },
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    CommittedRoute::admit(
        route,
        basis.clone(),
        current,
        admission_tick,
        crate::cpu::MovementIntent::Stop,
    )
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
        unknown: crate::cpu::UnknownActorFacts::ALL,
    })
}

fn pulse_lease(basis: &Observation, start_window: TickWindow) -> crate::cpu::routing::PulseLease {
    let key = crate::cpu::MemoryKey {
        world: basis.parts().world,
        cpu_seat: basis.parts().recipient,
    };
    let forecast = forecast(
        basis,
        &ObservedHazardMemory::new(key),
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    certify_respawn_pulse(
        basis,
        &forecast,
        PulseRequest {
            movement: crate::cpu::MovementIntent::Stop,
            hold_ticks: 1,
            start_window,
            stopped_safe_until: Tick(23),
            mode: RouteMode::CertifiedOnly,
            settled_stop_tick: Tick(0),
        },
        &mut WorkBudget::new(65_536),
    )
    .unwrap()
}

fn committed(basis: &Observation) -> CommittedPulse {
    let lease = pulse_lease(
        basis,
        TickWindow {
            start: Tick(1),
            end: Tick(2),
        },
    );
    CommittedPulse::admit(lease, basis.clone(), basis, Tick(0)).unwrap()
}

#[test]
fn own_countdown_is_admitted_but_an_opponent_phase_change_invalidates_the_pulse() {
    let basis = observed(0, SpikePitPhase::Warning, 6);
    let committed = committed(&basis);
    assert_eq!(
        committed.validate(&observed(1, SpikePitPhase::Warning, 5), Tick(1)),
        PulseStatus::Wait
    );
    assert_eq!(
        committed.validate(&observed(1, SpikePitPhase::Raised, 60), Tick(1)),
        PulseStatus::Invalidated
    );
}

#[test]
fn an_expired_guard_never_authorizes_future_input() {
    let basis = observed(0, SpikePitPhase::Idle, 0);
    assert_eq!(
        committed(&basis).validate(&observed(23, SpikePitPhase::Idle, 0), Tick(23)),
        PulseStatus::Invalidated
    );
}

#[test]
fn stale_publication_before_admission_has_no_pit_guard_and_rejects_the_pulse() {
    let basis = observed(0, SpikePitPhase::Idle, 0);
    let lease = pulse_lease(
        &basis,
        TickWindow {
            start: Tick(1),
            end: Tick(3),
        },
    );
    let admission_tick = Tick(1);
    let first_execution = Tick(2);
    assert_eq!(basis.parts().tick, Tick(0));
    assert!(
        lease
            .spike_pit_guard_at(first_execution, basis.parts().tick)
            .is_none()
    );
    let mut work = WorkBudget::new(super::WORK_ALLOWANCE);
    assert_eq!(
        lease.spike_pits_match_at(first_execution, &basis, &mut work),
        Ok(false)
    );
    assert_eq!(work.spent(), 1);
    assert!(CommittedPulse::admit(lease, basis.clone(), &basis, admission_tick).is_err());
}

#[test]
fn cpu26_planner_is_closed_to_rules10_and_returns_guarded_execution() {
    let observation = observed(0, SpikePitPhase::Idle, 0);
    let key = crate::cpu::MemoryKey {
        world: observation.parts().world,
        cpu_seat: observation.parts().recipient,
    };
    let plan = super::plan_spike_pits_v26_with_origin(
        &observation,
        &crate::cpu::policy::UtilityMemory::new(key),
        &ObservedHazardMemory::new(key),
        Tick(0),
        None,
        &mut crate::cpu::CpuRng::from_seed(key, [7; 32]),
        &mut WorkBudget::new(super::WORK_ALLOWANCE),
    )
    .unwrap()
    .survival;

    assert!(plan.lease.is_some() || plan.execution.is_some());
    assert!(
        crate::cpu::policy::plan_respawn_v25(
            &observation,
            &crate::cpu::policy::UtilityMemory::new(key),
            &mut crate::cpu::CpuRng::from_seed(key, [7; 32]),
            &mut WorkBudget::new(super::WORK_ALLOWANCE),
        )
        .is_err()
    );
}

#[test]
fn committed_route_selects_the_actual_lane_and_rejects_phase_or_config_changes() {
    let basis = observed(0, SpikePitPhase::Warning, 6);
    let at_admission = observed(3, SpikePitPhase::Warning, 3);
    let mut route = committed_route(&basis, Tick(3), &at_admission).unwrap();
    assert_eq!(
        route.observe(&observed(4, SpikePitPhase::Warning, 2)),
        RouteStatus::Wait
    );

    let wrong_phase = observed(3, SpikePitPhase::Raised, 60);
    assert!(committed_route(&basis, Tick(3), &wrong_phase).is_err());
    let wrong_config = observed_with_settings(
        3,
        SpikePitPhase::Warning,
        3,
        SpikePitSettings::new(7, 60).unwrap(),
    );
    assert!(committed_route(&basis, Tick(3), &wrong_config).is_err());
}
