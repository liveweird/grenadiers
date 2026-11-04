use grenadiers_core::{
    cpu::policy::{StrategicObjectiveKind, StrategicPhase, StrategicTarget},
    game::{ArenaConstructionProfile, CellId, SeatId, Tick},
    room::{BoardSize, CpuProfile, SpikePitSettings},
};

use super::trace::{LayoutCase, run_zero_bomber_trace_with_profile};

#[test]
fn hard_v64_demolishes_visible_block_and_survives_its_certified_bomb() {
    let result = run_zero_bomber_trace_with_profile(
        LayoutCase {
            label: "zero-bomber-v64",
            board: BoardSize::ThirteenByEleven,
            lairs: true,
            seed: 73,
            profile: ArenaConstructionProfile::LairClearanceV5,
            pit_settings: SpikePitSettings::new(60, 300).unwrap(),
        },
        840,
        SeatId::Two,
        CpuProfile::HARD_V64,
    );
    let (witness, plan, intent) = result
        .plan_witnesses
        .iter()
        .find_map(|witness| {
            let plan = witness.plan.as_ref()?;
            let intent = plan.control.next_memory.strategic_intent()?;
            (intent.objective == StrategicObjectiveKind::Farm
                && intent.target
                    == StrategicTarget::Block {
                        cell: CellId::new(11, 3),
                    })
            .then_some((witness, plan, intent))
        })
        .expect("V64 adopts the certified visible-block Farm intent");
    assert_eq!(witness.observation.parts().tick, Tick(183));
    assert_eq!(intent.approach_cell, Some(CellId::new(11, 1)));
    assert_eq!(intent.phase, StrategicPhase::Plant);
    assert!(witness.work_spent <= grenadiers_core::cpu::survival::WORK_ALLOWANCE);
    let certificate = plan
        .execution
        .as_ref()
        .and_then(|value| value.plant())
        .expect("admitted plan retains the complete Plant certificate");
    assert_eq!(certificate.route().survive_until(), Tick(400));
    let lifecycle = result.first_bombs[0].expect("real Seat Two bomb");
    assert_eq!(lifecycle.cell, CellId::new(11, 1));
    assert_eq!(lifecycle.placed_at, Tick(190));
    assert_eq!(lifecycle.detonated_at, Some(Tick(370)));
    assert_eq!(lifecycle.all_finite_flames_clear_at, Some(Tick(400)));
    let target = CellId::new(11, 3);
    assert!(result.initial_world.arena().blocks().contains(&target));
    let before_detonation = result
        .frames
        .iter()
        .find(|frame| frame.tick == Tick(369))
        .unwrap();
    assert!(before_detonation.blocks.contains(&target));
    assert!(
        result.frames.iter().any(|frame| {
            (Tick(370)..Tick(400)).contains(&frame.tick) && !frame.flames.is_empty()
        })
    );
    let at_clear = result
        .frames
        .iter()
        .find(|frame| frame.tick == Tick(400))
        .unwrap();
    assert!(!at_clear.blocks.contains(&target));
    assert!(at_clear.flames.is_empty());
    assert!(
        at_clear
            .actors
            .iter()
            .any(|actor| actor.seat == SeatId::Two && actor.alive)
    );
}
