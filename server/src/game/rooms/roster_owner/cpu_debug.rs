use grenadiers_core::{
    cpu::{
        WorldId,
        policy::{
            IntentDebugSummary, StrategicObjectiveKind, StrategicPhase, StrategicSwitchReason,
            StrategicTarget,
        },
    },
    game::SeatId,
    protocol::{Cell, v14::ResumeToken, v18, v19, v20},
    room::RosterParticipant,
};

use super::driver::Driver;
use crate::cpu_debug::{
    CpuDebugQueryError, CpuDebugResponse, CpuDebugRow, DebugObjective, DebugPhase,
    DebugSwitchReason, DebugTarget, debug_cell, debug_pickup, debug_seat,
};

impl Driver {
    pub(super) fn cpu_debug(
        &self,
        token: &ResumeToken,
    ) -> Result<CpuDebugResponse, CpuDebugQueryError> {
        let mut matched = None;
        for (index, slot) in self.seats.iter().enumerate() {
            let valid = slot
                .as_ref()
                .is_some_and(|slot| slot.credential.matches(token));
            let active = matches!(
                self.runtime
                    .lifecycle()
                    .participant(SeatId::ALL[index])
                    .ok(),
                Some(RosterParticipant::Connected)
            );
            if valid && active {
                matched = Some(SeatId::ALL[index]);
            }
        }
        let seat = matched.ok_or(CpuDebugQueryError::Unauthorized)?;

        let room = self
            .last
            .spike_v20_room()
            .map(|room| (&room.room_id, room.round_id, room.tick, room.revision))
            .or_else(|| {
                self.last
                    .spike_v19_room()
                    .map(|room| (&room.room_id, room.round_id, room.tick, room.revision))
            })
            .or_else(|| {
                self.last
                    .spike_room()
                    .map(|room| (&room.room_id, room.round_id, room.tick, room.revision))
            })
            .ok_or(CpuDebugQueryError::Unavailable)?;
        let unavailable_response = || CpuDebugResponse {
            protocol_version: 1,
            room_id: room.0.clone(),
            round_id: u64::from(room.1.get()),
            tick: room.2.get(),
            revision: room.3.get(),
            you_seat: debug_seat(seat),
            cpus: self
                .cpus
                .iter()
                .take(3)
                .map(|cpu| CpuDebugRow::Unavailable {
                    seat: debug_seat(cpu.seat()),
                })
                .collect(),
        };
        let recipient = wire_seat(seat);
        let state = if let Some(publication) = self.last.spike_v20_cpu_publication() {
            v20::project_recipient(publication, recipient)
                .map_err(|_| CpuDebugQueryError::Unavailable)?
                .state
        } else if let Some(publication) = self.last.spike_v19_cpu_publication() {
            v19::project_recipient(publication, recipient)
                .map_err(|_| CpuDebugQueryError::Unavailable)?
                .state
        } else if let Some(publication) = self.last.spike_cpu_publication() {
            v18::project_recipient(publication, recipient)
                .map_err(|_| CpuDebugQueryError::Unavailable)?
                .state
        } else {
            return Ok(unavailable_response());
        };
        let bomb_blind = state
            .actors
            .get(seat.index())
            .is_none_or(recipient_bomb_blind);
        let world = WorldId {
            room_instance: self.runtime.identity().registry_serial,
            physical_round: self.runtime.lifecycle().round_attempt(),
        };
        let cpus = self
            .cpus
            .iter()
            .take(3)
            .map(|cpu| {
                let unavailable = || CpuDebugRow::Unavailable {
                    seat: debug_seat(cpu.seat()),
                };
                if bomb_blind || !actor_visible(&state, cpu.seat()) {
                    return unavailable();
                }
                let Some(summary) = cpu.intent_debug_summary_for(world) else {
                    return unavailable();
                };
                available_row(summary, &state).unwrap_or_else(unavailable)
            })
            .collect();
        Ok(CpuDebugResponse {
            protocol_version: 1,
            room_id: room.0.clone(),
            round_id: u64::from(room.1.get()),
            tick: room.2.get(),
            revision: room.3.get(),
            you_seat: debug_seat(seat),
            cpus,
        })
    }
}

