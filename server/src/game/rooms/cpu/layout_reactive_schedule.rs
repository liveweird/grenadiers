use grenadiers_core::game::{ArenaConstructionProfile, Direction, SeatId, Tick};

use crate::game::rooms::roster_runtime::{RosterAction, RosterRuntime};

pub(super) fn apply(runtime: &mut RosterRuntime, profile: ArenaConstructionProfile) {
    if profile != ArenaConstructionProfile::WeightedLairClearanceV7 {
        return;
    }
    let action = match runtime.lifecycle().tick() {
        Tick(180) => {
            runtime
                .action(SeatId::One, 1, RosterAction::Bomb(1))
                .unwrap();
            RosterAction::Movement(Some(Direction::East))
        }
        Tick(200) => RosterAction::Movement(None),
        Tick(390) => RosterAction::Bomb(2),
        Tick(600) => RosterAction::Bomb(3),
        Tick(810) => RosterAction::Bomb(4),
        _ => return,
    };
    runtime.action(SeatId::One, 1, action).unwrap();
}
