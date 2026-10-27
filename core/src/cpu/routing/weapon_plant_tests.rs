use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::{
    BombKnowledge, MovementIntent, Observation, ObservedActor, ObservedBomb, ObservedBombTrigger,
    UnknownBombFacts, WorkBudget,
};
use crate::game::{
    BlastKind, CellId, Direction, EnabledItems, EntityId, InputAction, PickupKind, Position,
    SeatId, SeatInput, SpecialWeapon, Terrain, Tick,
};
use crate::room::RoomSettings;

use super::plant_tests::{current_scene, request};
use super::tests::{actual_state_with_pickup, key, observation};
use super::{PlantFailure, PlantPresence, RouteStep, plan_ordinary_plant, plan_timed_weapon_plant};

fn armed(weapon: SpecialWeapon, enabled: EnabledItems) -> Observation {
    let mut parts = observation(Vec::new(), Vec::new()).into_parts();
    parts.rules.settings = parts.rules.settings.with_enabled_items(enabled);
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.weapon = Some(weapon);
    Observation::new(parts).unwrap()
}

#[test]
fn timed_weapon_proof_binds_blast_and_consumption_to_presence() {
    for (weapon, enabled, blast) in [
        (
            SpecialWeapon::Piercing,
            EnabledItems::PIERCING_BOMB,
            BlastKind::Piercing,
        ),
        (
            SpecialWeapon::Power,
            EnabledItems::POWER_BOMB,
            BlastKind::Power,
        ),
    ] {
        let basis = armed(weapon, enabled);
        let certificate = plan_timed_weapon_plant(
            &basis,
            &ObservedHazardMemory::new(key()),
            request(MovementIntent::Stop),
            &mut WorkBudget::new(65_536),
        )
        .unwrap();
        assert_eq!(certificate.placement().blast, blast);
        assert_eq!(certificate.weapon_before(), Some(weapon));
        assert_eq!(
            certificate.expected_weapon(PlantPresence::Awaiting),
            Some(weapon)
        );
        assert_eq!(
            certificate.expected_weapon(PlantPresence::Absent),
            Some(weapon)
        );
        assert_eq!(
            certificate.expected_weapon(PlantPresence::Present(EntityId(700))),
            None
        );

        let present = current_scene(
            &basis,
            &certificate,
            Tick(8),
            Some((EntityId(700), certificate.placement().range)),
        );
        assert_eq!(
            certificate.confirm(&basis, &present),
            Ok(PlantPresence::Present(EntityId(700)))
        );
        let mut wrong = present.into_parts();
        let crate::cpu::BombKnowledge::Visible(bombs) = &mut wrong.bombs else {
            unreachable!()
        };
        bombs[0].blast = BlastKind::Standard;
        assert_eq!(
            certificate.confirm(&basis, &Observation::new(wrong).unwrap()),
            Err(PlantFailure::SceneMismatch)
        );
        let absent = current_scene(&basis, &certificate, Tick(8), None);
        assert_eq!(
            certificate.confirm(&basis, &absent),
            Ok(PlantPresence::Absent)
        );
    }
}

#[test]
fn wrappers_reject_the_wrong_loadout_or_disabled_timed_weapon() {
    let piercing = armed(SpecialWeapon::Piercing, EnabledItems::PIERCING_BOMB);
    assert_eq!(
        plan_ordinary_plant(
            &piercing,
            &ObservedHazardMemory::new(key()),
            request(MovementIntent::Stop),
            &mut WorkBudget::new(65_536),
        ),
        Err(PlantFailure::Unsupported)
    );
    for basis in [
        observation(Vec::new(), Vec::new()),
        armed(SpecialWeapon::Piercing, EnabledItems::EMPTY),
        armed(SpecialWeapon::Power, EnabledItems::EMPTY),
        armed(SpecialWeapon::Remote, EnabledItems::REMOTE_BOMB),
        armed(SpecialWeapon::Landmine, EnabledItems::LANDMINE),
    ] {
        assert_eq!(
            plan_timed_weapon_plant(
                &basis,
                &ObservedHazardMemory::new(key()),
                request(MovementIntent::Stop),
                &mut WorkBudget::new(65_536),
            ),
            Err(PlantFailure::Unsupported)
        );
    }

    let mut parts = armed(SpecialWeapon::Piercing, EnabledItems::PIERCING_BOMB).into_parts();
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.bomb_capacity = 2;
    actor.live_bomb_count = 1;
    parts.bombs = BombKnowledge::Visible(vec![ObservedBomb {
        id: EntityId(90),
        placed_at: Tick(0),
        cell: crate::game::CellId::new(5, 1),
        owner: actor.id,
        range: 2,
        trigger: ObservedBombTrigger::Remote,
        blast: BlastKind::Standard,
        unknown: UnknownBombFacts::ALL,
    }]);
    assert_eq!(
        plan_timed_weapon_plant(
            &Observation::new(parts).unwrap(),
            &ObservedHazardMemory::new(key()),
            request(MovementIntent::Stop),
            &mut WorkBudget::new(65_536),
        ),
        Err(PlantFailure::Unsupported)
    );
}

