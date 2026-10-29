use crate::game::{
    ActorInit, ArenaGenerationConfig, EnabledItems, PlayerCount, Position, RoundAssignment,
    RoundResult, RulesConfig, SeatId, SimulationSeed, SpeedLevel, State, StormCloudConfig, World,
    generate_variable_multiplayer_arena,
};
use crate::protocol::Counter;
use crate::room::{BoardSize, DepartureReason, InterruptReason, RosterMatchLifecycle};

use super::{
    AdmissionProjection, PersonalAdmission, ResumeToken, RoomView, Seat, capture_expanded,
    prepare_admission, server_conformance_lifecycle as support,
};

type RoomCase = (&'static str, &'static str, RoomView);

fn advance(lifecycle: &mut RosterMatchLifecycle, ticks: u64) {
    for _ in 0..ticks {
        lifecycle.advance_idle_tick().unwrap();
    }
}

fn finish_round(lifecycle: &mut RosterMatchLifecycle, winner: SeatId) {
    lifecycle
        .complete_current_tick(RoundResult::Winner(winner))
        .unwrap();
}

fn score_winner(count: PlayerCount, winner: SeatId) -> RosterMatchLifecycle {
    let mut lifecycle =
        support::lifecycle(count, BoardSize::ElevenByNine, EnabledItems::EMPTY, true);
    for round in 0..3 {
        finish_round(&mut lifecycle, winner);
        if round < 2 {
            advance(&mut lifecycle, crate::room::COUNTDOWN_TICKS * 2);
        }
    }
    lifecycle
}

fn case(
    id: &'static str,
    scenario: &'static str,
    lifecycle: &RosterMatchLifecycle,
    settings_locked: bool,
) -> RoomCase {
    (
        id,
        scenario,
        support::room_view(lifecycle, StormCloudConfig::Disabled, settings_locked),
    )
}

pub(super) fn room_cases() -> Vec<RoomCase> {
    let mut initial = support::lifecycle(
        PlayerCount::Two,
        BoardSize::ElevenByNine,
        EnabledItems::EMPTY,
        false,
    );
    initial.set_ready(SeatId::One, true).unwrap();

    let mut result = support::lifecycle(
        PlayerCount::Two,
        BoardSize::ElevenByNine,
        EnabledItems::EMPTY,
        true,
    );
    finish_round(&mut result, SeatId::One);
    let mut inter_round = result.clone();
    advance(&mut inter_round, crate::room::COUNTDOWN_TICKS);
    let mut next_running = inter_round.clone();
    advance(&mut next_running, crate::room::COUNTDOWN_TICKS);

    let terminal = score_winner(PlayerCount::Two, SeatId::One);

    let mut rematch = score_winner(PlayerCount::Three, SeatId::One);
    advance(&mut rematch, crate::room::COUNTDOWN_TICKS);
    rematch.set_rematch_vote(SeatId::One, true).unwrap();

    let mut forfeit = support::human_lifecycle(BoardSize::ElevenByNine, EnabledItems::EMPTY, true);
    forfeit
        .depart_with_grace([Some(DepartureReason::Explicit), None, None, None])
        .unwrap();

    let mut interrupted =
        support::human_lifecycle(BoardSize::ElevenByNine, EnabledItems::EMPTY, false);
    interrupted
        .interrupt(InterruptReason::InternalError)
        .unwrap();

    let mut disconnected = support::lifecycle(
        PlayerCount::Two,
        BoardSize::ElevenByNine,
        EnabledItems::EMPTY,
        true,
    );
    disconnected
        .depart_with_grace([Some(DepartureReason::TransportLoss), None, None, None])
        .unwrap();

    let mut abandoned = support::lifecycle(
        PlayerCount::Two,
        BoardSize::ElevenByNine,
        EnabledItems::EMPTY,
        true,
    );
    abandoned
        .depart_with_grace([Some(DepartureReason::Explicit), None, None, None])
        .unwrap();

    vec![
        case(
            "lifecycle-initial-countdown",
            "initialCountdown",
            &initial,
            false,
        ),
        case("lifecycle-round-result", "roundResult", &result, true),
        case(
            "lifecycle-inter-round",
            "interRoundCountdown",
            &inter_round,
            true,
        ),
        case("lifecycle-next-running", "nextRunning", &next_running, true),
        case("lifecycle-score-winner", "scoreWinner", &terminal, true),
        case("lifecycle-forfeit", "forfeit", &forfeit, true),
        case("lifecycle-interrupted", "interrupted", &interrupted, false),
        case(
            "lifecycle-rematch-eligible",
            "rematchEligible",
            &rematch,
            true,
        ),
        case(
            "lifecycle-disconnected",
            "disconnected",
            &disconnected,
            true,
        ),
        case("lifecycle-abandoned", "abandoned", &abandoned, true),
    ]
}

