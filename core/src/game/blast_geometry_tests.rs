use std::collections::{BTreeMap, BTreeSet};
use std::ops::ControlFlow;

use super::{BlastBomb, BlastField, BlastSource, BlastVisit, visit_blast};
use crate::game::{BlastKind, CellId, EntityId};

#[derive(Default)]
struct Field {
    walls: BTreeSet<CellId>,
    blocks: BTreeSet<CellId>,
    oil: BTreeSet<CellId>,
    bombs: BTreeMap<CellId, BlastBomb>,
}

impl BlastField for Field {
    fn is_floor(&self, cell: CellId) -> bool {
        cell.column < 15 && cell.row < 13 && !self.walls.contains(&cell)
    }

    fn has_soft_block(&self, cell: CellId) -> bool {
        self.blocks.contains(&cell)
    }

    fn has_unlit_oil(&self, cell: CellId) -> bool {
        self.oil.contains(&cell)
    }

    fn bomb_at(&self, cell: CellId) -> Option<BlastBomb> {
        self.bombs.get(&cell).copied()
    }
}

fn collect(field: &Field, kind: BlastKind, range: u16) -> Vec<BlastVisit> {
    let mut visits = Vec::new();
    let result = visit_blast(
        field,
        BlastSource {
            cell: CellId::new(7, 6),
            range,
            kind,
        },
        |visit| {
            visits.push(visit);
            ControlFlow::<()>::Continue(())
        },
    );
    assert!(result.is_continue());
    visits
}

#[test]
fn standard_layer_priority_stops_before_a_bomb_hidden_by_soft_occupancy() {
    let occupied = CellId::new(8, 6);
    let field = Field {
        blocks: BTreeSet::from([occupied]),
        bombs: BTreeMap::from([(
            occupied,
            BlastBomb {
                id: EntityId(9),
                contact: false,
            },
        )]),
        ..Field::default()
    };

    let visits = collect(&field, BlastKind::Standard, 3);
    let occupied_visit = visits.iter().find(|visit| visit.cell == occupied).unwrap();
    assert!(occupied_visit.soft_block);
    assert_eq!(occupied_visit.bomb, None);
    assert!(!visits.iter().any(|visit| visit.cell == CellId::new(9, 6)));
}

#[test]
fn contact_mines_do_not_stop_rays_but_regular_bombs_do() {
    let contact = CellId::new(8, 6);
    let regular = CellId::new(9, 6);
    let field = Field {
        bombs: BTreeMap::from([
            (
                contact,
                BlastBomb {
                    id: EntityId(2),
                    contact: true,
                },
            ),
            (
                regular,
                BlastBomb {
                    id: EntityId(3),
                    contact: false,
                },
            ),
        ]),
        ..Field::default()
    };

    let visits = collect(&field, BlastKind::Piercing, 4);
    assert!(visits.iter().any(|visit| visit.cell == contact));
    assert!(visits.iter().any(|visit| visit.cell == regular));
    assert!(!visits.iter().any(|visit| visit.cell == CellId::new(10, 6)));
}

#[test]
fn visitor_can_end_geometry_work_without_hidden_follow_on_visits() {
    let mut count = 0;
    let result = visit_blast(
        &Field::default(),
        BlastSource {
            cell: CellId::new(7, 6),
            range: 8,
            kind: BlastKind::Power,
        },
        |_| {
            count += 1;
            if count == 3 {
                ControlFlow::Break("exhausted")
            } else {
                ControlFlow::Continue(())
            }
        },
    );

    assert_eq!(result, ControlFlow::Break("exhausted"));
    assert_eq!(count, 3);
}
