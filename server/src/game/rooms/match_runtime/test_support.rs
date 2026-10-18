use std::sync::Arc;

use grenadiers_core::{
    game::{Direction, SeatId},
    protocol::v8::RoomPhase,
    protocol::{RoomCode, RoomId},
    room::MatchPhase,
};

use super::super::{profile::RoomProfile, types::RoomIdentity};
use super::{MatchAction, MatchNotice, MatchRuntime};
use crate::round_seed::RoundSeedSource;

pub(super) fn identity(profile: RoomProfile) -> RoomIdentity {
    RoomIdentity {
        id: RoomId::new("00000000000000000000000000000001").unwrap(),
        code: RoomCode::new("ABCDEFGH").unwrap(),
        profile,
    }
}

pub(super) fn fixed_runtime() -> MatchRuntime {
    MatchRuntime::new(
        identity(RoomProfile::FixedClassicV1),
        RoundSeedSource::with_entropy(|_| panic!("fixed profile requested entropy")),
    )
}

pub(super) fn generated_runtime(source: RoundSeedSource) -> MatchRuntime {
    MatchRuntime::new(identity(RoomProfile::GeneratedClassicV2), source)
}

pub(super) fn configurable_runtime(source: RoundSeedSource) -> MatchRuntime {
    MatchRuntime::new(identity(RoomProfile::GeneratedConfigurableV4), source)
}

pub(in crate::game::rooms) fn expanded_heart_runtime() -> MatchRuntime {
    MatchRuntime::new(
        identity(RoomProfile::ExpandedV5),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(3);
            Ok(())
        }),
    )
}

pub(in crate::game::rooms) fn start_expanded_heart_round(runtime: &mut MatchRuntime) {
    occupy_both(runtime);
    for seat in SeatId::ALL {
        runtime
            .action(
                seat,
                1,
                MatchAction::ReadyChecked {
                    ready: true,
                    settings_revision: 1,
                },
            )
            .unwrap();
    }
    tick_until_running(runtime);
}

pub(in crate::game::rooms) fn start_shield_runtime() -> MatchRuntime {
    start_optional_runtime(grenadiers_core::game::EnabledItems::SHIELD, 31)
}

pub(in crate::game::rooms) fn start_optional_runtime(
    enabled: grenadiers_core::game::EnabledItems,
    seed_byte: u8,
) -> MatchRuntime {
    start_optional_runtime_with_fuse(enabled, seed_byte, 180)
}

pub(in crate::game::rooms) fn start_optional_runtime_with_fuse(
    enabled: grenadiers_core::game::EnabledItems,
    seed_byte: u8,
    fuse_ticks: u64,
) -> MatchRuntime {
    let mut runtime = MatchRuntime::new(
        identity(RoomProfile::ExpandedV5),
        RoundSeedSource::with_entropy(move |bytes| {
            bytes.fill(seed_byte);
            Ok(())
        }),
    );
    occupy_both(&mut runtime);
    runtime
        .action(
            SeatId::One,
            1,
            MatchAction::UpdateSettings {
                settings_revision: 1,
                settings: grenadiers_core::room::RoomSettings::new(
                    grenadiers_core::game::SpeedLevel::Base,
                    256,
                    3,
                    fuse_ticks,
                )
                .unwrap()
                .with_enabled_items(enabled),
            },
        )
        .unwrap();
    for seat in SeatId::ALL {
        runtime
            .action(
                seat,
                1,
                MatchAction::ReadyChecked {
                    ready: true,
                    settings_revision: 2,
                },
            )
            .unwrap();
    }
    tick_until_running(&mut runtime);
    runtime
}

pub(in crate::game::rooms) fn collect_generated_heart(runtime: &mut MatchRuntime) {
    collect_generated_pickup(runtime);
    assert_eq!(
        runtime.sim().unwrap().state.actors()[0]
            .inventory()
            .equipment(),
        Some(grenadiers_core::game::Equipment::Heart)
    );
}

pub(in crate::game::rooms) fn collect_generated_pickup(runtime: &mut MatchRuntime) {
    move_cells(runtime, Direction::East, 1);
    plant(runtime, 1);
    move_cells(runtime, Direction::West, 1);
    move_cells(runtime, Direction::South, 1);
    wait_clear(runtime);
    move_cells(runtime, Direction::North, 1);
    move_cells(runtime, Direction::East, 2);
}

pub(in crate::game::rooms) fn absorb_generated_heart(runtime: &mut MatchRuntime) {
    runtime
        .action(
            SeatId::One,
            runtime.lifecycle().round_attempt(),
            MatchAction::Bomb(2),
        )
        .unwrap();
    for _ in 0..=180 {
        runtime.tick().unwrap();
        let actor = &runtime.sim().unwrap().state.actors()[0];
        assert!(actor.alive());
        if actor.inventory().equipment().is_none() {
            return;
        }
    }
    panic!("generated Heart was not consumed by ordinary bomb damage");
}

