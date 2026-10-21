use grenadiers_core::game::{BombTrigger, Direction, EnabledItems, SeatId, SpecialWeapon};
use grenadiers_core::room::{CountdownKind, MatchPhase};

use super::replay;
use crate::game::rooms::match_runtime::{
    MatchAction, MatchRuntime, collect_generated_pickup, move_cells, plant, start_optional_runtime,
};

fn advance_to_next_round(runtime: &mut MatchRuntime) {
    loop {
        runtime.tick().unwrap();
        if matches!(
            runtime.lifecycle().phase(),
            MatchPhase::Countdown {
                kind: CountdownKind::InterRound,
                ..
            }
        ) {
            break;
        }
    }
    while runtime.lifecycle().phase() != MatchPhase::Running {
        runtime.tick().unwrap();
    }
}

#[test]
fn generated_landmine_grace_exit_contact_and_resume_replay_deterministically() {
    let mut runtime = start_optional_runtime(EnabledItems::LANDMINE, 31);
    collect_generated_pickup(&mut runtime);
    assert_eq!(
        runtime.sim().unwrap().state.actors()[0]
            .inventory()
            .weapon(),
        Some(SpecialWeapon::Landmine)
    );
    plant(&mut runtime, 2);
    let mine = *runtime
        .sim()
        .unwrap()
        .state
        .world()
        .bombs()
        .values()
        .next()
        .unwrap();
    assert_eq!(mine.trigger(), BombTrigger::Contact);
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    let restoration = runtime.prepare_resume(SeatId::One).unwrap();
    runtime.commit_admission(restoration);
    for _ in 0..3 {
        runtime.tick().unwrap();
    }
    assert!(
        runtime
            .sim()
            .unwrap()
            .state
            .world()
            .bombs()
            .contains_key(&mine.id())
    );
    move_cells(&mut runtime, Direction::West, 2);
    assert!(
        runtime
            .sim()
            .unwrap()
            .state
            .world()
            .bombs()
            .contains_key(&mine.id())
    );
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    runtime
        .action(
            SeatId::One,
            runtime.lifecycle().round_attempt(),
            MatchAction::Movement(Some(Direction::East)),
        )
        .unwrap();
    let epoch = runtime.lifecycle().round_attempt();
    for update in 0..40 {
        if update > 0 && update % 10 == 0 {
            runtime
                .action(
                    SeatId::One,
                    epoch,
                    MatchAction::Movement(Some(Direction::East)),
                )
                .unwrap();
        }
        runtime.tick().unwrap();
        if !runtime
            .sim()
            .unwrap()
            .state
            .world()
            .bombs()
            .contains_key(&mine.id())
        {
            break;
        }
    }
    assert!(
        !runtime
            .sim()
            .unwrap()
            .state
            .world()
            .bombs()
            .contains_key(&mine.id())
    );
    assert!(!runtime.sim().unwrap().state.actors()[0].alive());
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    advance_to_next_round(&mut runtime);
    let next = &runtime.sim().unwrap().state;
    assert!(next.world().bombs().is_empty());
    assert!(next.actors().iter().all(|actor| actor.alive()));
    assert!(
        next.actors()
            .iter()
            .all(|actor| actor.inventory().weapon().is_none())
    );
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}
