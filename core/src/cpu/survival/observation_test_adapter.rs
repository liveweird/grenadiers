use crate::cpu::ObservedRespawnLairs;
use crate::game::{LairStatusView, State};
use crate::room::{RespawnLairConfig, RespawnLairSettings};

pub(super) fn projected_lair_statuses(state: &State) -> Vec<LairStatusView> {
    state
        .lair_statuses()
        .into_iter()
        .flatten()
        .map(|mut lair| {
            // The wire projection publishes overdue blocked births as zero
            // remaining ticks, reconstructing them at the current tick.
            if lair.next_spawn_at.is_some_and(|at| at < state.tick()) {
                lair.next_spawn_at = Some(state.tick());
            }
            lair
        })
        .collect()
}

pub(super) fn projected_respawn_lairs(
    state: &State,
    settings: RespawnLairSettings,
) -> Option<ObservedRespawnLairs> {
    state.lair_statuses().map(|_| {
        ObservedRespawnLairs::new(
            RespawnLairConfig::Enabled(settings),
            projected_lair_statuses(state),
            state.mobs(),
        )
    })
}