pub(super) fn occupy_both(runtime: &mut MatchRuntime) {
    runtime.occupy(SeatId::One).unwrap();
    runtime.occupy(SeatId::Two).unwrap();
}

pub(super) fn start_first_round(runtime: &mut MatchRuntime) -> MatchNotice {
    occupy_both(runtime);
    runtime
        .action(SeatId::One, 1, MatchAction::Ready(true))
        .unwrap();
    runtime
        .action(SeatId::Two, 1, MatchAction::Ready(true))
        .unwrap();
    tick_until_running(runtime)
}

pub(super) fn tick_until_running(runtime: &mut MatchRuntime) -> MatchNotice {
    loop {
        if runtime.lifecycle().phase() == MatchPhase::Running {
            panic!("runtime reached running without returning its boundary notice");
        }
        if let Some(notice) = runtime.tick().unwrap()
            && runtime.lifecycle().phase() == MatchPhase::Running
        {
            return notice;
        }
    }
}

pub(super) fn finish_round_with_bombs(
    runtime: &mut MatchRuntime,
    bombing: &[SeatId],
) -> MatchNotice {
    let epoch = runtime.lifecycle().round_attempt();
    for &seat in bombing {
        runtime.action(seat, epoch, MatchAction::Bomb(1)).unwrap();
    }
    loop {
        if let Some(notice) = runtime.tick().unwrap()
            && runtime.lifecycle().phase() != MatchPhase::Running
        {
            return notice;
        }
    }
}

pub(super) fn advance_to_inter_round(runtime: &mut MatchRuntime) -> MatchNotice {
    loop {
        let notice = runtime.tick().unwrap();
        if matches!(
            runtime.lifecycle().phase(),
            MatchPhase::Countdown {
                kind: grenadiers_core::room::CountdownKind::InterRound,
                ..
            }
        ) {
            return notice.expect("inter-round transition notice");
        }
    }
}

pub(super) fn snapshot_json(notice: &MatchNotice) -> serde_json::Value {
    serde_json::from_str(&notice.frame(SeatId::One).expect("snapshot notice")).unwrap()
}

pub(super) fn snapshot_state(notice: &MatchNotice) -> serde_json::Value {
    snapshot_json(notice)["state"].clone()
}

pub(super) fn assert_view_only(notice: &MatchNotice) {
    assert!(notice.frame(SeatId::One).is_none());
}

pub(super) fn assert_snapshot(notice: &MatchNotice) {
    assert!(notice.frame(SeatId::One).is_some());
}

pub(super) fn phase_type(notice: &MatchNotice) -> &str {
    match &notice.room.phase {
        RoomPhase::Waiting => "waiting",
        RoomPhase::Countdown { .. } => "countdown",
        RoomPhase::Running => "running",
        RoomPhase::Result { .. } => "result",
        RoomPhase::MatchResult { .. } => "matchResult",
        RoomPhase::Interrupted { .. } => "interrupted",
    }
}

pub(super) fn counting_source(counter: Arc<std::sync::atomic::AtomicUsize>) -> RoundSeedSource {
    RoundSeedSource::with_entropy(move |bytes| {
        let call = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        bytes.fill((call + 1) as u8);
        Ok(())
    })
}

pub(super) fn pickup_fixture_source() -> RoundSeedSource {
    let text = include_str!("../../../../../core/tests/fixtures/arena-v1-pickups.seed");
    let mut seed = [0_u8; 32];
    for (index, byte) in seed.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).unwrap();
    }
    RoundSeedSource::with_entropy(move |bytes| {
        bytes.copy_from_slice(&seed);
        Ok(())
    })
}

pub(in crate::game::rooms) fn move_cells(
    runtime: &mut MatchRuntime,
    direction: Direction,
    cells: usize,
) {
    let epoch = runtime.lifecycle().round_attempt();
    runtime
        .action(SeatId::One, epoch, MatchAction::Movement(Some(direction)))
        .unwrap();
    for update in 0..cells * 20 {
        if update > 0 && update.is_multiple_of(10) {
            runtime
                .action(SeatId::One, epoch, MatchAction::Movement(Some(direction)))
                .unwrap();
        }
        runtime.tick().unwrap();
    }
    runtime
        .action(SeatId::One, epoch, MatchAction::Movement(None))
        .unwrap();
    runtime.tick().unwrap();
}

pub(in crate::game::rooms) fn plant(runtime: &mut MatchRuntime, action_id: u32) {
    runtime
        .action(
            SeatId::One,
            runtime.lifecycle().round_attempt(),
            MatchAction::Bomb(action_id),
        )
        .unwrap();
    runtime.tick().unwrap();
}

pub(in crate::game::rooms) fn wait_clear(runtime: &mut MatchRuntime) {
    for _ in 0..240 {
        let world = runtime.sim().unwrap().state.world();
        if world.bombs().is_empty() && world.hazards().is_empty() {
            return;
        }
        runtime.tick().unwrap();
    }
    panic!("hazards failed to clear");
}
