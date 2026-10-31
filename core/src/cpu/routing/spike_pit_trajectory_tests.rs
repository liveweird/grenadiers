use crate::cpu::{
    ObservedActor, ObservedArena, ObservedEffect, ObservedEnvironment, ObservedSpikePits,
};
use crate::game::{
    EffectKind, Equipment, MovementSegment, Position, SpikePitPhase, SpikePitView,
    StormCloudConfig, fixed_arena, spike_pit_layout_v1,
};
use crate::room::{SpikePitConfig, SpikePitSettings};

use super::*;

pub(crate) fn observed(
    phase: SpikePitPhase,
    remaining_ticks: u64,
    shield_ticks: Option<u16>,
    grace_ticks: u8,
    heart: bool,
) -> (Observation, CellId) {
    let arena = fixed_arena();
    let mut pit_cells = spike_pit_layout_v1(&arena, []);
    pit_cells.sort_unstable();
    let pit = pit_cells[0];
    let arena_ref = &arena;
    let mut parts = crate::cpu::routing::tests::observation(Vec::new(), Vec::new()).into_parts();
    parts.rules.version = 10;
    parts.arena = ObservedArena {
        width: arena.width(),
        height: arena.height(),
        terrain: (0..arena_ref.height())
            .flat_map(|row| {
                (0..arena_ref.width())
                    .map(move |column| arena_ref.terrain(CellId::new(column, row)).unwrap())
            })
            .collect(),
        blocks: arena.blocks().iter().copied().collect(),
    };
    let actor_count = parts.actors.len();
    let actor = parts
        .actors
        .iter_mut()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor) if actor.seat == parts.recipient => Some(actor),
            _ => None,
        })
        .unwrap();
    actor.position = Position::center(pit);
    actor.damage_grace_remaining_ticks = grace_ticks;
    actor.equipment = heart.then_some(Equipment::Heart);
    actor.effects = shield_ticks
        .map(|remaining_ticks| ObservedEffect {
            kind: EffectKind::Shield,
            remaining_ticks,
        })
        .into_iter()
        .collect();
    let environment =
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0); actor_count]);
    let pits = ObservedSpikePits::new(
        SpikePitConfig::Enabled(SpikePitSettings::new(6, 60).unwrap()),
        pit_cells
            .into_iter()
            .map(|cell| SpikePitView {
                cell,
                phase: if cell == pit {
                    phase
                } else {
                    SpikePitPhase::Idle
                },
                remaining_ticks: if cell == pit { remaining_ticks } else { 0 },
            })
            .collect(),
    );
    (
        Observation::new_rules_v10(parts, environment, None, Some(pits)).unwrap(),
        pit,
    )
}

fn departing(pit: CellId) -> MovementSegment {
    MovementSegment::new(
        Position::center(pit),
        Position {
            x: Position::center(pit).x + 1_200,
            y: Position::center(pit).y,
        },
    )
}

#[test]
fn warning_rise_is_resolved_before_an_escape_segment() {
    let (observation, pit) = observed(SpikePitPhase::Warning, 1, None, 0, false);
    let mut work = WorkBudget::new(32);
    let mut path = PitTrajectory::from_observation(&observation, &mut work).unwrap();
    assert_eq!(
        path.advance(Tick(1), Position::center(pit), &[departing(pit)], &mut work,),
        Err(RouteFailure::NoRoute)
    );
}

#[test]
fn respawn_pulse_uses_the_same_rise_before_escape_trajectory() {
    let (observation, _) = observed(SpikePitPhase::Warning, 1, None, 0, false);
    let forecast = crate::cpu::routing::tests::forecast_for(&observation);
    let request = crate::cpu::routing::tests::request(
        crate::cpu::MovementIntent::Direction(crate::game::Direction::East),
        1,
    );
    assert_eq!(
        super::super::respawn_pulse::certify_respawn_pulse(
            &observation,
            &forecast,
            request,
            &mut WorkBudget::new(65_536),
        ),
        Err(RouteFailure::NoRoute)
    );
}

