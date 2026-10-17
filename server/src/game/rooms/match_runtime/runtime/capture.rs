use grenadiers_core::{
    game::{SeatId, Tick},
    room::{MatchLifecycle, MatchPhase},
};

use super::{MatchAction, MatchRuntime};
use crate::game::rooms::{
    input::InputMailbox,
    profile::RoomProfile,
    replay::{Header, Journal, Operation, ReplayAction},
    types::RoomIdentity,
};
use crate::round_seed::RoundSeedSource;

pub(super) fn journal(profile: RoomProfile) -> Journal {
    match profile {
        RoomProfile::GeneratedClassicV2 => Journal::new(Header::m6([false; 2])),
        RoomProfile::GeneratedClassicV3 => Journal::new(Header::m6_v3([false; 2])),
        #[cfg(test)]
        RoomProfile::FixedClassicV1 => Journal::unsupported_profile(),
    }
}

impl MatchRuntime {
    pub(super) fn with_journal(
        identity: RoomIdentity,
        round_seed_source: RoundSeedSource,
        journal: Journal,
    ) -> Self {
        Self {
            identity,
            lifecycle: MatchLifecycle::new(),
            revision: 0,
            round_seed_source,
            inputs: InputMailbox::default(),
            sim: None,
            reports: Vec::new(),
            journal,
        }
    }

    pub(super) fn capture(&mut self, tick: Tick, operation: Operation) {
        self.journal.record(tick, operation);
    }

    pub(super) fn capture_action(
        &mut self,
        tick: Tick,
        seat: SeatId,
        epoch: u64,
        action: MatchAction,
    ) {
        let action = match action {
            MatchAction::Ready(value) => ReplayAction::Ready(value),
            MatchAction::Movement(value) => ReplayAction::Movement(value),
            MatchAction::Bomb(value) => ReplayAction::Bomb(value),
            MatchAction::Rematch(value) => ReplayAction::Rematch(value),
        };
        self.capture(
            tick,
            Operation::Action {
                seat,
                epoch,
                action,
            },
        );
    }

    pub(super) fn capture_tick(&mut self, previous_tick: Tick, previous_phase: MatchPhase) {
        self.capture(previous_tick, Operation::Tick);
        if previous_phase == MatchPhase::Running {
            return;
        }
        if self.lifecycle.phase() == MatchPhase::Running {
            let marker = self.sim.as_ref().and_then(|sim| {
                sim.generation_record
                    .map(|generation| (sim.epoch, sim.state.round_assignment(), generation))
            });
            if let Some((epoch, assignment, generation)) = marker {
                self.capture(
                    self.lifecycle.tick(),
                    Operation::RoundStarted {
                        epoch,
                        assignment,
                        generation,
                    },
                );
            }
        } else if matches!(previous_phase, MatchPhase::Countdown { .. })
            && internal_error(self.lifecycle.phase())
        {
            self.journal.mark_round_construction_incomplete();
        }
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn replay_journal(&self) -> &Journal {
        &self.journal
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn replay_journal_mut(&mut self) -> &mut Journal {
        &mut self.journal
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn new_without_replay(
        identity: RoomIdentity,
        round_seed_source: RoundSeedSource,
    ) -> Self {
        Self::with_journal(identity, round_seed_source, Journal::unsupported_profile())
    }
}

fn internal_error(phase: MatchPhase) -> bool {
    matches!(
        phase,
        MatchPhase::Interrupted {
            reason: grenadiers_core::room::InterruptReason::InternalError,
            ..
        }
    ) || matches!(phase, MatchPhase::MatchResult { outcome, .. }
    if matches!(outcome.result(), grenadiers_core::room::series::MatchResult::Interrupted(
        grenadiers_core::room::InterruptReason::InternalError
    )))
}
