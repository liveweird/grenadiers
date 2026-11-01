use super::*;

#[test]
fn bombs_flames_oil_exposed_teleporter_and_local_pickup_disable_optional_scoring() {
    let original = collected_basis_at(START).observation;
    let flame = original
        .clone()
        .map_parts(|parts| {
            parts.flames.push(crate::cpu::ObservedFlame {
                cell: CellId::new(1, 2),
                remaining_ticks: 5,
            });
        })
        .unwrap();
    let oil = original
        .clone()
        .map_parts(|parts| {
            parts.oil.push(crate::cpu::ObservedOil {
                cell: CellId::new(1, 2),
                state: crate::game::OilBarrelState::Unlit,
            });
        })
        .unwrap();
    let teleporter = original
        .clone()
        .map_parts(|parts| {
            parts.pickups.push(crate::cpu::ObservedPickup {
                cell: CellId::new(1, 2),
                kind: crate::game::PickupKind::Teleporter,
            });
        })
        .unwrap();
    let local_pickup = original
        .clone()
        .map_parts(|parts| {
            parts.pickups.push(crate::cpu::ObservedPickup {
                cell: CellId::new(8, 7),
                kind: crate::game::PickupKind::BombCapacity,
            });
        })
        .unwrap();
    let bomb = original
        .map_parts(|parts| {
            let ObservedActor::Visible(owner) = &mut parts.actors[0] else {
                unreachable!()
            };
            owner.live_bomb_count = 1;
            parts.bombs = BombKnowledge::Visible(vec![crate::cpu::ObservedBomb {
                id: crate::game::EntityId(10),
                placed_at: parts.tick,
                cell: CellId::new(1, 2),
                owner: owner.id,
                range: 3,
                trigger: crate::cpu::ObservedBombTrigger::Timed {
                    remaining_ticks: 180,
                    paused: false,
                },
                blast: BlastKind::Standard,
                unknown: crate::cpu::UnknownBombFacts::ALL,
            }]);
        })
        .unwrap();
    for observation in [flame, oil, teleporter, local_pickup, bomb] {
        let mut work = WorkBudget::new(65_536);
        assert!(supported_target(&observation, &rival(origins(), 65_536), &mut work).is_none());
    }
}

#[test]
fn adjacent_pit_and_observed_mob_disable_optional_scoring() {
    let pit = crate::cpu::survival::refuge_centering_test_fixture::captured_scene(SeatId::Three)
        .map_parts(|parts| {
            let ObservedActor::Visible(rival) = &mut parts.actors[0] else {
                unreachable!()
            };
            rival.position = Position::center(CellId::new(5, 4));
            rival.voluntary_actions_frozen = false;
            parts.bombs = BombKnowledge::Visible(Vec::new());
            parts.flames.clear();
            parts.oil.clear();
            parts.pickups.clear();
        })
        .unwrap();
    assert!(
        pit.spike_pits()
            .unwrap()
            .pits()
            .iter()
            .any(|pit| pit.cell == CellId::new(5, 5))
    );
    let mut pit_rival = rival(origins(), 65_536);
    pit_rival.position = Position::center(CellId::new(5, 4));
    assert!(supported_target(&pit, &pit_rival, &mut WorkBudget::new(65_536)).is_none());

    let basis =
        crate::cpu::survival::exit_pressure_fixture::collected_basis_at_with_lairs(START, true);
    let environment = basis.observation.environment().unwrap().clone();
    let old_lairs = basis.observation.respawn_lairs().unwrap();
    let config = old_lairs.config();
    let mut lairs = old_lairs.lairs().to_vec();
    lairs[0].living_mobs = 1;
    lairs[0].next_spawn_at = None;
    let mobs = vec![crate::game::MobView {
        id: crate::game::EntityId(10),
        lair: 0,
        position: Position::center(CellId::new(8, 7)),
        heading: Direction::West,
        frozen: false,
    }];
    let observed = crate::cpu::Observation::new_rules_v10(
        basis.observation.into_parts(),
        environment,
        Some(crate::cpu::ObservedRespawnLairs::new(config, lairs, mobs)),
        None,
    )
    .unwrap();
    assert!(
        supported_target(
            &observed,
            &rival(origins(), 65_536),
            &mut WorkBudget::new(65_536)
        )
        .is_none()
    );
}