#[test]
fn authoritative_state_consumes_each_timed_weapon_and_replays_the_certified_escape() {
    for (weapon, pickup, enabled, blast) in [
        (
            SpecialWeapon::Piercing,
            PickupKind::PiercingBomb,
            EnabledItems::PIERCING_BOMB,
            BlastKind::Piercing,
        ),
        (
            SpecialWeapon::Power,
            PickupKind::PowerBomb,
            EnabledItems::POWER_BOMB,
            BlastKind::Power,
        ),
    ] {
        let basis = armed(weapon, enabled);
        let actor = actor(&basis);
        let certificate = plan_timed_weapon_plant(
            &basis,
            &ObservedHazardMemory::new(key()),
            request(MovementIntent::Stop),
            &mut WorkBudget::new(65_536),
        )
        .unwrap();
        let settings = RoomSettings::DEFAULT.with_enabled_items(enabled);
        let mut state = actual_state_with_pickup(
            actor.position,
            Direction::East,
            actor.speed,
            settings,
            Some(pickup),
        );
        state.step(&[]).unwrap();
        assert_eq!(
            state.actors()[SeatId::Two.index()].inventory().weapon(),
            Some(weapon)
        );
        while state.tick() < Tick(6) {
            state.step(&[]).unwrap();
        }
        state
            .step(&[SeatInput {
                seat: SeatId::Two,
                action: InputAction::PlaceBomb,
            }])
            .unwrap();
        let placed = state
            .world()
            .bombs()
            .values()
            .find(|bomb| bomb.owner() == certificate.placement().owner)
            .unwrap();
        assert_eq!(placed.blast(), blast);
        assert_eq!(placed.placed_at(), certificate.placement().placed_at);
        assert_eq!(
            state.actors()[SeatId::Two.index()].inventory().weapon(),
            None
        );
        let lane = certificate.route().lane(Tick(7)).unwrap();
        while state.tick().0 + 1 < certificate.route().survive_until().0 {
            let next = Tick(state.tick().0 + 1);
            let mut inputs = Vec::new();
            for (index, step) in certificate.route().steps().iter().enumerate() {
                if index > 0
                    && lane.step_start(index) == Some(next)
                    && let RouteStep::Move { direction, .. } = step
                {
                    inputs.push(SeatInput {
                        seat: SeatId::Two,
                        action: InputAction::Press(*direction),
                    });
                }
                if matches!(step, RouteStep::Move { .. }) && lane.stop_update(index) == Some(next) {
                    inputs.push(SeatInput {
                        seat: SeatId::Two,
                        action: InputAction::Clear,
                    });
                }
            }
            state.step(&inputs).unwrap();
            let actual = &state.actors()[SeatId::Two.index()];
            assert_eq!(
                lane.pose_at(state.tick()),
                Some((actual.position(), actual.heading()))
            );
        }
        assert!(state.actors()[SeatId::Two.index()].alive());
    }
}

#[test]
fn diagonal_refuge_certifies_standard_but_not_power() {
    let standard = refuge_pocket(None);
    let ordinary = plan_ordinary_plant(
        &standard,
        &ObservedHazardMemory::new(key()),
        request(MovementIntent::Stop),
        &mut WorkBudget::new(65_536),
    )
    .expect("the diagonal refuge is outside the Standard cross");
    let lane = ordinary.route().lane(Tick(7)).unwrap();
    let terminal = lane
        .pose_at(Tick(ordinary.route().survive_until().0 - 1))
        .unwrap()
        .0
        .cell();
    assert_eq!(terminal, Some(CellId::new(4, 4)));

    let power = refuge_pocket(Some(SpecialWeapon::Power));
    let mut work = WorkBudget::new(65_536);
    assert!(
        plan_timed_weapon_plant(
            &power,
            &ObservedHazardMemory::new(key()),
            request(MovementIntent::Stop),
            &mut work,
        )
        .is_err()
    );
    assert!(
        work.spent() > 1_000,
        "failure must follow bounded proof work"
    );
}

fn refuge_pocket(weapon: Option<SpecialWeapon>) -> Observation {
    let mut parts = observation(Vec::new(), Vec::new()).into_parts();
    parts.arena.terrain.fill(Terrain::Wall);
    for cell in [
        CellId::new(1, 1),
        CellId::new(13, 11),
        CellId::new(3, 3),
        CellId::new(3, 4),
        CellId::new(4, 4),
    ] {
        let index =
            usize::from(cell.row) * usize::from(parts.arena.width) + usize::from(cell.column);
        parts.arena.terrain[index] = Terrain::Floor;
    }
    if weapon.is_some() {
        parts.rules.settings = parts
            .rules
            .settings
            .with_enabled_items(EnabledItems::POWER_BOMB);
    }
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.position = Position::center(CellId::new(3, 3));
    actor.heading = Direction::South;
    actor.weapon = weapon;
    Observation::new(parts).unwrap()
}

fn actor(observation: &Observation) -> &crate::cpu::VisibleActor {
    let ObservedActor::Visible(actor) = &observation.parts().actors[1] else {
        unreachable!()
    };
    actor
}
