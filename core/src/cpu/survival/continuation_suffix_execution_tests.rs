use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::UtilityMemory;
use crate::cpu::routing::{
    CONTINUATION_RENEWAL_ALLOWANCE, ContinuationCertificate, PreparedFrame, RouteFailure,
};
use crate::cpu::{
    BombKnowledge, MovementIntent, Observation, ObservedActor, ObservedEffect, ObservedPickup,
    WorkBudget,
};
use crate::game::{CellId, EffectKind, InputAction, PickupKind, SeatId, SeatInput, Tick};

use super::continuation_compatibility::prepare_checked;
use super::continuation_scene_fixture::{ContinuationScene, scene};
use super::continuation_selection::OriginRecoveryChoice;
use super::continuation_test_fixture::{CPU, observe, observe_for};
use super::{WORK_ALLOWANCE, plan_continuation_lairs_v62};

fn build_certificate(
    basis: &Observation,
    key: crate::cpu::MemoryKey,
    origin: &super::PlantEscapeOrigin,
) -> Box<ContinuationCertificate> {
    let choice = plan_continuation_lairs_v62(
        basis,
        &ObservedHazardMemory::new(key),
        basis.parts().tick,
        &UtilityMemory::new(key),
        origin,
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .expect("v62 continuation selector");
    let OriginRecoveryChoice::Continuation(certificate) = choice else {
        panic!("fixture must select retained continuation")
    };
    certificate
}

fn input(seat: SeatId, movement: MovementIntent) -> SeatInput {
    SeatInput {
        seat,
        action: match movement {
            MovementIntent::Direction(direction) => InputAction::Press(direction),
            MovementIntent::Stop => InputAction::Clear,
        },
    }
}

#[derive(Clone, Copy)]
enum SuffixPublication {
    First,
    Last,
}

fn retained_suffix_publication(
    requested: SuffixPublication,
) -> (Box<ContinuationCertificate>, Observation) {
    let ContinuationScene {
        mut state,
        settings,
        basis,
        rival_basis,
        origin,
        rival_origin,
        key,
        rival_key,
    } = scene();
    let certificate = build_certificate(&basis, key, &origin);
    let rival = build_certificate(&rival_basis, rival_key, &rival_origin);
    let (first, last, _) = certificate
        .stationary_suffix_shape_for_test()
        .expect("retained suffix");
    let publication = match requested {
        SuffixPublication::First => first,
        SuffixPublication::Last => last,
    };
    let own = certificate.checkpoints_for_test();
    let other = rival.checkpoints_for_test();

    while state.tick() < publication {
        let update = Tick(state.tick().0 + 1);
        let own_movement = own
            .iter()
            .find(|checkpoint| checkpoint.tick == update)
            .map_or(MovementIntent::Stop, |checkpoint| checkpoint.movement);
        let rival_movement = other
            .iter()
            .find(|checkpoint| checkpoint.tick == update)
            .map_or(MovementIntent::Stop, |checkpoint| checkpoint.movement);
        state
            .step(&[input(CPU, own_movement), input(SeatId::One, rival_movement)])
            .unwrap();
    }
    let current = observe(&state, settings);
    let rival_current = observe_for(&state, settings, SeatId::One);
    assert_eq!(current.parts().tick, publication);
    assert_eq!(rival_current.parts().tick, publication);
    (certificate, current)
}

#[test]
fn stationary_suffix_renewal_has_an_exact_fail_closed_budget_boundary() {
    let (certificate, current) = retained_suffix_publication(SuffixPublication::First);
    let mut measured = WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE);
    assert!(matches!(
        prepare_checked(&certificate, &current, &mut measured),
        Ok(PreparedFrame::Hold(_))
    ));
    let required = measured.spent();
    assert!((1..=CONTINUATION_RENEWAL_ALLOWANCE).contains(&required));
    let exact = certificate.clone();
    assert!(matches!(
        prepare_checked(&exact, &current, &mut WorkBudget::new(required)),
        Ok(PreparedFrame::Hold(_))
    ));
    let one_short = certificate.clone();
    assert!(matches!(
        prepare_checked(&one_short, &current, &mut WorkBudget::new(required - 1)),
        Err(RouteFailure::WorkExhausted)
    ));
}

#[test]
fn stationary_suffix_rejects_bounded_tail_fact_mutations() {
    let (certificate, current) = retained_suffix_publication(SuffixPublication::First);
    let mutations = [
        current
            .clone()
            .map_parts(|parts| {
                let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                    unreachable!()
                };
                bombs[0].id.0 += 100;
            })
            .unwrap(),
        current
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
                    remaining_ticks: 2,
                });
            })
            .unwrap(),
        current
            .clone()
            .map_parts(|parts| {
                parts.pickups.push(ObservedPickup {
                    cell: CellId::new(7, 1),
                    kind: PickupKind::Teleporter,
                });
            })
            .unwrap(),
    ];
    for changed in mutations {
        assert!(
            prepare_checked(
                &certificate,
                &changed,
                &mut WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE),
            )
            .is_err()
        );
    }
}

#[test]
fn stationary_suffix_accepts_the_last_single_protected_update() {
    let (certificate, current) = retained_suffix_publication(SuffixPublication::Last);
    assert_eq!(current.parts().tick.0 + 1, certificate.survive_until().0);
    assert!(matches!(
        prepare_checked(
            &certificate,
            &current,
            &mut WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE),
        ),
        Ok(PreparedFrame::Hold(_))
    ));
}
