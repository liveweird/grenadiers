use crate::cpu::routing::CONTINUATION_RENEWAL_ALLOWANCE;
use crate::cpu::{
    BombKnowledge, DecisionId, Observation, ObservedActor, ObservedEffect, ObservedPickup,
    WorkBudget,
};
use crate::game::{CellId, EffectKind, PickupKind, Position, Tick};

use super::continuation_execution::{
    CommittedContinuation, ContinuationPublication, ContinuationStatus,
};
use super::continuation_scene_fixture::no_lair_scene;

#[test]
fn publication_constructor_rejects_cross_world_and_rewind() {
    let scene = no_lair_scene();
    let decision = DecisionId {
        world: scene.basis.parts().world,
        command_epoch: 4,
        observation_tick: scene.basis.parts().tick,
        checked_request_serial: 9,
    };
    let mut wrong_key = scene.key;
    wrong_key.world.physical_round += 1;
    assert!(ContinuationPublication::new(decision, wrong_key, 1, Tick(199)).is_err());
    assert!(ContinuationPublication::new(decision, scene.key, 1, Tick(195)).is_err());
    assert!(ContinuationPublication::new(decision, scene.key, 1, Tick(196)).is_ok());
}

pub(super) fn assert_renewal_bound_and_rejections(
    committed: &CommittedContinuation,
    current: &Observation,
    identity: ContinuationPublication,
) {
    let mut measured_executor = committed.fork_for_test();
    let mut measured = WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE);
    measured_executor
        .prepare_grant(current, identity, &mut measured)
        .unwrap();
    let required = measured.spent();
    assert!((1..=CONTINUATION_RENEWAL_ALLOWANCE).contains(&required));
    assert!(
        committed
            .fork_for_test()
            .prepare_grant(current, identity, &mut WorkBudget::new(required))
            .is_ok()
    );
    let mut short = committed.fork_for_test();
    assert!(
        short
            .prepare_grant(current, identity, &mut WorkBudget::new(required - 1))
            .is_err()
    );
    assert!(matches!(
        short.poll_command(current.parts().tick, identity.decision.command_epoch),
        ContinuationStatus::Invalidated
    ));

    let mut duplicate = committed.fork_for_test();
    duplicate
        .prepare_grant(
            current,
            identity,
            &mut WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE),
        )
        .unwrap();
    assert!(
        duplicate
            .prepare_grant(
                current,
                identity,
                &mut WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE),
            )
            .is_err()
    );
    assert!(matches!(
        duplicate.poll_command(current.parts().tick, identity.decision.command_epoch),
        ContinuationStatus::Invalidated
    ));

    let mut stale = committed.fork_for_test();
    assert!(
        stale
            .prepare_grant(
                current,
                ContinuationPublication {
                    sequence: identity.sequence - 1,
                    ..identity
                },
                &mut WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE),
            )
            .is_err()
    );

    let mut wrong_epoch = committed.fork_for_test();
    let mut changed_identity = identity;
    changed_identity.decision.command_epoch += 1;
    assert!(
        wrong_epoch
            .prepare_grant(
                current,
                changed_identity,
                &mut WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE),
            )
            .is_err()
    );

    let mut missing = committed.fork_for_test();
    assert!(matches!(
        missing.poll_command(
            Tick(current.parts().tick.0 + 1),
            identity.decision.command_epoch,
        ),
        ContinuationStatus::Invalidated
    ));

    for mutation in mutations(current) {
        let mut changed_executor = committed.fork_for_test();
        assert!(
            changed_executor
                .prepare_grant(
                    &mutation,
                    identity,
                    &mut WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE),
                )
                .is_err()
        );
    }
}

fn mutations(current: &Observation) -> Vec<Observation> {
    let flame = current
        .clone()
        .map_parts(|parts| {
            parts.flames.push(crate::cpu::ObservedFlame {
                cell: CellId::new(7, 1),
                remaining_ticks: 2,
            })
        })
        .unwrap();
    let moved_bomb = current
        .clone()
        .map_parts(|parts| {
            let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                unreachable!()
            };
            bombs[0].cell = CellId::new(2, 9);
        })
        .unwrap();
    let pickup = current
        .clone()
        .map_parts(|parts| {
            parts.pickups.push(ObservedPickup {
                cell: CellId::new(2, 11),
                kind: PickupKind::Teleporter,
            });
        })
        .unwrap();
    let changed_actor = current
        .clone()
        .map_parts(|parts| {
            let ObservedActor::Visible(actor) = parts
                .actors
                .iter_mut()
                .find(|actor| actor.seat() == parts.recipient)
                .unwrap()
            else {
                unreachable!()
            };
            actor.effects.push(ObservedEffect {
                kind: EffectKind::ReverseControls,
                remaining_ticks: 20,
            });
        })
        .unwrap();
    let blocking_rival = current
        .clone()
        .map_parts(|parts| {
            let ObservedActor::Visible(rival) = parts
                .actors
                .iter_mut()
                .find(|actor| actor.seat() != parts.recipient)
                .unwrap()
            else {
                unreachable!()
            };
            rival.position = Position { x: 1536, y: 10_100 };
        })
        .unwrap();
    vec![flame, moved_bomb, pickup, changed_actor, blocking_rival]
}
