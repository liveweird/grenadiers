use grenadiers_core::{
    game::{GenerationRecord, SeatId, State, StepReport},
    protocol::v19,
    room::{
        CpuDifficulty, MatchPhase, RosterMatchLifecycle, RosterParticipant, SeatControl,
        SpikeRoomConfiguration,
    },
};

use super::{
    error::RosterRuntimeError,
    publication,
    runtime::EnvironmentProfile,
    types::{RosterIdentity, RosterNotice, RosterPublication, RosterSeatFrames, wire_seat_v19},
};

pub(super) fn room_view(
    identity: &RosterIdentity,
    lifecycle: &RosterMatchLifecycle,
    profile: EnvironmentProfile,
    revision: u32,
    settings_revision: u32,
    settings_locked: bool,
) -> Result<v19::RoomView, RosterRuntimeError> {
    let EnvironmentProfile::SpikeV10 {
        cloud,
        lairs,
        spike_pits,
    } = profile
    else {
        return Err(RosterRuntimeError::UnsupportedConfiguration);
    };
    let base = grenadiers_core::room::LairRoomConfiguration::new(
        identity.configuration,
        cloud,
        lairs.as_legacy(),
    );
    let configuration = SpikeRoomConfiguration::new(base, spike_pits);
    let room = v19::RoomView {
        settings: v19::RoomSettings::new(
            publication::settings(lifecycle.settings())?,
            environment(cloud, lairs, spike_pits),
        ),
        settings_revision: publication::counter(settings_revision)?,
        settings_editor: (!settings_locked)
            .then(|| lifecycle.settings_editor())
            .flatten()
            .map(wire_seat_v19),
        available_items: publication::available_items()?,
        configuration: v19::RoomConfiguration::from_domain(configuration)
            .map_err(|_| RosterRuntimeError::Publication)?,
        room_id: identity.id.clone(),
        room_code: identity.code.clone(),
        round_id: publication::counter64(lifecycle.round_attempt())?,
        revision: publication::counter(revision)?,
        tick: publication::tick(lifecycle.tick())?,
        rules_version: v19::RulesVersion::ARENA_V10,
        map_id: v19::MapId::GeneratedV2,
        round_number: v19::RoundCount::new(lifecycle.actual_round_number())
            .map_err(|_| RosterRuntimeError::CapacityReached)?,
        scores: publication::scores(lifecycle)?,
        last_round: lifecycle
            .latest_completed()
            .map(publication::completed)
            .transpose()?,
        seats: seats(lifecycle)?,
        phase: phase(lifecycle.phase())?,
    };
    v19::validate_room_view(&room).map_err(|_| RosterRuntimeError::Publication)?;
    Ok(room)
}

pub(super) fn snapshot(
    room: v19::RoomView,
    epoch: u64,
    state: &State,
    reports: &[StepReport],
    generation: GenerationRecord,
) -> Result<RosterNotice, RosterRuntimeError> {
    let capture =
        v19::capture_expanded(state, reports).map_err(|_| RosterRuntimeError::Publication)?;
    let publication =
        v19::bind_expanded(&room, publication::counter64(epoch)?, &capture, generation)
            .map_err(|_| RosterRuntimeError::Publication)?;
    let encoded =
        v19::prepare_snapshots(&publication).map_err(|_| RosterRuntimeError::Publication)?;
    let frames = RosterSeatFrames::new_v19(encoded).ok_or(RosterRuntimeError::Publication)?;
    Ok(RosterNotice {
        publication: RosterPublication::SpikeV19(room, Some(std::sync::Arc::new(publication))),
        frames: Some(frames),
    })
}

pub(super) fn view_notice(room: v19::RoomView) -> RosterNotice {
    RosterNotice {
        publication: RosterPublication::SpikeV19(room, None),
        frames: None,
    }
}

fn seats(lifecycle: &RosterMatchLifecycle) -> Result<Vec<v19::SeatStatus>, RosterRuntimeError> {
    lifecycle
        .participants()
        .iter()
        .enumerate()
        .map(|(index, &participant)| {
            let seat = SeatId::ALL[index];
            let controller = match lifecycle.configuration().controller(seat) {
                Some(SeatControl::Human) => v19::Controller::Human,
                Some(SeatControl::Cpu) => v19::Controller::Cpu {
                    difficulty: match lifecycle.configuration().cpu_difficulty() {
                        Some(CpuDifficulty::Easy) => v19::Difficulty::Easy,
                        Some(CpuDifficulty::Standard) => v19::Difficulty::Standard,
                        Some(CpuDifficulty::Hard) => v19::Difficulty::Hard,
                        None => return Err(RosterRuntimeError::Publication),
                    },
                },
                None => return Err(RosterRuntimeError::Publication),
            };
            let connection = match participant {
                RosterParticipant::NeverAdmitted => v19::ConnectionStatus::Vacant,
                RosterParticipant::Connected => v19::ConnectionStatus::Connected,
                RosterParticipant::Reserved { expires_at } => v19::ConnectionStatus::Disconnected {
                    expires_at: publication::tick(expires_at)?,
                },
                RosterParticipant::Departed => v19::ConnectionStatus::Expired,
                RosterParticipant::Cpu => v19::ConnectionStatus::ServerControlled,
            };
            Ok(v19::SeatStatus {
                seat: wire_seat_v19(seat),
                occupied: !matches!(participant, RosterParticipant::NeverAdmitted),
                ready: lifecycle.ready(seat).map_err(super::error::lifecycle)?,
                rematch_ready: lifecycle
                    .rematch_vote(seat)
                    .map_err(super::error::lifecycle)?,
                controller,
                connection,
            })
        })
        .collect()
}

pub(super) const fn environment(
    cloud: grenadiers_core::game::StormCloudConfig,
    lairs: grenadiers_core::room::RespawnLairTuning,
    spike_pits: grenadiers_core::room::SpikePitTuning,
) -> v19::Environment {
    v19::Environment {
        storm_cloud: matches!(cloud, grenadiers_core::game::StormCloudConfig::Single),
        lairs: v19::Lairs {
            enabled: lairs.enabled(),
            spawn_seconds: lairs.settings().spawn_seconds(),
            max_living_mobs: 1,
        },
        spike_pits: v19::SpikePits {
            enabled: spike_pits.enabled(),
            warning_ticks: spike_pits.settings().warning_ticks() as u16,
            raised_ticks: spike_pits.settings().raised_ticks() as u16,
        },
    }
}

fn phase(value: MatchPhase) -> Result<v19::RoomPhase, RosterRuntimeError> {
    super::publication_v18::phase(value)
}
