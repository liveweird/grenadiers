use crate::cpu::forecast::HazardForecast;
use crate::cpu::{MobContactLease, Observation, ObservedActor, WorkBudget};

use super::pulse_support::contact_envelope;
use super::{PulseLease, PulseRequest, RouteFailure, RouteMode};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PickupSafety {
    Reject,
    AllowTeleporterGuard,
    AllowUnmodeledGuard,
}

pub(crate) fn certify_lair_contact_pulse(
    observation: &Observation,
    forecast: &HazardForecast,
    request: PulseRequest,
    work: &mut WorkBudget,
) -> Result<MobContactLease, RouteFailure> {
    let envelope = contact_envelope(observation, request, work)?;
    let _ = super::pulse::certify_pulse_inner(
        observation,
        forecast,
        request,
        None,
        PickupSafety::AllowTeleporterGuard,
        Some(&envelope),
        work,
    )?;
    Ok(MobContactLease::new(envelope, request.stopped_safe_until))
}

pub(crate) fn certify_pickup_pulse(
    observation: &Observation,
    forecast: &HazardForecast,
    request: PulseRequest,
    with_contact: bool,
    work: &mut WorkBudget,
) -> Result<(PulseLease, Option<MobContactLease>), RouteFailure> {
    if request.hold_ticks != 1 || request.mode != RouteMode::CertifiedOnly {
        return Err(RouteFailure::InvalidRequest);
    }
    let actor = observation
        .parts()
        .actors
        .iter()
        .find_map(|candidate| match candidate {
            ObservedActor::Visible(actor) if actor.seat == observation.parts().recipient => {
                Some(actor)
            }
            _ => None,
        })
        .ok_or(RouteFailure::InvalidRequest)?;
    let center = actor.position.cell().ok_or(RouteFailure::InvalidRequest)?;
    if observation
        .parts()
        .pickups
        .iter()
        .any(|pickup| pickup.cell == center && collectible_at_start(actor, pickup.kind))
    {
        return Err(RouteFailure::Unsupported);
    }
    let envelope = with_contact
        .then(|| contact_envelope(observation, request, work))
        .transpose()?;
    let lease = super::pulse::certify_pulse_inner(
        observation,
        forecast,
        request,
        None,
        PickupSafety::AllowUnmodeledGuard,
        envelope.as_ref(),
        work,
    )?;
    let contact = envelope.map(|value| MobContactLease::new(value, request.stopped_safe_until));
    Ok((lease, contact))
}

fn collectible_at_start(actor: &crate::cpu::VisibleActor, kind: crate::game::PickupKind) -> bool {
    kind != crate::game::PickupKind::SlowFoot
        || actor
            .effects
            .iter()
            .filter(|effect| {
                effect.kind == crate::game::EffectKind::SlowFoot && effect.remaining_ticks > 1
            })
            .count()
            < crate::game::effects::MAX_SLOW_FOOT_STACKS
}

#[cfg(test)]
mod tests {
    use crate::cpu::{MovementIntent, ObservedPickup, WorkBudget};
    use crate::game::{CellId, Direction, PickupKind};

    use super::*;

    fn certify(kind: PickupKind, cell: CellId) -> Result<PulseLease, RouteFailure> {
        let observed = crate::cpu::routing::tests::observation(
            Vec::new(),
            vec![ObservedPickup { cell, kind }],
        )
        .map_parts(|parts| {
            let crate::cpu::ObservedActor::Visible(actor) = &mut parts.actors[1] else {
                unreachable!()
            };
            actor.position.x = 3_050;
        })
        .unwrap();
        let forecast = crate::cpu::routing::tests::forecast_for(&observed);
        certify_pickup_pulse(
            &observed,
            &forecast,
            crate::cpu::routing::tests::request(MovementIntent::Direction(Direction::East), 1),
            false,
            &mut WorkBudget::new(65_536),
        )
        .map(|(lease, _)| lease)
    }

    #[test]
    fn passive_and_physics_changing_pickups_get_only_one_checked_update() {
        for kind in [PickupKind::GoldenFlame, PickupKind::ReverseControls] {
            let lease = certify(kind, CellId::new(3, 1)).unwrap();
            assert_eq!(lease.hold_ticks(), 1, "{kind:?}");
        }
    }

