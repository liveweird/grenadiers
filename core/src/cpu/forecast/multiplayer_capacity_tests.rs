use crate::cpu::{
    BombKnowledge, Observation, ObservedBomb, ObservedBombTrigger, UnknownBombFacts, WorkBudget,
};
use crate::game::{BlastKind, CellId, EntityId, Terrain, Tick};

use super::tests::{key, observation};
use super::{
    ForecastFailure, ForecastLimits, HypotheticalPlacement, ObservedHazardMemory, forecast,
};

fn bombs(count: usize) -> Vec<ObservedBomb> {
    (0..count)
        .map(|index| ObservedBomb {
            id: EntityId(100 + index as u32),
            placed_at: Tick(0),
            cell: CellId::new((2 * (index % 7) + 1) as u16, (2 * (index / 7) + 1) as u16),
            owner: EntityId(10),
            range: 1,
            trigger: ObservedBombTrigger::Contact,
            blast: BlastKind::Standard,
            unknown: UnknownBombFacts::ALL,
        })
        .collect()
}

fn observed(rules_version: u16, count: usize) -> Observation {
    let mut parts = observation(0, BombKnowledge::Visible(Vec::new())).into_parts();
    parts.rules.version = rules_version;
    let bombs = bombs(count);
    parts.arena.terrain.fill(Terrain::Wall);
    for bomb in &bombs {
        let index = usize::from(bomb.cell.row) * usize::from(parts.arena.width)
            + usize::from(bomb.cell.column);
        parts.arena.terrain[index] = Terrain::Floor;
    }
    let hypothetical = placement().cell;
    let index = usize::from(hypothetical.row) * usize::from(parts.arena.width)
        + usize::from(hypothetical.column);
    parts.arena.terrain[index] = Terrain::Floor;
    parts.bombs = BombKnowledge::Visible(bombs);
    Observation::new(parts).unwrap()
}

fn placement() -> HypotheticalPlacement {
    HypotheticalPlacement {
        placed_at: Tick(7),
        owner: EntityId(20),
        cell: CellId::new(13, 11),
        range: 1,
        blast: BlastKind::Standard,
        detonates_at: Tick(180),
    }
}

fn run(
    observation: &Observation,
    hypothetical: Option<HypotheticalPlacement>,
) -> Result<(), ForecastFailure> {
    forecast(
        observation,
        &ObservedHazardMemory::new(key()),
        hypothetical,
        ForecastLimits::default(),
        &mut WorkBudget::new(65_536),
    )
    .map(|_| ())
}

#[test]
fn rules_six_forecasts_thirty_two_visible_bombs_without_changing_legacy_capacity() {
    assert_eq!(run(&observed(6, 32), None), Ok(()));
    assert_eq!(
        run(&observed(5, 16), Some(placement())),
        Err(ForecastFailure::IntervalCapacity)
    );
    assert_eq!(run(&observed(5, 15), Some(placement())), Ok(()));
}

#[test]
fn rules_six_hypothetical_placement_obeys_its_thirty_two_bomb_capacity() {
    assert_eq!(run(&observed(6, 31), Some(placement())), Ok(()));
    assert_eq!(
        run(&observed(6, 32), Some(placement())),
        Err(ForecastFailure::IntervalCapacity)
    );
}
