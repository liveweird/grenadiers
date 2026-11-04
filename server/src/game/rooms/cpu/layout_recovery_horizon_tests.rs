use super::trace::{LayoutCase, run_layout_trace_with_profile};
use grenadiers_core::{
    game::{ArenaConstructionProfile, CellId, InputAction, SeatId, StepEventKind, Tick},
    room::{BoardSize, CpuProfile, SpikePitSettings},
};

#[test]
fn crossing_recovery_survives_own_bomb_and_complete_flame_window() {
    let result = run_layout_trace_with_profile(
        LayoutCase {
            label: "crossing-recovery-witness",
            board: BoardSize::ThirteenByEleven,
            lairs: true,
            seed: 73,
            profile: ArenaConstructionProfile::LairClearanceV5,
            pit_settings: SpikePitSettings::new(60, 300).unwrap(),
        },
        840,
        CpuProfile::HARD_V63,
    );
    let bomb = result.first_bombs[2].expect("Seat Four first bomb");
    assert_eq!(
        (bomb.id.0, bomb.cell, bomb.detonated_at),
        (5, CellId::new(7, 7), Some(Tick(511)))
    );
    assert!(
        bomb.all_finite_flames_clear_at
            .is_some_and(|tick| tick >= Tick(541))
    );
    let active = result
        .frames
        .iter()
        .filter(|frame| frame.continuation_authority[2])
        .map(|frame| frame.tick)
        .collect::<Vec<_>>();
    let deaths = result
        .frames
        .iter()
        .filter(|frame| {
            frame.tick <= Tick(541)
                && frame.report.events.iter().any(|event| {
                    matches!(
                        event.kind,
                        StepEventKind::ActorEliminated {
                            seat: SeatId::Four,
                            ..
                        }
                    )
                })
        })
        .collect::<Vec<_>>();
    assert!(deaths.is_empty());
    assert!((399..=541).all(|tick| active.contains(&Tick(tick))));
    assert_eq!(
        result.continuation_traces[2].completion_tick,
        Some(Tick(542))
    );
    assert!(result.continuation_traces[2].maximum_renewal_work <= 8_192);
    assert!(!result.plan_witnesses.iter().any(|witness| {
        witness.observation.parts().recipient == SeatId::Four
            && (Tick(399)..=Tick(541)).contains(&witness.observation.parts().tick)
    }));
    for frame in result
        .frames
        .iter()
        .filter(|frame| (Tick(399)..=Tick(541)).contains(&frame.tick))
    {
        assert!(
            frame.continuation_authority[2],
            "lost authority at {:?}",
            frame.tick
        );
        assert!(
            frame
                .accepted_inputs
                .iter()
                .all(|(seat, action)| { *seat != SeatId::Four || *action == InputAction::Clear })
        );
    }
    assert!(result.stationary_preflight.iter().all(|probe| {
        probe.spent <= 8_192
            && (probe.safe_until != Tick((probe.publication.0 + 14).min(542))
                || probe.hold_ticks != 0
                || probe.outcome.is_ok())
    }));
}
