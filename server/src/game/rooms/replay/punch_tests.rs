use grenadiers_core::game::{Direction, EnabledItems, SeatId};

use super::{Operation, ReplayAction, ReplayError, replay};
use crate::game::rooms::match_runtime::{MatchAction, start_optional_runtime};

#[test]
fn equipment_replays_with_the_effective_punch_setting_and_shared_id() {
    let mut runtime = start_optional_runtime(EnabledItems::PUNCH, 31);
    runtime
        .action(SeatId::One, 1, MatchAction::Equipment(7))
        .unwrap();
    runtime.tick().unwrap();
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    let update = runtime
        .replay_journal_mut()
        .records
        .iter_mut()
        .find_map(|record| match &mut record.operation {
            Operation::Action {
                action: ReplayAction::UpdateSettings { settings, .. },
                ..
            } => Some(settings),
            _ => None,
        })
        .unwrap();
    *update = update.with_enabled_items(EnabledItems::BRACELET);
    assert_eq!(
        replay(runtime.replay_journal()),
        Err(ReplayError::InvalidActionProfile)
    );
}

#[test]
fn bracelet_settings_replay_as_a_passive_movement_profile() {
    let mut runtime = start_optional_runtime(EnabledItems::BRACELET, 31);
    runtime
        .action(SeatId::One, 1, MatchAction::Movement(Some(Direction::East)))
        .unwrap();
    runtime.tick().unwrap();
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}

#[test]
fn archived_rules_and_invalid_equipment_ids_are_rejected() {
    let mut legacy = super::Journal::new(super::Header::m6([false; 2]));
    legacy.record(
        grenadiers_core::game::Tick(0),
        Operation::Action {
            seat: SeatId::One,
            epoch: 1,
            action: ReplayAction::Equipment(1),
        },
    );
    assert_eq!(replay(&legacy), Err(ReplayError::InvalidActionProfile));

    let mut invalid = super::Journal::new(super::Header::expanded_v5());
    invalid.header.initial_settings = invalid
        .header
        .initial_settings
        .with_enabled_items(EnabledItems::PUNCH);
    invalid.record(
        grenadiers_core::game::Tick(0),
        Operation::Action {
            seat: SeatId::One,
            epoch: 1,
            action: ReplayAction::Equipment(0),
        },
    );
    assert_eq!(replay(&invalid), Err(ReplayError::InvalidBounds));
}
