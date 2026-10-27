use std::sync::Arc;

use grenadiers_core::{
    game::{EnabledItems, SeatId, SpeedLevel},
    protocol::{Counter, v12::ConnectionStatus},
    room::{CpuProfile, RoomMode, RoomSettings},
};
use tokio::{sync::watch, time::Instant};

use super::{PreparedMatchOwner, test_support::*};
use crate::{
    game::rooms::{match_runtime::MatchAction, profile::RoomProfile, types::RoomError},
    round_seed::RoundSeedSource,
};

fn prepared_solo() -> (
    super::MatchOwnerHandle,
    PreparedMatchOwner,
    watch::Sender<super::ShutdownState>,
) {
    let (shutdown, shutdown_rx) = watch::channel(false);
    let mut room = identity(RoomProfile::ExpandedV5);
    room.mode = RoomMode::Solo(CpuProfile::CURRENT);
    room.private_cpu_seed = Some([17; 32]);
    PreparedMatchOwner::new(
        room,
        Instant::now(),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(23);
            Ok(())
        }),
        Arc::new(|bytes: &mut [u8]| {
            bytes.fill(31);
            Ok(())
        }),
        shutdown_rx,
    )
    .map(|(handle, owner)| (handle, owner, shutdown))
    .unwrap()
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn solo_creator_is_the_only_network_admission_and_cpu_is_ready() {
    let (handle, owner, shutdown) = prepared_solo();
    let initial = owner.initial.clone();
    assert!(!initial.room.seats[0].occupied);
    assert!(initial.room.seats[1].occupied);
    assert_eq!(
        initial.room.seats[1].connection,
        ConnectionStatus::ServerControlled
    );
    let value = start_prepared(handle, owner, shutdown);

    assert!(matches!(
        value.handle.join(binding(2, 1)).await.unwrap(),
        Err(RoomError::RoomFull)
    ));
    let mut creator = value
        .handle
        .admit_creator(binding(1, 1), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(creator.seat, grenadiers_core::protocol::Seat::One);
    let personal: serde_json::Value = serde_json::from_str(&creator.personal_frame).unwrap();
    assert_eq!(personal["room"]["seats"][1]["ready"], true);
    assert!(matches!(
        creator.room.phase,
        grenadiers_core::protocol::v12::RoomPhase::Waiting
    ));
    let super::MatchOwnerNotice::Publication(baseline) =
        creator.notices.borrow_and_update().clone()
    else {
        panic!("expected creator watch baseline");
    };
    assert!(baseline.room.seats[1].ready);
    assert_eq!(baseline.room, creator.room);
    assert!(publication(&value.notices).room.seats[1].ready);
    value
        .handle
        .action(
            binding(1, 1),
            creator.room_id.clone(),
            creator.round_id,
            MatchAction::ReadyChecked {
                ready: true,
                settings_revision: 1,
            },
        )
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(
        publication(&value.notices).room.phase,
        grenadiers_core::protocol::v12::RoomPhase::Countdown { .. }
    ));
    value.task.abort();
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn failed_solo_personal_frame_rolls_back_both_creator_and_cpu_ready() {
    let (handle, owner, shutdown) = prepared_solo();
    let value = start_prepared(handle, owner, shutdown);
    super::driver::admissions::fail_next_admission_encoding();

    assert!(matches!(
        value
            .handle
            .admit_creator(binding(1, 1), Counter::new(1).unwrap())
            .await
            .unwrap(),
        Err(RoomError::CapacityReached)
    ));
    let creator = value
        .handle
        .admit_creator(binding(2, 1), Counter::new(2).unwrap())
        .await
        .unwrap()
        .unwrap();
    let personal: serde_json::Value = serde_json::from_str(&creator.personal_frame).unwrap();
    assert_eq!(personal["room"]["seats"][0]["occupied"], true);
    assert_eq!(personal["room"]["seats"][1]["ready"], true);
    assert_eq!(creator.room.revision.get(), 3);
    value.task.abort();
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn settings_change_rechecks_cpu_readiness_and_explicit_leave_closes_room() {
    let (handle, owner, shutdown) = prepared_solo();
    let value = start_prepared(handle, owner, shutdown);
    let mut creator = value
        .handle
        .admit_creator(binding(1, 1), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    let settings = RoomSettings::new(SpeedLevel::One, 192, 2, 150)
        .unwrap()
        .with_enabled_items(EnabledItems::ALL);
    value
        .handle
        .action(
            binding(1, 1),
            creator.room_id.clone(),
            creator.round_id,
            MatchAction::UpdateSettings {
                settings_revision: 1,
                settings,
            },
        )
        .await
        .unwrap()
        .unwrap();
    let room = publication(&value.notices).room;
    assert_eq!(room.settings_revision.get(), 2);
    assert!(room.seats[SeatId::Two.index()].ready);

    creator.guard.disarm();
    value
        .handle
        .leave(binding(1, 1), creator.room_id.clone(), creator.round_id)
        .await
        .unwrap()
        .unwrap();
    advance_ticks(1).await;
    value.task.await.unwrap();
}

async fn running_solo() -> (
    Harness,
    super::MatchAdmission,
    crate::game::rooms::BindingKey,
) {
    let (handle, owner, shutdown) = prepared_solo();
    let value = start_prepared(handle, owner, shutdown);
    let human = binding(1, 1);
    let creator = value
        .handle
        .admit_creator(human, Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    value
        .handle
        .action(
            human,
            creator.room_id.clone(),
            creator.round_id,
            MatchAction::ReadyChecked {
                ready: true,
                settings_revision: 1,
            },
        )
        .await
        .unwrap()
        .unwrap();
    advance_ticks(180).await;
    assert_running(&value.notices);
    (value, creator, human)
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn running_transport_loss_keeps_cpu_and_physics_alive_across_resume() {
    let (value, creator, human) = running_solo().await;
    let room_id = creator.room_id.clone();
    let round_id = creator.round_id;
    let token = creator.resume_token.clone();
    drop(creator);
    advance_ticks(12).await;
    let before = publication(&value.notices).room.tick.get();
    let resumed = value
        .handle
        .resume(binding(2, 1), Counter::new(2).unwrap(), room_id, token)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(resumed.seat, grenadiers_core::protocol::Seat::One);
    assert!(resumed.room.tick.get() >= before);
    assert_eq!(
        resumed.room.seats[1].connection,
        ConnectionStatus::ServerControlled
    );
    assert_eq!(
        value
            .handle
            .action(
                human,
                resumed.room_id.clone(),
                round_id,
                MatchAction::Movement(None),
            )
            .await
            .unwrap(),
        Err(RoomError::StaleRoom)
    );
    value.task.abort();
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn grace_expiry_settles_cpu_forfeit_then_destroys_solo_room() {
    let (value, creator, _) = running_solo().await;
    drop(creator);
    advance_ticks(602).await;
    value.task.await.unwrap();
    let room = publication(&value.notices).room;
    assert!(matches!(
        room.phase,
        grenadiers_core::protocol::v12::RoomPhase::MatchResult {
            result: grenadiers_core::protocol::v12::MatchResult::Winner {
                seat: grenadiers_core::protocol::Seat::Two,
                reason: grenadiers_core::protocol::v12::MatchWinnerReason::Forfeit,
            },
            ..
        }
    ));
    assert_eq!(room.seats[0].connection, ConnectionStatus::Expired);
    assert_eq!(room.seats[1].connection, ConnectionStatus::ServerControlled);
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn completed_solo_match_requires_human_rematch_and_resets_cpu_ready() {
    let (handle, owner, shutdown) = prepared_solo();
    let value = start_prepared(handle, owner, shutdown);
    let human = binding(1, 1);
    let creator = value
        .handle
        .admit_creator(human, Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    let settings = RoomSettings::new(SpeedLevel::Base, 256, 1, 60).unwrap();
    value
        .handle
        .action(
            human,
            creator.room_id.clone(),
            creator.round_id,
            MatchAction::UpdateSettings {
                settings_revision: 1,
                settings,
            },
        )
        .await
        .unwrap()
        .unwrap();
    value
        .handle
        .action(
            human,
            creator.room_id.clone(),
            creator.round_id,
            MatchAction::ReadyChecked {
                ready: true,
                settings_revision: 2,
            },
        )
        .await
        .unwrap()
        .unwrap();
    advance_ticks(180).await;
    let running = publication(&value.notices).room;
    value
        .handle
        .action(
            human,
            creator.room_id.clone(),
            u64::from(running.round_id.get()),
            MatchAction::Bomb(1),
        )
        .await
        .unwrap()
        .unwrap();
    let mut rematch_at = None;
    for _ in 0..1_000 {
        advance_ticks(1).await;
        if let grenadiers_core::protocol::v12::RoomPhase::MatchResult { rematch_at: at, .. } =
            publication(&value.notices).room.phase
        {
            rematch_at = Some(at);
            break;
        }
    }
    let rematch_at = rematch_at.expect("solo fixture reaches a final match result");
    let current = publication(&value.notices).room.tick;
    advance_ticks(usize::try_from(rematch_at.get() - current.get()).unwrap()).await;
    let terminal = publication(&value.notices).room;
    assert!(terminal.seats[1].rematch_ready);
    value
        .handle
        .action(
            human,
            creator.room_id.clone(),
            u64::from(terminal.round_id.get()),
            MatchAction::Rematch(true),
        )
        .await
        .unwrap()
        .unwrap();
    advance_ticks(1).await;
    let reset = publication(&value.notices).room;
    assert!(matches!(
        reset.phase,
        grenadiers_core::protocol::v12::RoomPhase::Waiting
    ));
    assert!(reset.seats[1].ready);
    assert!(!reset.seats[0].ready);
    value.task.abort();
}

#[test]
fn archived_cpu_profile_cannot_label_a_current_live_worker() {
    for profile in [
        CpuProfile::INITIAL,
        CpuProfile::PULSE_V2,
        CpuProfile::ROUTE_V3,
        CpuProfile::PLANT_V4,
        CpuProfile::GOALS_V5,
        CpuProfile::WEAPONS_V6,
        CpuProfile::WALL_PASS_V7,
        CpuProfile::RESPONSIVE_V8,
        CpuProfile::ORIGIN_V9,
        CpuProfile::GUIDED_V10,
        CpuProfile::CLEAR_GAP_V11,
        CpuProfile::STEERING_V12,
        CpuProfile::BRACELET_V13,
    ] {
        let mut room = identity(RoomProfile::ExpandedV5);
        room.mode = RoomMode::Solo(profile);
        room.private_cpu_seed = Some([17; 32]);
        let (_, shutdown) = watch::channel(false);
        let result = PreparedMatchOwner::new_with_planner(
            room,
            Instant::now(),
            RoundSeedSource::with_entropy(|bytes| {
                bytes.fill(23);
                Ok(())
            }),
            Arc::new(|bytes: &mut [u8]| {
                bytes.fill(31);
                Ok(())
            }),
            crate::game::rooms::cpu::CpuPlanner::without_workers(),
            shutdown,
        );
        assert!(matches!(
            result,
            Err(crate::game::rooms::match_runtime::MatchRuntimeError::Publication)
        ));
    }
}
