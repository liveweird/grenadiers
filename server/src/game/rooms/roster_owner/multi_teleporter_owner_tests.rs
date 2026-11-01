use std::collections::BTreeMap;

use grenadiers_core::{
    cpu::survival::WORK_ALLOWANCE,
    game::{CellId, EntityId, PickupKind, SeatId, Tick},
    room::CpuProfile,
};
use tokio::time::Instant;

use super::teleporter_owner_support::{CPU, TELEPORTER, test_driver_with_pickups};

const SECOND: CellId = CellId::new(12, 11);

#[derive(Debug)]
struct Outcome {
    collected: Vec<(CellId, Tick)>,
    plant: Option<Tick>,
    destroyed: bool,
    opponent_defeated: bool,
    survived: bool,
    moves: usize,
    fences: usize,
    delivered: usize,
    failed: usize,
    max_work: u32,
}

fn run(profile: CpuProfile, limit: usize) -> Outcome {
    let (mut owner, planner) = test_driver_with_pickups(
        profile,
        profile.difficulty,
        false,
        false,
        BTreeMap::from([
            (TELEPORTER, PickupKind::Teleporter),
            (SECOND, PickupKind::Teleporter),
        ]),
    );
    let initial_blocks = owner
        .runtime
        .state_for_test()
        .unwrap()
        .world()
        .arena()
        .blocks()
        .clone();
    let mut remaining = vec![TELEPORTER, SECOND];
    let mut collected = Vec::new();
    let mut plant = None;
    let mut finish = None;
    let mut delivered = 0;
    let mut failed = 0;
    let mut max_work = 0;
    for _ in 0..limit {
        let report = planner.pump_for_test(usize::MAX);
        delivered += report.delivered;
        failed += report.failed + report.cancelled + report.dropped;
        max_work = max_work.max(report.work.into_iter().max().unwrap_or(0));
        owner.tick(Instant::now()).unwrap();
        assert_eq!(
            owner.runtime.lifecycle().round_attempt(),
            1,
            "the witness cannot cross a rematch"
        );
        let state = owner.runtime.state_for_test().unwrap();
        for target in remaining.clone() {
            if !state.world().exposed_pickups().contains_key(&target) {
                if collected.is_empty() && target == TELEPORTER {
                    assert!(
                        state.world().exposed_pickups().contains_key(&SECOND),
                        "an unselected pickup cannot chain in the same update"
                    );
                }
                let marker = state.actor(CPU).unwrap().last_teleport_tick();
                assert_eq!(marker, Some(state.tick()));
                assert_eq!(
                    state.actor(SeatId::One).unwrap().last_teleport_tick(),
                    marker
                );
                collected.push((target, state.tick()));
                remaining.retain(|cell| *cell != target);
            }
        }
        // The other pickup survives every first-target Approach/Collect step.
        if collected.len() == 1 && collected[0].0 == TELEPORTER {
            assert_eq!(
                state
                    .world()
                    .exposed_pickups()
                    .get(&SECOND)
                    .map(|pickup| pickup.kind()),
                Some(PickupKind::Teleporter)
            );
        }
        if plant.is_none()
            && let Some(bomb) = state
                .world()
                .bombs()
                .values()
                .find(|bomb| bomb.owner() == EntityId(2))
        {
            assert!(
                remaining.is_empty(),
                "the two pickups must clear before ordinary planting"
            );
            plant = Some(bomb.placed_at());
            finish = bomb
                .detonates_at()
                .map(|tick| Tick(tick.0 + state.rules().flame_ticks() + 1));
        }
        if !state.actor(SeatId::One).unwrap().alive()
            || finish.is_some_and(|tick| state.tick() >= tick)
        {
            break;
        }
    }
    let state = owner.runtime.state_for_test().unwrap();
    let trace = owner.teleporter_trace_for_test();
    Outcome {
        collected,
        plant,
        destroyed: initial_blocks
            .iter()
            .any(|cell| !state.world().arena().blocks().contains(cell)),
        opponent_defeated: !state.actor(SeatId::One).unwrap().alive(),
        survived: state.actor(CPU).unwrap().alive()
            && (!state.actor(SeatId::One).unwrap().alive()
                || finish.is_some_and(|tick| state.tick() >= tick)),
        moves: trace.iter().filter(|event| event.0 == 0 && event.3).count(),
        fences: trace.iter().filter(|event| event.0 == 2).count(),
        delivered,
        failed,
        max_work,
    }
}

#[test]
fn current_profiles_clear_two_distant_teleporters_then_farm_or_win() {
    for profile in [
        CpuProfile::HARD_V51,
        CpuProfile::TELEPORTER_V50_STANDARD,
        CpuProfile::TELEPORTER_V50_EASY,
    ] {
        let outcome = run(profile, 8_000);
        eprintln!("MULTI_OWNER profile={profile:?} {outcome:?}");
        assert_eq!(outcome.collected.len(), 2);
        assert_eq!(outcome.collected[0].0, TELEPORTER);
        assert_eq!(outcome.collected[1].0, SECOND);
        assert!(outcome.collected[1].1 > outcome.collected[0].1);
        assert!(outcome.plant > Some(outcome.collected[1].1));
        assert!(outcome.destroyed || outcome.opponent_defeated);
        assert!(outcome.survived);
        assert!(outcome.moves > 2 && outcome.delivered > 0);
        assert_eq!(outcome.fences, 0);
        assert_eq!(outcome.failed, 0);
        assert!(outcome.max_work <= WORK_ALLOWANCE);
    }
}

#[test]
fn archived_hard49_retains_its_exact_one_target_boundary() {
    let outcome = run(CpuProfile::HARD_V49, 240);
    assert!(outcome.collected.is_empty());
    assert!(outcome.plant.is_none());
    assert_eq!(outcome.moves, 0);
}
