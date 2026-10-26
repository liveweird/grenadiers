use grenadiers_core::{
    game::{SeatId, Tick},
    room::{MatchLifecycle, MatchPhase},
};

use super::{MatchAction, MatchRuntime};
use crate::game::rooms::{
    input::InputMailbox,
    replay::{Journal, Operation, ReplayAction},
    types::RoomIdentity,
};
#[cfg(test)]
use crate::game::rooms::{profile::RoomProfile, replay::Header};
use crate::round_seed::RoundSeedSource;

#[cfg(test)]
fn journal(profile: RoomProfile) -> Journal {
    match profile {
        RoomProfile::GeneratedClassicV2 => Journal::new(Header::m6([false; 2])),
        RoomProfile::GeneratedClassicV3 => Journal::new(Header::m6_v3([false; 2])),
        RoomProfile::GeneratedConfigurableV4 => Journal::new(Header::configurable_v4()),
        RoomProfile::ExpandedV5 => Journal::new(Header::expanded_v5_with_settings(
            profile.initial_settings(),
        )),
        #[cfg(test)]
        RoomProfile::FixedClassicV1 => Journal::unsupported_profile(),
    }
}

impl MatchRuntime {
    #[cfg(test)]
    pub(in crate::game::rooms) fn new_recording(
        identity: RoomIdentity,
        round_seed_source: RoundSeedSource,
    ) -> Self {
        let settings = identity.profile.initial_settings();
        let settings_editor = identity.profile.initial_editor();
        let journal = journal(identity.profile);
        Self::with_configuration(
            identity,
            round_seed_source,
            Some(journal),
            settings,
            1,
            settings_editor,
        )
    }

    pub(super) fn with_configuration(
        identity: RoomIdentity,
        round_seed_source: RoundSeedSource,
        journal: Option<Journal>,
        settings: grenadiers_core::room::RoomSettings,
        settings_revision: u32,
        settings_editor: Option<SeatId>,
    ) -> Self {
        Self {
            identity,
            lifecycle: MatchLifecycle::with_settings(settings),
            revision: 0,
            settings_revision,
            settings_editor,
            round_seed_source,
            inputs: InputMailbox::default(),
            sim: None,
            reports: Vec::new(),
            journal,
        }
    }

    pub(super) fn capture(&mut self, tick: Tick, operation: Operation) {
        let Some(journal) = self.journal.as_mut() else {
            return;
        };
        journal.record(tick, operation);
    }

    pub(super) fn capture_action(
        &mut self,
        tick: Tick,
        seat: SeatId,
        epoch: u64,
        action: MatchAction,
    ) {
        if self.journal.is_none() {
            return;
        }
        let action = match action {
            MatchAction::Ready(value) => ReplayAction::Ready(value),
            MatchAction::ReadyChecked {
                ready,
                settings_revision,
            } => ReplayAction::ReadyChecked {
                ready,
                settings_revision,
            },
            MatchAction::UpdateSettings {
                settings_revision,
                settings,
            } => ReplayAction::UpdateSettings {
                settings_revision,
                settings,
            },
            MatchAction::Movement(value) => ReplayAction::Movement(value),
            MatchAction::Bomb(value) => ReplayAction::Bomb(value),
            MatchAction::Equipment(value) => ReplayAction::Equipment(value),
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
        if self.journal.is_none() {
            return;
        }
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
            self.journal
                .as_mut()
                .expect("recording was checked above")
                .mark_round_construction_incomplete();
        }
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn replay_journal(&self) -> &Journal {
        self.journal
            .as_ref()
            .expect("runtime recording is disabled")
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn replay_journal_mut(&mut self) -> &mut Journal {
        self.journal
            .as_mut()
            .expect("runtime recording is disabled")
    }

    #[cfg(test)]
    pub(in crate::game::rooms) const fn recording_enabled_for_test(&self) -> bool {
        self.journal.is_some()
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn new_without_replay(
        identity: RoomIdentity,
        round_seed_source: RoundSeedSource,
        settings: grenadiers_core::room::RoomSettings,
        settings_revision: u32,
        settings_editor: Option<SeatId>,
    ) -> Self {
        Self::with_configuration(
            identity,
            round_seed_source,
            None,
            settings,
            settings_revision,
            settings_editor,
        )
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
