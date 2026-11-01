use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, forecast};
use crate::cpu::policy::{ControlPlan, UtilityMemory};
use crate::cpu::{BombKnowledge, MemoryKey, Observation, ObservedActor, ObservedFlame, WorkBudget};
use crate::game::{CellId, Position, SeatId};

use super::refuge_centering_test_fixture::captured_scene;
use super::steering::{RefugeGuidance, target};

fn target_for(observation: &Observation, guidance: RefugeGuidance) -> Option<CellId> {
    target_with_forecast_from(observation, observation, guidance)
}

fn target_with_forecast_from(
    observation: &Observation,
    forecast_source: &Observation,
    guidance: RefugeGuidance,
) -> Option<CellId> {
    let key = MemoryKey {
        world: forecast_source.parts().world,
        cpu_seat: forecast_source.parts().recipient,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(forecast_source).unwrap();
    let forecast = forecast(
        forecast_source,
        &history,
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    target(
        observation,
        &forecast,
        &ControlPlan::stop(UtilityMemory::new(key)),
        guidance,
        &mut WorkBudget::new(65_536),
    )
}

fn actor_at(observation: Observation, seat: SeatId, position: Position) -> Observation {
    observation
        .map_parts(|parts| {
            let actor = parts
                .actors
                .iter_mut()
                .find_map(|actor| match actor {
                    ObservedActor::Visible(actor) if actor.seat == seat => Some(actor),
                    _ => None,
                })
                .unwrap();
            actor.position = position;
        })
        .unwrap()
}

#[test]
fn captured_horizontal_edges_center_the_safe_current_root_for_both_sides() {
    for (seat, root) in [
        (SeatId::Three, CellId::new(2, 9)),
        (SeatId::Four, CellId::new(12, 9)),
    ] {
        let observation = captured_scene(seat);
        assert_ne!(
            target_for(&observation, RefugeGuidance::default()),
            Some(root)
        );
        assert_eq!(
            target_for(&observation, RefugeGuidance::CENTER_THREATENED_ROOT),
            Some(root)
        );
    }
}

#[test]
fn vertical_footprint_edges_center_the_safe_current_root() {
    for (position, flame_cell) in [
        (Position { x: 1536, y: 9218 }, CellId::new(1, 8)),
        (Position { x: 1536, y: 10_238 }, CellId::new(1, 10)),
    ] {
        let observation = actor_at(captured_scene(SeatId::Three), SeatId::Three, position)
            .map_parts(|parts| {
                parts.bombs = BombKnowledge::Visible(Vec::new());
                parts.flames = vec![ObservedFlame {
                    cell: flame_cell,
                    remaining_ticks: 130,
                }];
            })
            .unwrap();
        assert_eq!(
            target_for(&observation, RefugeGuidance::CENTER_THREATENED_ROOT),
            Some(CellId::new(1, 9))
        );
    }
}

#[test]
fn blocked_or_definitely_lethal_current_root_is_not_selected_for_centering() {
    let source = captured_scene(SeatId::Three);
    let blocked = source
        .clone()
        .map_parts(|parts| {
            parts.arena.blocks.push(CellId::new(2, 9));
        })
        .unwrap();
    assert_ne!(
        target_with_forecast_from(&blocked, &source, RefugeGuidance::CENTER_THREATENED_ROOT),
        Some(CellId::new(2, 9))
    );

    let lethal = captured_scene(SeatId::Three)
        .map_parts(|parts| {
            parts.bombs = BombKnowledge::Visible(Vec::new());
            parts.flames = vec![ObservedFlame {
                cell: CellId::new(2, 9),
                remaining_ticks: 130,
            }];
        })
        .unwrap();
    assert_ne!(
        target_for(&lethal, RefugeGuidance::CENTER_THREATENED_ROOT),
        Some(CellId::new(2, 9))
    );
}
