use crate::game::{EnabledItems, PlayerCount, SeatId, SpeedLevel, StormCloudConfig};
use crate::protocol::{Counter, RoomCode, RoomId, Tick};
use crate::room::{
    BoardSize, CountdownKind, CpuDifficulty, CpuProfile, LairRoomConfiguration, LairSettings,
    LairTuning, MatchPhase, RoomConfiguration as DomainConfiguration,
    RoomSettings as DomainSettings, RosterMatchLifecycle, RosterParticipant, SeatControl,
};

use super::*;

fn counter64(value: u64) -> Counter {
    Counter::new(u32::try_from(value).unwrap()).unwrap()
}

fn seat(value: SeatId) -> Seat {
    match value {
        SeatId::One => Seat::One,
        SeatId::Two => Seat::Two,
        SeatId::Three => Seat::Three,
        SeatId::Four => Seat::Four,
    }
}

fn controls(count: PlayerCount) -> Vec<SeatControl> {
    match count {
        PlayerCount::Two => vec![SeatControl::Human, SeatControl::Cpu],
        PlayerCount::Three => vec![SeatControl::Human, SeatControl::Cpu, SeatControl::Human],
        PlayerCount::Four => vec![
            SeatControl::Human,
            SeatControl::Cpu,
            SeatControl::Human,
            SeatControl::Cpu,
        ],
    }
}

fn difficulty(count: PlayerCount) -> CpuDifficulty {
    if count == PlayerCount::Three {
        CpuProfile::ENVIRONMENT_EASY.difficulty
    } else {
        CpuProfile::ENVIRONMENT_STANDARD.difficulty
    }
}

pub(super) fn lifecycle(
    count: PlayerCount,
    board: BoardSize,
    enabled: EnabledItems,
    running: bool,
) -> RosterMatchLifecycle {
    let controls = controls(count);
    let configuration =
        DomainConfiguration::new(&controls, Some(difficulty(count)), board).unwrap();
    configured_lifecycle(configuration, enabled, running)
}

pub(super) fn human_lifecycle(
    board: BoardSize,
    enabled: EnabledItems,
    running: bool,
) -> RosterMatchLifecycle {
    let configuration =
        DomainConfiguration::new(&[SeatControl::Human, SeatControl::Human], None, board).unwrap();
    configured_lifecycle(configuration, enabled, running)
}

fn configured_lifecycle(
    configuration: DomainConfiguration,
    enabled: EnabledItems,
    running: bool,
) -> RosterMatchLifecycle {
    let settings = DomainSettings::new(SpeedLevel::Base, 256, 3, 180)
        .unwrap()
        .with_enabled_items(enabled);
    let mut lifecycle = RosterMatchLifecycle::new(configuration, settings);
    for seat in configuration.human_seats() {
        lifecycle.occupy(seat).unwrap();
    }
    if running {
        let humans = configuration.human_seats().collect::<Vec<_>>();
        for seat in humans {
            lifecycle.set_ready(seat, true).unwrap();
        }
        for _ in 0..crate::room::COUNTDOWN_TICKS {
            lifecycle.advance_idle_tick().unwrap();
        }
        assert_eq!(lifecycle.phase(), MatchPhase::Running);
    }
    lifecycle
}

pub(super) fn room_view(
    lifecycle: &RosterMatchLifecycle,
    cloud: StormCloudConfig,
    settings_locked: bool,
) -> RoomView {
    room_view_with_tuning(
        lifecycle,
        cloud,
        LairTuning::new(true, LairSettings::DEFAULT),
        settings_locked,
    )
}

pub(super) fn room_view_with_tuning(
    lifecycle: &RosterMatchLifecycle,
    cloud: StormCloudConfig,
    tuning: LairTuning,
    settings_locked: bool,
) -> RoomView {
    let environment = Environment {
        storm_cloud: cloud == StormCloudConfig::Single,
        lairs: Lairs {
            enabled: tuning.enabled(),
            spawn_seconds: tuning.settings().spawn_seconds(),
            max_living_mobs: tuning.settings().mobs_per_lair(),
        },
    };
    let settings = lifecycle.settings();
    let room = RoomView {
        settings: RoomSettings::new(
            GameSettings::new(
                settings.starting_speed() as u8,
                settings.turn_assist_units(),
                settings.wins_to_finish(),
                u16::try_from(settings.bomb_fuse_ticks()).unwrap(),
                ItemSet::from_mask(ItemMask::new(settings.enabled_items().bits()).unwrap()),
            )
            .unwrap(),
            environment,
        ),
        settings_revision: Counter::new(1).unwrap(),
        settings_editor: (!settings_locked)
            .then(|| lifecycle.settings_editor())
            .flatten()
            .map(seat),
        available_items: ItemSet::from_mask(ItemMask::new(EnabledItems::ALL.bits()).unwrap()),
        configuration: RoomConfiguration::from_domain(LairRoomConfiguration::new(
            lifecycle.configuration(),
            cloud,
            tuning,
        )),
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        room_code: RoomCode::new("ABCDEFGH").unwrap(),
        round_id: counter64(lifecycle.round_attempt()),
        revision: Counter::new(1).unwrap(),
        tick: Tick::new(lifecycle.tick().0).unwrap(),
        rules_version: RulesVersion::ENVIRONMENT_V8,
        map_id: MapId::GeneratedV2,
        round_number: RoundCount::new(lifecycle.actual_round_number()).unwrap(),
        scores: lifecycle
            .scores()
            .iter()
            .enumerate()
            .map(|(index, &wins)| Score {
                seat: seat(SeatId::ALL[index]),
                wins: Wins::new(wins).unwrap(),
            })
            .collect(),
        last_round: lifecycle.latest_completed().map(|last| CompletedRound {
            round_id: counter64(last.epoch()),
            round_number: Counter::new(last.assignment().round_number().get()).unwrap(),
            ended_at: Tick::new(last.ended_at().0).unwrap(),
            result: match last.result() {
                crate::game::RoundResult::Winner(winner) => {
                    RoundResult::Winner { seat: seat(winner) }
                }
                crate::game::RoundResult::Draw(reason) => RoundResult::Draw {
                    reason: match reason {
                        crate::game::DrawReason::MutualElimination => DrawReason::MutualElimination,
                        crate::game::DrawReason::Timeout => DrawReason::Timeout,
                    },
                },
            },
        }),
        seats: lifecycle
            .participants()
            .iter()
            .enumerate()
            .map(|(index, &participant)| status(lifecycle, SeatId::ALL[index], participant))
            .collect(),
        phase: phase(lifecycle.phase()),
    };
    super::room_validation::validate_room_view(&room).unwrap();
    room
}

