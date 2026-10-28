use crate::cpu::{BombKnowledge, Observation, ObservedFlame, WorkBudget};
use crate::game::{BlastKind, CellId, Terrain};
use crate::room::BoardSize;

use super::tests::{observation, timed};
use super::{ForecastLimits, ObservedHazardMemory, forecast};

fn resized(board: BoardSize, bombs: BombKnowledge) -> Observation {
    let mut parts = observation(0, bombs).into_parts();
    let (width, height) = board.dimensions();
    parts.rules.version = 6;
    parts.arena.width = width;
    parts.arena.height = height;
    parts.arena.terrain = vec![Terrain::Floor; board.cell_count()];
    for row in 0..height {
        for column in 0..width {
            if row == 0 || column == 0 || row + 1 == height || column + 1 == width {
                parts.arena.terrain[usize::from(row) * usize::from(width) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    Observation::new(parts).unwrap()
}

fn forecast_for(observation: &Observation, work: &mut WorkBudget) -> super::HazardForecast {
    forecast(
        observation,
        &ObservedHazardMemory::new(super::tests::key()),
        None,
        ForecastLimits::default(),
        work,
    )
    .unwrap()
}

#[test]
fn small_power_forecast_uses_authoritative_dimensions_without_axis_aliases() {
    let mut bomb = timed(1, CellId::new(5, 4), 2, BlastKind::Power);
    bomb.range = 8;
    let observed = resized(BoardSize::ElevenByNine, BombKnowledge::Visible(vec![bomb]));
    let mut work = WorkBudget::new(65_536);
    let hazards = forecast_for(&observed, &mut work);

    assert_eq!(hazards.dimensions(), (11, 9));
    assert_eq!(hazards.cells().len(), 99);
    assert!(!hazards.cell(CellId::new(9, 7)).unwrap().lethal.is_empty());
    assert!(hazards.cell(CellId::new(10, 7)).unwrap().lethal.is_empty());
    assert!(hazards.cell(CellId::new(11, 0)).is_none());
    assert!(work.spent() <= 65_536);
}

#[test]
fn forecast_evolution_rejects_cross_dimension_reuse() {
    let large = observation(0, BombKnowledge::Visible(Vec::new()));
    let small = resized(BoardSize::ElevenByNine, BombKnowledge::Visible(Vec::new()));
    let mut work = WorkBudget::new(65_536);
    let hazards = forecast_for(&large, &mut work);

    assert!(!hazards.evolution().matches_scene(&small, &small));
}

#[test]
fn small_forecast_accepts_edge_hazards_at_the_last_valid_axis() {
    let mut parts = resized(
        BoardSize::ThirteenByEleven,
        BombKnowledge::Visible(Vec::new()),
    )
    .into_parts();
    parts.flames.push(ObservedFlame {
        cell: CellId::new(11, 9),
        remaining_ticks: 3,
    });
    let observed = Observation::new(parts).unwrap();
    let mut work = WorkBudget::new(65_536);
    let hazards = forecast_for(&observed, &mut work);

    assert!(!hazards.cell(CellId::new(11, 9)).unwrap().lethal.is_empty());
    assert!(hazards.cell(CellId::new(12, 9)).unwrap().lethal.is_empty());
}