    #[test]
    fn diarrhea_and_teleporter_collection_are_not_handed_off() {
        for kind in [PickupKind::Diarrhea, PickupKind::Teleporter] {
            assert_eq!(
                certify(kind, CellId::new(3, 1)),
                Err(RouteFailure::Unsupported),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn eligible_pickup_under_current_center_rejects_delayed_execution() {
        for kind in [PickupKind::GoldenFlame, PickupKind::ReverseControls] {
            assert_eq!(
                certify(kind, CellId::new(2, 1)),
                Err(RouteFailure::Unsupported),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn checked_update_collects_motion_effects_then_stops_for_fresh_state() {
        use crate::game::{EffectKind, EnabledItems, InputAction, SeatId, SeatInput, SpeedLevel};

        let none = EnabledItems::from_bits(0).unwrap();
        for (kind, enabled, effect) in [
            (PickupKind::MovementSpeed, none, None),
            (
                PickupKind::SlowFoot,
                EnabledItems::SLOW_FOOT,
                Some(EffectKind::SlowFoot),
            ),
            (
                PickupKind::ReverseControls,
                EnabledItems::REVERSE_CONTROLS,
                Some(EffectKind::ReverseControls),
            ),
        ] {
            let position = crate::game::Position { x: 3_050, y: 1_536 };
            let mut state = crate::cpu::routing::tests::actual_state(
                position,
                Direction::East,
                SpeedLevel::Base,
                crate::room::RoomSettings::DEFAULT.with_enabled_items(enabled),
            );
            state.insert_test_pickup(CellId::new(3, 1), kind);
            state
                .step(&[SeatInput {
                    seat: SeatId::Two,
                    action: InputAction::Press(Direction::East),
                }])
                .unwrap();
            assert!(
                !state
                    .world()
                    .exposed_pickups()
                    .contains_key(&CellId::new(3, 1))
            );
            let actor = state.actor(SeatId::Two).unwrap();
            if let Some(effect) = effect {
                assert!(actor.effect_observations().any(|(kind, _)| kind == effect));
            } else {
                assert_eq!(actor.speed(), SpeedLevel::One);
            }
            let collected = actor.position();
            state
                .step(&[SeatInput {
                    seat: SeatId::Two,
                    action: InputAction::Release(Direction::East),
                }])
                .unwrap();
            assert_eq!(
                state.actor(SeatId::Two).unwrap().position(),
                collected,
                "{kind:?}"
            );
        }
    }

    #[test]
    fn pickup_lease_allows_last_publication_phase_but_rejects_stale_or_changed_state() {
        use crate::cpu::forecast::TickWindow;
        use crate::cpu::survival::{CommittedPulse, PulseStatus};
        use crate::game::{SpeedLevel, Tick};

        let basis = crate::cpu::routing::tests::observation(
            Vec::new(),
            vec![ObservedPickup {
                cell: CellId::new(3, 1),
                kind: PickupKind::MovementSpeed,
            }],
        )
        .map_parts(|parts| {
            let crate::cpu::ObservedActor::Visible(actor) = &mut parts.actors[1] else {
                unreachable!()
            };
            actor.position.x = 3_050;
        })
        .unwrap();
        let forecast = crate::cpu::routing::tests::forecast_for(&basis);
        let (lease, _) = certify_pickup_pulse(
            &basis,
            &forecast,
            PulseRequest {
                movement: MovementIntent::Direction(Direction::East),
                hold_ticks: 1,
                start_window: TickWindow {
                    start: Tick(1),
                    end: Tick(8),
                },
                stopped_safe_until: Tick(18),
                mode: RouteMode::CertifiedOnly,
                settled_stop_tick: Tick(0),
            },
            false,
            &mut WorkBudget::new(65_536),
        )
        .unwrap();
        let committed =
            CommittedPulse::admit(lease.clone(), basis.clone(), &basis, Tick(6)).unwrap();
        assert_eq!(committed.first_execution(), Tick(7));
        assert!(CommittedPulse::admit(lease.clone(), basis.clone(), &basis, Tick(7)).is_err());

        let mut changed = basis.clone().into_parts();
        changed.tick = Tick(7);
        changed.rules.running_updates += 7;
        changed.rules.remaining_ticks -= 7;
        changed.pickups.clear();
        let crate::cpu::ObservedActor::Visible(actor) = &mut changed.actors[1] else {
            unreachable!()
        };
        actor.position = lease.pose_after(1).min();
        actor.speed = SpeedLevel::One;
        let changed = crate::cpu::Observation::new(changed).unwrap();
        assert_eq!(
            committed.validate(&changed, Tick(7)),
            PulseStatus::Invalidated
        );
    }
}