fn physical_state(
    tick: crate::game::Tick,
    assignment: RoundAssignment,
) -> (State, crate::game::GenerationRecord) {
    let generated = generate_variable_multiplayer_arena(
        [29; 32],
        EnabledItems::EMPTY,
        ArenaGenerationConfig {
            player_count: PlayerCount::Two,
            board: BoardSize::ElevenByNine,
            assignment,
        },
    )
    .unwrap();
    let record = generated.record();
    let (world, _) = World::from_generated(generated).unwrap();
    let actors: Vec<_> = world
        .arena()
        .spawns()
        .iter()
        .enumerate()
        .map(|(index, &cell)| ActorInit {
            id: crate::game::EntityId(40 + index as u32),
            seat: SeatId::ALL[index],
            position: Position::center(cell),
            heading: crate::game::Direction::East,
            speed: SpeedLevel::Base,
        })
        .collect();
    let state = State::new_at_tick_with_assignment_and_seed(
        RulesConfig::environment_v7(
            crate::room::RoomSettings::DEFAULT,
            StormCloudConfig::Disabled,
        ),
        world,
        actors,
        tick,
        assignment,
        SimulationSeed::new([29; 32]),
    )
    .unwrap();
    (state, record)
}

fn admission(
    lifecycle: &RosterMatchLifecycle,
    state: &State,
    generation: crate::game::GenerationRecord,
    recipient: Seat,
) -> String {
    let room = support::room_view(lifecycle, StormCloudConfig::Disabled, true);
    let capture = capture_expanded(state, &[]).unwrap();
    prepare_admission(
        AdmissionProjection {
            room: &room,
            state_epoch: Some(room.round_id),
            capture: Some(&capture),
            generation: Some(generation),
        },
        PersonalAdmission {
            recipient,
            last_action_id: Some(Counter::new(1).unwrap()),
            sequence: Counter::new(1).unwrap(),
            resume_token: ResumeToken::new("d".repeat(64)).unwrap(),
        },
    )
    .unwrap()
}

pub(super) fn admission_cases() -> Vec<(&'static str, &'static str, Seat, String)> {
    let mut result = support::human_lifecycle(BoardSize::ElevenByNine, EnabledItems::EMPTY, true);
    let (mut result_state, result_generation) =
        physical_state(result.tick(), RoundAssignment::first());
    result_state
        .eliminate_departed([true, false, false, false])
        .unwrap();
    finish_round(&mut result, SeatId::Two);

    let mut forfeit = support::human_lifecycle(BoardSize::ElevenByNine, EnabledItems::EMPTY, true);
    let (forfeit_state, forfeit_generation) =
        physical_state(forfeit.tick(), RoundAssignment::first());
    forfeit
        .depart_with_grace([Some(DepartureReason::Explicit), None, None, None])
        .unwrap();

    let terminal = score_winner(PlayerCount::Two, SeatId::Two);
    let assignment = RoundAssignment::new(std::num::NonZeroU32::new(3).unwrap());
    let (mut terminal_state, terminal_generation) = physical_state(terminal.tick(), assignment);
    terminal_state
        .eliminate_departed([true, false, false, false])
        .unwrap();

    let mut continued_forfeit =
        support::human_lifecycle(BoardSize::ElevenByNine, EnabledItems::EMPTY, true);
    finish_round(&mut continued_forfeit, SeatId::One);
    advance(&mut continued_forfeit, crate::room::COUNTDOWN_TICKS * 2);
    let second = RoundAssignment::new(std::num::NonZeroU32::new(2).unwrap());
    let (continued_state, continued_generation) = physical_state(continued_forfeit.tick(), second);
    continued_forfeit
        .depart_with_grace([Some(DepartureReason::Explicit), None, None, None])
        .unwrap();

    vec![
        (
            "lifecycle-result-admission",
            "resultAdmission",
            Seat::One,
            admission(&result, &result_state, result_generation, Seat::One),
        ),
        (
            "lifecycle-forfeit-admission",
            "forfeitAdmission",
            Seat::Two,
            admission(&forfeit, &forfeit_state, forfeit_generation, Seat::Two),
        ),
        (
            "lifecycle-score-winner-admission",
            "scoreWinnerAdmission",
            Seat::One,
            admission(&terminal, &terminal_state, terminal_generation, Seat::One),
        ),
        (
            "lifecycle-continued-forfeit-admission",
            "continuedForfeitAdmission",
            Seat::Two,
            admission(
                &continued_forfeit,
                &continued_state,
                continued_generation,
                Seat::Two,
            ),
        ),
    ]
}
