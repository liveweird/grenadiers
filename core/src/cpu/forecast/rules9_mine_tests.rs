use super::tests::{key, observation, timed};
use super::{ForecastEvolution, ObservedHazardMemory, forecast};
use crate::cpu::{
    BombKnowledge, Observation, ObservationParts, ObservedActor, ObservedBomb, ObservedBombTrigger,
    ObservedEnvironment, WorkBudget,
};
use crate::game::{BlastKind, CellId, EntityId, StormCloudConfig, Tick};

fn observed_bombs(mines: usize) -> Vec<ObservedBomb> {
    let mut bombs = vec![timed(99, CellId::new(7, 1), 300, BlastKind::Standard)];
    bombs.extend((0..mines).map(|index| {
        let mut mine = timed(
            100 + index as u32,
            CellId::new(1 + (index % 13) as u16, 3 + (index / 13) as u16),
            0,
            BlastKind::Standard,
        );
        mine.trigger = ObservedBombTrigger::Contact;
        mine
    }));
    bombs
}

fn checked(parts: ObservationParts) -> Observation {
    Observation::new_rules_v9(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0); 2]),
        None,
    )
    .unwrap()
}

fn extra_mines(mines: usize) -> Observation {
    let mut parts = observation(0, BombKnowledge::Visible(Vec::new()))
        .parts()
        .clone();
    parts.rules.version = 9;
    parts.bombs = BombKnowledge::Visible(observed_bombs(mines));
    if let ObservedActor::Visible(actor) = &mut parts.actors[0] {
        actor.live_bomb_count = 1;
    }
    checked(parts)
}

#[test]
fn dense_mines_keep_normal_slot_accounting_and_removal_history() {
    let before = extra_mines(33);
    let mut evolution = ForecastEvolution::new(&before, Tick(30)).unwrap();
    assert_eq!(
        evolution.expected_live_bombs(EntityId(10), Tick(0)),
        Some(1)
    );
    evolution
        .record(
            Tick(1),
            [EntityId(100)],
            [],
            [],
            [],
            &mut WorkBudget::new(65_536),
        )
        .unwrap();
    assert_eq!(
        evolution.expected_live_bombs(EntityId(10), Tick(1)),
        Some(1)
    );
    let mut parts = before.parts().clone();
    parts.tick = Tick(1);
    parts.rules.running_updates += 1;
    parts.rules.remaining_ticks -= 1;
    let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
        unreachable!()
    };
    bombs.retain(|bomb| bomb.id != EntityId(100));
    bombs[0].trigger = ObservedBombTrigger::Timed {
        remaining_ticks: 299,
        paused: false,
    };
    assert!(evolution.matches_scene(&before, &checked(parts)));
    let mut budget = WorkBudget::new(65_536);
    let result = forecast(
        &before,
        &ObservedHazardMemory::new(key()),
        None,
        super::ForecastLimits::default(),
        &mut budget,
    );
    // Dense overlapping uncertainty retains the established per-cell interval ceiling.
    assert!(matches!(
        result,
        Err(super::ForecastFailure::IntervalCapacity)
    ));
    let sparse = extra_mines(1);
    assert!(
        forecast(
            &sparse,
            &ObservedHazardMemory::new(key()),
            None,
            super::ForecastLimits::default(),
            &mut WorkBudget::new(65_536),
        )
        .is_ok()
    );
    assert!(budget.spent() <= 65_536);
}

#[test]
fn archived_forecasts_still_count_mines_and_reject_dense_bomb_scenes() {
    let mut parts = extra_mines(1).parts().clone();
    parts.rules.version = 6;
    if let ObservedActor::Visible(actor) = &mut parts.actors[0] {
        actor.live_bomb_count = 2;
    }
    let old = Observation::new(parts).unwrap();
    let evolution = ForecastEvolution::new(&old, Tick(30)).unwrap();
    assert_eq!(
        evolution.expected_live_bombs(EntityId(10), Tick(0)),
        Some(2)
    );
    let mut dense = extra_mines(33).parts().clone();
    dense.rules.version = 6;
    assert!(Observation::new(dense).is_err());
}

#[test]
fn extra_mine_observations_require_distinct_bomb_cells() {
    let mut parts = extra_mines(1).parts().clone();
    let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
        unreachable!()
    };
    bombs[1].cell = bombs[0].cell;
    assert!(
        Observation::new_rules_v9(
            parts,
            ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0); 2]),
            None,
        )
        .is_err()
    );
}