fn status(
    lifecycle: &RosterMatchLifecycle,
    domain_seat: SeatId,
    participant: RosterParticipant,
) -> SeatStatus {
    let controller = match lifecycle.configuration().controller(domain_seat).unwrap() {
        SeatControl::Human => Controller::Human,
        SeatControl::Cpu => Controller::Cpu {
            difficulty: match lifecycle.configuration().cpu_difficulty().unwrap() {
                CpuDifficulty::Easy => Difficulty::Easy,
                CpuDifficulty::Standard => Difficulty::Standard,
            },
        },
    };
    let connection = match participant {
        RosterParticipant::NeverAdmitted => ConnectionStatus::Vacant,
        RosterParticipant::Connected => ConnectionStatus::Connected,
        RosterParticipant::Reserved { expires_at } => ConnectionStatus::Disconnected {
            expires_at: Tick::new(expires_at.0).unwrap(),
        },
        RosterParticipant::Departed => ConnectionStatus::Expired,
        RosterParticipant::Cpu => ConnectionStatus::ServerControlled,
    };
    SeatStatus {
        seat: seat(domain_seat),
        occupied: participant != RosterParticipant::NeverAdmitted,
        ready: lifecycle.ready(domain_seat).unwrap(),
        rematch_ready: lifecycle.rematch_vote(domain_seat).unwrap(),
        controller,
        connection,
    }
}

fn phase(value: MatchPhase) -> RoomPhase {
    match value {
        MatchPhase::Waiting => RoomPhase::Waiting,
        MatchPhase::Countdown { kind, ends_at } => RoomPhase::Countdown {
            kind: match kind {
                CountdownKind::Initial => super::CountdownKind::Initial,
                CountdownKind::InterRound => super::CountdownKind::InterRound,
            },
            ends_at: Tick::new(ends_at.0).unwrap(),
        },
        MatchPhase::Running => RoomPhase::Running,
        MatchPhase::Result { next_countdown_at } => RoomPhase::Result {
            hold_until: Tick::new(next_countdown_at.0).unwrap(),
        },
        MatchPhase::Interrupted { ended_at, reason } => RoomPhase::Interrupted {
            ended_at: Tick::new(ended_at.0).unwrap(),
            reason: match reason {
                crate::room::InterruptReason::ParticipantLeft
                | crate::room::InterruptReason::ConnectionLost => {
                    MatchInterruptReason::ConnectionLost
                }
                crate::room::InterruptReason::ServerOverloaded => {
                    MatchInterruptReason::ServerOverloaded
                }
                crate::room::InterruptReason::ServerStopped => MatchInterruptReason::ServerStopped,
                crate::room::InterruptReason::InternalError => MatchInterruptReason::InternalError,
            },
        },
        MatchPhase::MatchResult {
            outcome,
            rematch_at,
        } => RoomPhase::MatchResult {
            ended_at: Tick::new(outcome.ended_at().0).unwrap(),
            result: match outcome.result() {
                crate::room::series::MatchResult::ScoreWinner(winner) => MatchResult::Winner {
                    seat: seat(winner),
                    reason: MatchWinnerReason::RoundWins,
                },
                crate::room::series::MatchResult::ForfeitWinner(winner) => MatchResult::Winner {
                    seat: seat(winner),
                    reason: MatchWinnerReason::Forfeit,
                },
                crate::room::series::MatchResult::Abandoned => MatchResult::Abandoned,
                crate::room::series::MatchResult::Interrupted(reason) => MatchResult::Interrupted {
                    reason: match reason {
                        crate::room::InterruptReason::ParticipantLeft
                        | crate::room::InterruptReason::ConnectionLost => {
                            MatchInterruptReason::ConnectionLost
                        }
                        crate::room::InterruptReason::ServerOverloaded => {
                            MatchInterruptReason::ServerOverloaded
                        }
                        crate::room::InterruptReason::ServerStopped => {
                            MatchInterruptReason::ServerStopped
                        }
                        crate::room::InterruptReason::InternalError => {
                            MatchInterruptReason::InternalError
                        }
                    },
                },
            },
            rematch_at: Tick::new(rematch_at.0).unwrap(),
        },
    }
}
