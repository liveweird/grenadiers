use grenadiers_core::{
    cpu::Observation,
    game::{SeatId, Tick},
    room::CpuProfile,
};

use super::hard_opening_owner_support::{Outcome, run};

#[test]
fn generated_seed53_hard70_accepts_and_completes_a_productive_opening_before_hard69() {
    let hard69 = run(CpuProfile::HARD_V69);
    let hard70 = run(CpuProfile::HARD_V70);
    eprintln!("HARD70_OPENING hard69={hard69:#?} hard70={hard70:#?}");

    assert_eq!(hard69.initial, hard70.initial);
    for (old, new) in hard69.first_bases.iter().zip(&hard70.first_bases) {
        assert_eq!(old.as_ref().unwrap().parts().tick, Tick(183));
        assert_same_public_content(old.as_ref().unwrap(), new.as_ref().unwrap());
    }
    let old = hard69
        .first_priority_bomb
        .expect("Hard69 priority CPU places its control bomb");
    let new = hard70
        .productive
        .expect("Hard70 generated opening is productive");
    assert_eq!((old.seat, new.bomb.seat), (SeatId::Four, SeatId::Four));
    assert!(
        hard70
            .accepted_bombs
            .contains(&(new.bomb.accepted_running_tick, new.bomb.seat))
    );
    assert!(hard70.placed_bombs > 0);
    assert_eq!(old.accepted_running_tick, 199, "{hard69:#?}");
    assert_eq!(old.placed_at, Tick(379), "{hard69:#?}");
    assert_eq!(old.placed_at.0 - 180, old.accepted_running_tick);
    assert!(
        new.bomb.accepted_running_tick < old.accepted_running_tick,
        "{hard70:#?}"
    );
    assert!(new.bomb.placed_at < old.placed_at, "{hard70:#?}");
    assert_eq!(new.bomb.placed_at.0 - 180, new.bomb.accepted_running_tick);
    assert!(new.blocks_destroyed > 0);
    assert!(
        hard70
            .initial
            .world()
            .arena()
            .blocks()
            .contains(&new.destroyed_block)
    );
    assert_eq!(new.destroyed_at, new.bomb.detonates_at);
    assert!(hard70.survived_window, "{hard70:#?}");
    assert!(hard70.completed_escape, "{hard70:#?}");
    assert!(hard70.movement_after_window, "{hard70:#?}");
    assert_accounted(&hard69);
    assert_accounted(&hard70);
    assert_eq!(hard69.rng_draws, hard70.rng_draws);
    assert_eq!(hard70.rng_draws, [0; 3]);
}

fn assert_same_public_content(old: &Observation, new: &Observation) {
    let old_parts = old.parts();
    let new_parts = new.parts();
    assert_eq!(
        old_parts.world.physical_round,
        new_parts.world.physical_round
    );
    assert_ne!(old_parts.world.room_instance, new_parts.world.room_instance);
    assert_eq!(old_parts.tick, new_parts.tick);
    assert_eq!(old_parts.recipient, new_parts.recipient);
    assert_eq!(old_parts.rules, new_parts.rules);
    assert_eq!(old_parts.arena, new_parts.arena);
    assert_eq!(old_parts.actors, new_parts.actors);
    assert_eq!(old_parts.bombs, new_parts.bombs);
    assert_eq!(old_parts.flames, new_parts.flames);
    assert_eq!(old_parts.oil, new_parts.oil);
    assert_eq!(old_parts.pickups, new_parts.pickups);
    assert_eq!(old.environment(), new.environment());
    assert_eq!(old.lairs(), new.lairs());
    assert_eq!(old.respawn_lairs(), new.respawn_lairs());
    assert_eq!(old.spike_pits(), new.spike_pits());
}

fn assert_accounted(outcome: &Outcome) {
    let ledger = &outcome.ledger;
    assert!(ledger.delivered > 0, "{outcome:#?}");
    assert_eq!((ledger.failed, ledger.dropped), (0, 0), "{outcome:#?}");
    assert!(ledger.cancelled <= 3, "{outcome:#?}");
    assert_eq!(
        ledger.dequeued,
        ledger.delivered + ledger.failed + ledger.cancelled + ledger.dropped,
        "{outcome:#?}"
    );
    assert!(outcome.queue_drained, "{outcome:#?}");
    assert!(ledger.total_work > 0, "{outcome:#?}");
    assert!(ledger.max_work > 0, "{outcome:#?}");
    assert!(
        ledger.max_work <= grenadiers_core::cpu::survival::WORK_ALLOWANCE,
        "{outcome:#?}"
    );
    assert!(
        outcome
            .admissions
            .iter()
            .all(|trace| trace == &Default::default()),
        "{outcome:#?}"
    );
}
