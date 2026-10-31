use grenadiers_core::{
    cpu::policy::{
        IntentDebugSummary, StrategicObjectiveKind, StrategicPhase, StrategicSwitchReason,
        StrategicTarget,
    },
    game::{CellId, PickupKind, SeatId, Tick},
    protocol::{Counter, Direction, EntityId, Position, Tick as WireTick, v14::ResumeToken, v18},
    room::{CpuDifficulty, SeatControl},
};

use super::{
    cpu_debug::{available_row, recipient_bomb_blind},
    tests::{advance_tick, binding, owner},
};
use crate::{cpu_debug::CpuDebugQueryError, game::rooms::roster_runtime::RosterAction};

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn credential_query_is_read_only_and_retired_credentials_fail_closed() {
    let (handle, prepared) = owner(
        &[SeatControl::Human, SeatControl::Cpu, SeatControl::Human],
        Some(CpuDifficulty::Standard),
    );
    let task = prepared.start(|| {});
    let creator_binding = binding(1);
    let mut admission = handle
        .admit_creator(creator_binding, Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    let token = admission.resume_token.clone();
    let _retained_peer = handle
        .join_sequenced(binding(2), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        handle.cpu_debug(token.clone()).await.unwrap(),
        Err(CpuDebugQueryError::Unavailable)
    );
    assert_eq!(
        handle
            .action(
                creator_binding,
                admission.room_id.clone(),
                admission.round_id,
                RosterAction::Ready {
                    ready: true,
                    settings_revision: 1,
                },
            )
            .await
            .unwrap(),
        Ok(())
    );
    assert!(admission.notices.changed().await.is_ok());
    assert_eq!(
        handle.cpu_debug(token.clone()).await.unwrap(),
        Err(CpuDebugQueryError::Unavailable)
    );

    let foreign = ResumeToken::new("ff".repeat(32)).unwrap();
    assert_eq!(
        handle.cpu_debug(foreign).await.unwrap(),
        Err(CpuDebugQueryError::Unauthorized)
    );

    let leave = handle.leave(creator_binding, admission.room_id, admission.round_id);
    advance_tick().await;
    assert_eq!(leave.await.unwrap(), Ok(()));
    assert_eq!(
        handle.cpu_debug(token).await.unwrap(),
        Err(CpuDebugQueryError::Unauthorized)
    );
    task.abort();
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn reserved_seat_is_unauthorized_until_normal_resume_reconnects_it() {
    let (handle, prepared) = owner(
        &[SeatControl::Human, SeatControl::Cpu, SeatControl::Human],
        Some(CpuDifficulty::Standard),
    );
    let task = prepared.start(|| {});
    let original_binding = binding(10);
    let admission = handle
        .admit_creator(original_binding, Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    let _retained_peer = handle
        .join_sequenced(binding(20), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    let room_id = admission.room_id.clone();
    let round_id = admission.round_id;
    let token = admission.resume_token.clone();
    drop(admission.guard);
    advance_tick().await;

    assert_eq!(
        handle.cpu_debug(token.clone()).await.unwrap(),
        Err(CpuDebugQueryError::Unauthorized)
    );
    assert_eq!(
        handle
            .action(
                original_binding,
                room_id.clone(),
                round_id,
                RosterAction::Ready {
                    ready: false,
                    settings_revision: 1,
                },
            )
            .await
            .unwrap(),
        Err(crate::game::rooms::RoomError::StaleRoom)
    );

    let resumed = handle
        .resume(
            binding(11),
            Counter::new(2).unwrap(),
            room_id,
            token.clone(),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        handle.cpu_debug(token).await.unwrap(),
        Err(CpuDebugQueryError::Unavailable)
    );
    drop(resumed);
    task.abort();
}

#[tokio::test]
async fn saturated_diagnostic_queue_does_not_consume_gameplay_command_capacity() {
    let (handle, prepared) = owner(
        &[SeatControl::Human, SeatControl::Cpu],
        Some(CpuDifficulty::Standard),
    );
    let token = ResumeToken::new("ff".repeat(32)).unwrap();
    let pending = (0..super::types::CPU_DEBUG_COMMAND_CAPACITY)
        .map(|_| handle.cpu_debug(token.clone()))
        .collect::<Vec<_>>();
    assert_eq!(
        handle.cpu_debug(token).await.unwrap(),
        Err(CpuDebugQueryError::Unavailable)
    );

    let admission = handle.admit_creator(binding(1), Counter::new(1).unwrap());
    let task = prepared.start(|| {});
    assert!(admission.await.unwrap().is_ok());
    for response in pending {
        assert!(response.await.unwrap().is_err());
    }
    task.abort();
}

#[test]
fn hidden_or_stale_targets_redact_the_entire_intent_row() {
    let mut state = empty_state();
    state.actors = vec![hidden_actor(v18::Seat::One), hidden_actor(v18::Seat::Two)];
    let mut summary = summary(StrategicTarget::Opponent { seat: SeatId::Two });
    assert!(available_row(summary, &state).is_none());

    summary.target = StrategicTarget::Pickup {
        cell: CellId::new(2, 2),
        kind: PickupKind::Kick,
    };
    assert!(available_row(summary, &state).is_none());
    summary.target = StrategicTarget::Block {
        cell: CellId::new(2, 2),
    };
    assert!(available_row(summary, &state).is_none());
    summary.target = StrategicTarget::Tile {
        cell: CellId::new(99, 99),
    };
    assert!(available_row(summary, &state).is_none());
}

#[test]
fn invisible_bomb_effect_conservatively_marks_recipient_bomb_blind() {
    assert!(recipient_bomb_blind(&hidden_actor(v18::Seat::One)));
    assert!(recipient_bomb_blind(&visible_actor(true)));
    assert!(!recipient_bomb_blind(&visible_actor(false)));
}

fn summary(target: StrategicTarget) -> IntentDebugSummary {
    IntentDebugSummary {
        cpu_seat: SeatId::Two,
        objective: StrategicObjectiveKind::Attack,
        target,
        approach_cell: None,
        phase: StrategicPhase::Approach,
        last_switch_reason: StrategicSwitchReason::Initial,
        observation_tick: Tick(1),
    }
}

fn empty_state() -> v18::GameState {
    v18::GameState {
        running_updates: 0,
        remaining_ticks: 0,
        physical_tick: WireTick::new(1).unwrap(),
        arena: v18::ArenaView {
            width: 15,
            height: 13,
            terrain: Vec::new(),
            blocks: Vec::new(),
        },
        actors: Vec::new(),
        bombs: Vec::new(),
        flames: Vec::new(),
        oil_barrels: Vec::new(),
        pickups: Vec::new(),
        storm_cloud: None,
        lairs: Vec::new(),
        mobs: Vec::new(),
        spike_pits: Vec::new(),
    }
}

fn hidden_actor(seat: v18::Seat) -> v18::ActorView {
    let id = match seat {
        v18::Seat::One => 1,
        v18::Seat::Two => 2,
        v18::Seat::Three => 3,
        v18::Seat::Four => 4,
    };
    v18::ActorView::Hidden {
        id: EntityId::new(id),
        seat,
        alive: true,
    }
}

fn visible_actor(bomb_blind: bool) -> v18::ActorView {
    let effects = bomb_blind
        .then(|| v18::EffectView::new(v18::EffectKind::InvisibleBombs, 1).unwrap())
        .into_iter()
        .collect();
    v18::ActorView::Visible {
        id: EntityId::new(1),
        seat: v18::Seat::One,
        position: Position::new(0, 0).unwrap(),
        heading: Direction::North,
        speed_level: 0,
        alive: true,
        bomb_capacity: 1,
        bomb_range: 1,
        live_bomb_count: 0,
        inventory: v18::InventoryView {
            equipment: None,
            weapon: None,
        },
        effects: v18::EffectViews::new(effects).unwrap(),
        damage_grace_remaining_ticks: v18::DamageGraceTicks::new(0).unwrap(),
        voluntary_actions_frozen: false,
        last_teleport_tick: None,
        storm_slow_remaining_ticks: 0,
    }
}
