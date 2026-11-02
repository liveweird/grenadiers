use grenadiers_core::{
    cpu::survival::WORK_ALLOWANCE,
    game::{CellId, EntityId, SeatId, Tick},
    room::CpuProfile,
};

use super::early_farm_owner_interaction_support::{CPUS, run_inspecting};

#[derive(Clone, Copy, Debug)]
struct FirstPlant {
    id: EntityId,
    placed_at: Tick,
    detonates_at: Tick,
    flame_end: Tick,
    cell: CellId,
    range: u16,
}

#[test]
fn corner_refuges_produce_a_useful_natural_plant_and_survive_its_actual_flames() {
    for profile in [
        CpuProfile::HARD_V57,
        CpuProfile::HARD_V58,
        CpuProfile::HARD_V59,
        CpuProfile::CORNER_REFUGES_V56_STANDARD,
        CpuProfile::CORNER_REFUGES_V56_EASY,
    ] {
        let mut first = None;
        let mut deaths = [None; 3];
        let mut intact_before = false;
        let mut destroyed_at = false;
        let mut flame_seen = false;
        let mut resolved = false;
        let target = CellId::new(11, 2);
        let episode = run_inspecting(profile, 600, |state| {
            for (index, seat) in CPUS.into_iter().enumerate() {
                if !state.actor(seat).unwrap().alive() && deaths[index].is_none() {
                    deaths[index] = Some(state.tick());
                }
            }
            let cpu_id = state.actor(SeatId::Two).unwrap().id();
            if first.is_none()
                && let Some((&id, bomb)) = state
                    .world()
                    .bombs()
                    .iter()
                    .find(|(_, bomb)| bomb.owner() == cpu_id)
            {
                let detonates_at = bomb
                    .detonates_at()
                    .expect("fixture plants ordinary timed bombs");
                first = Some(FirstPlant {
                    id,
                    placed_at: bomb.placed_at(),
                    detonates_at,
                    flame_end: Tick(detonates_at.0 + state.rules().flame_ticks()),
                    cell: bomb.cell(),
                    range: bomb.range(),
                });
            }
            if let Some(plant) = first {
                if state.tick().0 + 1 == plant.detonates_at.0 {
                    intact_before = state.world().arena().blocks().contains(&target);
                }
                if state.tick() == plant.detonates_at {
                    destroyed_at = !state.world().arena().blocks().contains(&target);
                    flame_seen = state.world().hazards().get(&target) == Some(&plant.flame_end);
                }
                if state.tick().0 == plant.flame_end.0 + 1 {
                    resolved = !state.world().bombs().contains_key(&plant.id)
                        && !state.world().hazards().contains_key(&target);
                }
            }
        });
        eprintln!(
            "CORNER_FARM profile={profile:?} first={first:?} deaths={deaths:?} intact={intact_before} destroyed={destroyed_at} flame={flame_seen} resolved={resolved} first_plants={:?} admissions={:?} work_max={:?}",
            episode.first_plants,
            episode.admissions,
            episode.worker.work.iter().max(),
        );
        let first = first.expect("the current policy must make a natural useful plant");
        assert_eq!(first.placed_at, Tick(439));
        assert_eq!(first.cell, CellId::new(11, 1));
        assert_eq!(first.range, 2);
        assert_eq!(first.detonates_at, Tick(619));
        assert_eq!(first.flame_end, Tick(649));
        assert!(intact_before && destroyed_at && flame_seen && resolved);
        assert!(
            deaths
                .iter()
                .all(|death| death.is_none_or(|tick| tick > Tick(650)))
        );
        assert_eq!(episode.first_plants[0], Some(first.placed_at));
        assert_eq!(episode.worker.dequeued, episode.worker.delivered);
        assert_eq!(
            (
                episode.worker.failed,
                episode.worker.cancelled,
                episode.worker.dropped
            ),
            (0, 0, 0)
        );
        assert!(
            episode
                .worker
                .work
                .iter()
                .all(|work| *work <= WORK_ALLOWANCE)
        );
    }
}
