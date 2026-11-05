use grenadiers_core::{game::SeatId, room::DepartureReason};

use super::{
    super::{
        replay_v8::ReplayAction,
        replay_v39, replay_v40,
        roster_runtime::{
            EvaluationOperation, RosterAction, RosterRuntime, RosterRuntimeInspection,
        },
    },
    live_cpu_evaluation_types::{AcceptedCpuActions, CPU_SEATS},
};

pub(super) trait EvaluationRecording: Sized {
    fn runtime(&self) -> &RosterRuntime;
    fn admit(&mut self, seat: SeatId, resume: bool) -> Result<(), String>;
    fn human_action(
        &mut self,
        seat: SeatId,
        epoch: u64,
        action: ReplayAction,
    ) -> Result<(), String>;
    fn cpu_action(&mut self, seat: SeatId, epoch: u64, action: ReplayAction) -> Result<(), String>;
    fn input_fence(&mut self, seat: SeatId);
    fn settle(&mut self, departures: [Option<DepartureReason>; 4]) -> Result<(), String>;
    fn publish_flush(&mut self) -> Result<(), String>;
    fn verify(self, expected: &RosterRuntimeInspection) -> Result<(), String>;
}

macro_rules! recording {
    ($recording:ty, $replay:path) => {
        impl EvaluationRecording for $recording {
            fn runtime(&self) -> &RosterRuntime {
                self.runtime()
            }

            fn admit(&mut self, seat: SeatId, resume: bool) -> Result<(), String> {
                self.admit(seat, resume)
                    .map_err(|error| format!("admit evaluation human: {error:?}"))
            }

            fn human_action(
                &mut self,
                seat: SeatId,
                epoch: u64,
                action: ReplayAction,
            ) -> Result<(), String> {
                self.human_action(seat, epoch, action)
                    .map_err(|error| format!("shadow human action: {error:?}"))
            }

            fn cpu_action(
                &mut self,
                seat: SeatId,
                epoch: u64,
                action: ReplayAction,
            ) -> Result<(), String> {
                self.cpu_action(seat, epoch, action)
                    .map_err(|error| format!("shadow CPU action: {error:?}"))
            }

            fn input_fence(&mut self, seat: SeatId) {
                self.input_fence(seat);
            }

            fn settle(&mut self, departures: [Option<DepartureReason>; 4]) -> Result<(), String> {
                self.settle(departures)
                    .map_err(|error| format!("shadow settle: {error:?}"))
            }

            fn publish_flush(&mut self) -> Result<(), String> {
                self.publish_flush()
                    .map_err(|error| format!("shadow publication flush: {error:?}"))
            }

            fn verify(self, expected: &RosterRuntimeInspection) -> Result<(), String> {
                let (_, journal) = self.into_parts();
                $replay(&journal, expected)
            }
        }
    };
}

recording!(replay_v39::RecordingRosterRuntime, replay_v39::verify_exact);
recording!(replay_v40::RecordingRosterRuntime, replay_v40::verify_exact);

pub(super) fn start_round(recording: &mut impl EvaluationRecording) -> Result<(), String> {
    recording.admit(SeatId::One, false)?;
    recording.human_action(
        SeatId::One,
        1,
        ReplayAction::Ready {
            ready: true,
            settings_revision: 1,
        },
    )?;
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        recording.settle([None; 4])?;
    }
    recording.publish_flush()
}

pub(super) fn drain_trace(
    actual: &mut RosterRuntime,
    shadow: &mut impl EvaluationRecording,
) -> Result<AcceptedCpuActions, String> {
    let mut accepted = AcceptedCpuActions::default();
    for operation in actual
        .drain_evaluation_trace_for_test()
        .map_err(|error| format!("evaluation trace failed: {error:?}"))?
    {
        match operation {
            EvaluationOperation::HumanAction {
                seat,
                epoch,
                action,
            } => shadow.human_action(seat, epoch, replay_action(action)?)?,
            EvaluationOperation::CpuAction {
                seat,
                epoch,
                action,
            } => {
                if let Some(index) = CPU_SEATS.iter().position(|candidate| *candidate == seat) {
                    accepted.movement[index] +=
                        u64::from(matches!(action, RosterAction::Movement(_)));
                    accepted.bombs[index] += u64::from(matches!(action, RosterAction::Bomb(_)));
                }
                shadow.cpu_action(seat, epoch, replay_action(action)?)?;
            }
            EvaluationOperation::InputFence(seat) => shadow.input_fence(seat),
            EvaluationOperation::Settle { tick, departures } => {
                if shadow.runtime().lifecycle().tick() != tick {
                    return Err("shadow settle pre-operation tick mismatch".to_owned());
                }
                shadow.settle(departures)?;
            }
            EvaluationOperation::PublicationFlush => shadow.publish_flush()?,
        }
    }
    Ok(accepted)
}

fn replay_action(action: RosterAction) -> Result<ReplayAction, String> {
    Ok(match action {
        RosterAction::Ready {
            ready,
            settings_revision,
        } => ReplayAction::Ready {
            ready,
            settings_revision,
        },
        RosterAction::Movement(direction) => ReplayAction::Movement(direction),
        RosterAction::Bomb(id) => ReplayAction::Bomb(id),
        RosterAction::Equipment(id) => ReplayAction::Equipment(id),
        RosterAction::Rematch(value) => ReplayAction::Rematch(value),
        RosterAction::UpdateSettings { .. } => {
            return Err("generic settings updates are outside live evaluation".to_owned());
        }
    })
}
