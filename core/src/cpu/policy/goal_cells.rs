use crate::cpu::Observation;
use crate::game::{CellId, Terrain};

pub(in crate::cpu::policy) fn legal_opening(observation: &Observation, cell: CellId) -> bool {
    let Some(index) = cell_index(observation, cell) else {
        return false;
    };
    observation.parts().arena.terrain.get(index) == Some(&Terrain::Floor)
        && !observation.parts().arena.blocks.contains(&cell)
        && !observation.parts().oil.iter().any(|oil| oil.cell == cell)
}

pub(in crate::cpu::policy) fn cell_index(observation: &Observation, cell: CellId) -> Option<usize> {
    let arena = &observation.parts().arena;
    (cell.column < arena.width && cell.row < arena.height)
        .then_some(usize::from(cell.row) * usize::from(arena.width) + usize::from(cell.column))
}
