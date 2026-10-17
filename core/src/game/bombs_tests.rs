use super::{Bomb, BombError, BombTrigger};
use crate::game::events::{StepEvent, StepEventKind};
use crate::game::types::{CellId, EntityId, Tick};
use crate::game::view::BombView;

fn bomb(placed_at: Tick, trigger: BombTrigger) -> Result<Bomb, BombError> {
    Bomb::new(
        EntityId(1),
        CellId::new(2, 3),
        EntityId(10),
        2,
        placed_at,
        trigger,
    )
}

#[test]
fn timed_trigger_accepts_equal_or_later_deadlines_and_rejects_earlier_ones() {
    for detonates_at in [Tick(5), Tick(6)] {
        let bomb = bomb(Tick(5), BombTrigger::Timed { detonates_at }).unwrap();
        assert_eq!(bomb.trigger(), BombTrigger::Timed { detonates_at });
        assert_eq!(bomb.detonates_at(), Some(detonates_at));
    }

    assert_eq!(
        bomb(
            Tick(5),
            BombTrigger::Timed {
                detonates_at: Tick(4),
            },
        ),
        Err(BombError::DeadlineBeforePlacement)
    );
}

#[test]
fn untimed_triggers_never_expose_a_deadline() {
    for trigger in [BombTrigger::Remote, BombTrigger::Contact] {
        let bomb = bomb(Tick(u64::MAX), trigger).unwrap();
        assert_eq!(bomb.trigger(), trigger);
        assert_eq!(bomb.detonates_at(), None);
    }
}

#[test]
fn domain_views_and_events_retain_each_untimed_trigger() {
    for trigger in [BombTrigger::Remote, BombTrigger::Contact] {
        let view = BombView::from(bomb(Tick(5), trigger).unwrap());
        assert_eq!(view.trigger, trigger);

        for kind in [
            StepEventKind::BombPlaced(view),
            StepEventKind::BombDetonated(view),
        ] {
            let event = StepEvent::new(Tick(6), kind);
            assert!(matches!(
                event.kind,
                StepEventKind::BombPlaced(bomb) | StepEventKind::BombDetonated(bomb)
                    if bomb.trigger == trigger
            ));
        }
    }
}
