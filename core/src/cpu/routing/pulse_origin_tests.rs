use super::*;
use crate::game::{MovementSegment, Position};

#[test]
fn origin_grace_is_consumed_on_exit_and_cannot_be_reentered() {
    let origin = PulseOriginGrace {
        cell: CellId::new(2, 1),
        bomb_id: EntityId(7),
    };
    let inside = Position::center(origin.cell);
    let outside = Position::center(CellId::new(3, 1));
    let mut active = true;
    update_origin(
        Some(origin),
        &[MovementSegment::new(inside, outside)],
        &mut active,
    )
    .unwrap();
    assert!(!active);
    assert_eq!(
        update_origin(
            Some(origin),
            &[MovementSegment::new(outside, inside)],
            &mut active,
        ),
        Err(RouteFailure::NoRoute)
    );
}
