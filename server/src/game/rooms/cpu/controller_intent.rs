use grenadiers_core::cpu::BombKnowledge;
use grenadiers_core::cpu::policy::{IntentDebugSummary, StrategicPhase, StrategicSwitchReason};

use super::CpuController;

#[derive(Clone, Copy)]
pub(super) struct StrategicBlastContext {
    world: grenadiers_core::cpu::WorldId,
    cell: grenadiers_core::game::CellId,
    range: u16,
    blast: grenadiers_core::game::BlastKind,
}

impl CpuController {
    pub(in crate::game::rooms) fn intent_debug_summary_for(
        &self,
        expected_world: grenadiers_core::cpu::WorldId,
    ) -> Option<IntentDebugSummary> {
        if self.world != Some(expected_world) || self.memory?.key().world != expected_world {
            return None;
        }
        let intent = self.memory?.strategic_intent()?;
        Some(IntentDebugSummary {
            cpu_seat: self.cpu_seat,
            objective: intent.objective,
            target: intent.target,
            approach_cell: intent.approach_cell,
            phase: intent.phase,
            last_switch_reason: intent.last_switch_reason,
            observation_tick: intent.observation_tick,
        })
    }

    pub(super) fn observe_strategic_intent(
        &mut self,
        observation: &grenadiers_core::cpu::Observation,
    ) {
        let Some(memory) = self.memory else { return };
        let Some(intent) = memory.strategic_intent() else {
            return;
        };
        if matches!(
            intent.phase,
            StrategicPhase::Approach | StrategicPhase::Reengage | StrategicPhase::Evade
        ) {
            self.strategic_blast = None;
        }
        let owner = observation
            .parts()
            .actors
            .iter()
            .find(|actor| actor.seat() == self.cpu_seat)
            .map(grenadiers_core::cpu::ObservedActor::id);
        let owned_bomb = match &observation.parts().bombs {
            BombKnowledge::Visible(bombs) => {
                owner.and_then(|owner| bombs.iter().find(|bomb| bomb.owner == owner))
            }
            BombKnowledge::Unavailable => None,
        };
        if let Some(bomb) = owned_bomb.filter(|_| {
            matches!(
                intent.phase,
                StrategicPhase::Plant | StrategicPhase::Escape | StrategicPhase::Waiting
            )
        }) {
            self.strategic_blast = Some(StrategicBlastContext {
                world: observation.parts().world,
                cell: bomb.cell,
                range: bomb.range,
                blast: bomb.blast,
            });
        }
        let waiting_hazard = owned_bomb.is_some()
            || self.strategic_blast.is_some_and(|context| {
                context.world == observation.parts().world
                    && observation
                        .parts()
                        .flames
                        .iter()
                        .any(|flame| relevant_flame(context, flame.cell))
            });
        self.memory = Some(match intent.phase {
            StrategicPhase::Plant if owned_bomb.is_some() => memory.mark_strategic_phase(
                StrategicPhase::Escape,
                StrategicSwitchReason::SafetyInterrupt,
                observation.parts().tick,
            ),
            StrategicPhase::Plant if self.committed_route.is_some() => {
                memory.suspend_strategic_deadline(observation.parts().tick)
            }
            StrategicPhase::Plant => memory.mark_strategic_phase(
                StrategicPhase::Approach,
                intent.last_switch_reason,
                observation.parts().tick,
            ),
            StrategicPhase::Escape if waiting_hazard && self.committed_route.is_some() => {
                memory.suspend_strategic_deadline(observation.parts().tick)
            }
            StrategicPhase::Escape if waiting_hazard => memory
                .mark_strategic_phase(
                    StrategicPhase::Waiting,
                    intent.last_switch_reason,
                    observation.parts().tick,
                )
                .suspend_strategic_deadline(observation.parts().tick),
            StrategicPhase::Escape => memory.mark_strategic_phase(
                StrategicPhase::Reengage,
                StrategicSwitchReason::Resumed,
                observation.parts().tick,
            ),
            StrategicPhase::Waiting
                if intent.target != grenadiers_core::cpu::policy::StrategicTarget::None
                    && waiting_hazard =>
            {
                memory.suspend_strategic_deadline(observation.parts().tick)
            }
            StrategicPhase::Waiting
                if intent.target != grenadiers_core::cpu::policy::StrategicTarget::None =>
            {
                self.strategic_blast = None;
                memory.mark_strategic_phase(
                    StrategicPhase::Reengage,
                    StrategicSwitchReason::Resumed,
                    observation.parts().tick,
                )
            }
            _ => memory,
        });
    }
}