pub(super) fn available_row(
    summary: IntentDebugSummary,
    state: &v18::GameState,
) -> Option<CpuDebugRow> {
    let target = match summary.target {
        StrategicTarget::Pickup { cell, kind } => {
            let cell = debug_cell(cell)?;
            let kind = debug_pickup(kind);
            state
                .pickups
                .iter()
                .any(|pickup| pickup.cell == cell && wire_pickup(pickup.kind) == kind)
                .then_some(DebugTarget::Pickup { cell, kind })?
                .into()
        }
        StrategicTarget::Block { cell } => {
            let cell = debug_cell(cell)?;
            state
                .arena
                .blocks
                .contains(&cell)
                .then_some(DebugTarget::Block { cell })?
                .into()
        }
        StrategicTarget::Opponent { seat } => actor_visible(state, seat)
            .then_some(DebugTarget::Opponent {
                seat: debug_seat(seat),
            })?
            .into(),
        StrategicTarget::Tile { cell } => {
            let cell = debug_cell(cell)?;
            cell_visible(state, cell)
                .then_some(DebugTarget::Tile { cell })?
                .into()
        }
        StrategicTarget::None => None,
    };
    if matches!(
        summary.objective,
        StrategicObjectiveKind::Survive | StrategicObjectiveKind::Wait
    ) && target.is_some()
    {
        return None;
    }
    let approach = match summary.approach_cell {
        Some(cell) => Some(debug_cell(cell)?),
        None => None,
    };
    if approach.is_some_and(|cell| !cell_visible(state, cell)) {
        return None;
    }
    Some(CpuDebugRow::Available {
        seat: debug_seat(summary.cpu_seat),
        observed_tick: summary.observation_tick.0,
        objective: match summary.objective {
            StrategicObjectiveKind::Collect => DebugObjective::Collect,
            StrategicObjectiveKind::Farm => DebugObjective::Farm,
            StrategicObjectiveKind::Attack => DebugObjective::Attack,
            StrategicObjectiveKind::Explore => DebugObjective::Explore,
            StrategicObjectiveKind::Survive => DebugObjective::Survive,
            StrategicObjectiveKind::Wait => DebugObjective::Wait,
        },
        target,
        approach,
        phase: match summary.phase {
            StrategicPhase::Approach => DebugPhase::Approach,
            StrategicPhase::Plant => DebugPhase::Plant,
            StrategicPhase::Escape => DebugPhase::Escape,
            StrategicPhase::Reengage => DebugPhase::Reengage,
            StrategicPhase::Evade => DebugPhase::Evade,
            StrategicPhase::Waiting => DebugPhase::Waiting,
        },
        switch_reason: match summary.last_switch_reason {
            StrategicSwitchReason::Initial => DebugSwitchReason::Initial,
            StrategicSwitchReason::Completed => DebugSwitchReason::Completed,
            StrategicSwitchReason::TargetInvalid => DebugSwitchReason::TargetInvalid,
            StrategicSwitchReason::VisibilityLost => DebugSwitchReason::VisibilityLost,
            StrategicSwitchReason::BetterOpportunity => DebugSwitchReason::BetterOpportunity,
            StrategicSwitchReason::Stalled => DebugSwitchReason::Stalled,
            StrategicSwitchReason::NoTargets => DebugSwitchReason::NoTargets,
            StrategicSwitchReason::SafetyInterrupt => DebugSwitchReason::SafetyInterrupt,
            StrategicSwitchReason::Resumed => DebugSwitchReason::Resumed,
        },
    })
}

fn actor_visible(state: &v18::GameState, seat: SeatId) -> bool {
    state
        .actors
        .get(seat.index())
        .is_some_and(|actor| matches!(actor, v18::ActorView::Visible { .. }))
}
pub(super) fn recipient_bomb_blind(actor: &v18::ActorView) -> bool {
    match actor {
        v18::ActorView::Visible { effects, .. } => effects
            .as_slice()
            .iter()
            .any(|effect| effect.kind() == v18::EffectKind::InvisibleBombs),
        v18::ActorView::Hidden { .. } => true,
    }
}
fn cell_visible(state: &v18::GameState, cell: Cell) -> bool {
    cell.column() < state.arena.width && cell.row() < state.arena.height
}
fn wire_seat(seat: SeatId) -> v18::Seat {
    match seat {
        SeatId::One => v18::Seat::One,
        SeatId::Two => v18::Seat::Two,
        SeatId::Three => v18::Seat::Three,
        SeatId::Four => v18::Seat::Four,
    }
}

fn wire_pickup(value: v18::PickupKind) -> crate::cpu_debug::DebugPickupKind {
    use crate::cpu_debug::DebugPickupKind as D;
    use v18::PickupKind as W;
    match value {
        W::BombCapacity => D::BombCapacity,
        W::BlastRange => D::BlastRange,
        W::MovementSpeed => D::MovementSpeed,
        W::Kick => D::Kick,
        W::Punch => D::Punch,
        W::Bracelet => D::Bracelet,
        W::WallPass => D::WallPass,
        W::Heart => D::Heart,
        W::RemoteBomb => D::RemoteBomb,
        W::PiercingBomb => D::PiercingBomb,
        W::PowerBomb => D::PowerBomb,
        W::Landmine => D::Landmine,
        W::Shield => D::Shield,
        W::Stopwatch => D::Stopwatch,
        W::InvisibilityCloak => D::InvisibilityCloak,
        W::GoldenFlame => D::GoldenFlame,
        W::ShortFuse => D::ShortFuse,
        W::Diarrhea => D::Diarrhea,
        W::SlowFoot => D::SlowFoot,
        W::ShortFlame => D::ShortFlame,
        W::LongFuse => D::LongFuse,
        W::ReverseControls => D::ReverseControls,
        W::InvisibleBombs => D::InvisibleBombs,
        W::OilBarrel => D::OilBarrel,
        W::Teleporter => D::Teleporter,
    }
}