#[test]
fn idle_contact_arms_and_history_survives_leaving_and_revisit() {
    let (observation, pit) = observed(SpikePitPhase::Idle, 0, None, 0, false);
    let mut work = WorkBudget::new(64);
    let mut path = PitTrajectory::from_observation(&observation, &mut work).unwrap();
    let idle_key = path.state_key();
    let standing = MovementSegment::new(Position::center(pit), Position::center(pit));
    path.advance(Tick(1), Position::center(pit), &[standing], &mut work)
        .unwrap();
    assert_ne!(path.state_key(), idle_key);
    let away = departing(pit).end();
    let away_segment = MovementSegment::new(away, away);
    path.advance(Tick(2), away, &[away_segment], &mut work)
        .unwrap();
    path.advance(Tick(3), away, &[departing(pit)], &mut work)
        .unwrap();
    assert_eq!(
        path.guard().unwrap().expected()[0].phase,
        SpikePitPhase::Warning
    );
}

#[test]
fn only_public_active_protection_allows_raised_origin_exit_before_expiry() {
    for (shield, grace) in [(Some(2), 0), (None, 2)] {
        let (observation, pit) = observed(SpikePitPhase::Raised, 20, shield, grace, false);
        let mut work = WorkBudget::new(32);
        let mut path = PitTrajectory::from_observation(&observation, &mut work).unwrap();
        path.advance(Tick(1), Position::center(pit), &[departing(pit)], &mut work)
            .unwrap();
    }

    let (expiring, pit) = observed(SpikePitPhase::Raised, 20, Some(1), 0, false);
    let mut work = WorkBudget::new(32);
    assert_eq!(
        PitTrajectory::from_observation(&expiring, &mut work)
            .unwrap()
            .advance(Tick(1), Position::center(pit), &[departing(pit)], &mut work,),
        Err(RouteFailure::NoRoute)
    );

    let (heart, pit) = observed(SpikePitPhase::Raised, 20, None, 0, true);
    let mut work = WorkBudget::new(32);
    assert_eq!(
        PitTrajectory::from_observation(&heart, &mut work)
            .unwrap()
            .advance(Tick(1), Position::center(pit), &[departing(pit)], &mut work,),
        Err(RouteFailure::NoRoute)
    );
}

#[test]
fn stationary_fast_forward_stops_at_the_first_unprotected_raised_tick() {
    let (observation, pit) = observed(SpikePitPhase::Raised, 60, Some(2), 0, false);
    let mut work = WorkBudget::new(32);
    let mut path = PitTrajectory::from_observation(&observation, &mut work).unwrap();
    assert_eq!(
        path.stationary_until(Tick(61), Position::center(pit), &mut work),
        Err(RouteFailure::NoRoute)
    );
    assert!(work.spent() < 16, "event advancement must stay bounded");
}

#[test]
fn route_search_and_exact_replay_choose_departure_before_protection_expires() {
    let (observation, _) = observed(SpikePitPhase::Raised, 60, Some(40), 0, false);
    let forecast = crate::cpu::routing::tests::forecast_for(&observation);
    let route = super::super::route::plan_respawn_route(
        &observation,
        &forecast,
        super::super::RouteRequest {
            start_window: crate::cpu::forecast::TickWindow {
                start: Tick(1),
                end: Tick(2),
            },
            settled_stop_tick: Tick(0),
            survive_until: Tick(61),
            mode: super::super::RouteMode::CertifiedOnly,
            limits: super::super::RouteLimits::default(),
        },
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    assert!(matches!(
        route.steps()[0],
        super::super::RouteStep::Move { .. }
    ));
    let lane = route.lane(Tick(1)).unwrap();
    assert!(lane.spike_pit_guard_at(Tick(60)).is_some());
}

#[test]
fn guard_binds_disabled_enabled_empty_settings_and_timestamp() {
    let (observation, _) = observed(SpikePitPhase::Idle, 0, None, 0, false);
    let mut work = WorkBudget::new(32);
    let guard = PitTrajectory::from_observation(&observation, &mut work)
        .unwrap()
        .guard()
        .unwrap();
    assert!(guard.matches(&observation, &mut work).unwrap());
    let changed = observation
        .clone()
        .map_parts(|parts| parts.tick = Tick(1))
        .unwrap();
    assert!(!guard.matches(&changed, &mut work).unwrap());
}