fn relevant_flame(context: StrategicBlastContext, cell: grenadiers_core::game::CellId) -> bool {
    let column = context.cell.column.abs_diff(cell.column);
    let row = context.cell.row.abs_diff(cell.row);
    match context.blast {
        grenadiers_core::game::BlastKind::Power => column.saturating_add(row) <= context.range,
        grenadiers_core::game::BlastKind::Standard | grenadiers_core::game::BlastKind::Piercing => {
            (column == 0 && row <= context.range) || (row == 0 && column <= context.range)
        }
    }
}

#[cfg(test)]
mod tests {
    use grenadiers_core::cpu::ObservedFlame;
    use grenadiers_core::cpu::policy::{
        STRATEGIC_PROGRESS_TICKS, StrategicIntent, StrategicObjectiveKind, StrategicTarget,
        UtilityMemory,
    };
    use grenadiers_core::game::{BlastKind, CellId, Tick};
    use grenadiers_core::room::CpuProfile;

    use super::*;
    use crate::game::rooms::cpu::CpuPlanner;

    fn waiting_controller() -> (CpuController, grenadiers_core::cpu::Observation) {
        let observation = crate::game::rooms::cpu::tests::observation();
        let mut controller = CpuController::new(
            CpuPlanner::without_workers(),
            observation.parts().recipient,
            observation.parts().world.room_instance,
            [24; 32],
            CpuProfile::CURRENT,
        );
        controller.reset_world(observation.parts().world.physical_round);
        let key = controller.memory.unwrap().key();
        controller.memory = Some(
            UtilityMemory::new(key).with_strategic_intent(StrategicIntent {
                objective: StrategicObjectiveKind::Farm,
                target: StrategicTarget::Block {
                    cell: CellId::new(8, 7),
                },
                approach_cell: Some(CellId::new(7, 7)),
                phase: StrategicPhase::Waiting,
                last_switch_reason: StrategicSwitchReason::SafetyInterrupt,
                observation_tick: Tick(19),
                score: 2,
                committed_until: Tick(30),
                progress_deadline: Tick(20),
                last_distance: 0,
                last_cpu_position: grenadiers_core::game::Position::center(CellId::new(7, 7)),
                deferred_until: None,
            }),
        );
        controller.strategic_blast = Some(StrategicBlastContext {
            world: observation.parts().world,
            cell: CellId::new(7, 7),
            range: 2,
            blast: BlastKind::Standard,
        });
        (controller, observation)
    }

    #[test]
    fn only_relevant_owned_blast_flames_suspend_waiting() {
        let (mut relevant, observation) = waiting_controller();
        let relevant_observation = observation
            .clone()
            .map_parts(|parts| {
                parts.flames.push(ObservedFlame {
                    cell: CellId::new(8, 7),
                    remaining_ticks: 2,
                });
            })
            .unwrap();
        relevant.observe_strategic_intent(&relevant_observation);
        let retained = relevant.memory.unwrap().strategic_intent().unwrap();
        assert_eq!(retained.phase, StrategicPhase::Waiting);
        assert_eq!(
            retained.progress_deadline,
            Tick(relevant_observation.parts().tick.0 + STRATEGIC_PROGRESS_TICKS)
        );

        let (mut unrelated, observation) = waiting_controller();
        let unrelated_observation = observation
            .map_parts(|parts| {
                parts.flames.push(ObservedFlame {
                    cell: CellId::new(2, 2),
                    remaining_ticks: 2,
                });
            })
            .unwrap();
        unrelated.observe_strategic_intent(&unrelated_observation);
        let resumed = unrelated.memory.unwrap().strategic_intent().unwrap();
        assert_eq!(resumed.phase, StrategicPhase::Reengage);
        assert_eq!(resumed.last_switch_reason, StrategicSwitchReason::Resumed);
        assert!(unrelated.strategic_blast.is_none());
    }
}
