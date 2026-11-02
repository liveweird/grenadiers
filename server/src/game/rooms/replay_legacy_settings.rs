use grenadiers_core::room::{BombCapacityDropMultiplier, RoomSettings};

use super::replay_v8;

pub(super) fn settings_are_legacy(settings: RoomSettings) -> bool {
    settings.bomb_capacity_drop_multiplier() == BombCapacityDropMultiplier::LEGACY
}

pub(super) fn body_is_legacy(body: &replay_v8::Journal) -> bool {
    settings_are_legacy(body.header.initial_settings)
        && body.records.iter().all(|record| match record.operation {
            replay_v8::record::Operation::Action {
                action:
                    replay_v8::ReplayAction::UpdateSettings { settings, .. }
                    | replay_v8::ReplayAction::UpdateSpikeSettings { settings, .. },
                ..
            } => settings_are_legacy(settings),
            _ => true,
        })
}
