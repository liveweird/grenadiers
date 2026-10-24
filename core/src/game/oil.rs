use std::collections::BTreeMap;
use std::fmt;

use super::PickupKind;
use super::types::CellId;

pub const MAX_OIL_BARRELS: usize = 8;

pub(crate) fn generated_pair_fits(
    concealed: &BTreeMap<CellId, PickupKind>,
    cell: CellId,
    mirror: CellId,
) -> bool {
    let existing = concealed
        .values()
        .filter(|kind| **kind == PickupKind::OilBarrel)
        .count();
    existing + usize::from(cell != mirror) < MAX_OIL_BARRELS
}

pub(crate) fn ordinary_pickup_count(concealed: &BTreeMap<CellId, PickupKind>) -> usize {
    concealed
        .values()
        .filter(|kind| **kind != PickupKind::OilBarrel)
        .count()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OilState {
    Concealed,
    Pending,
    Unlit,
    Burning,
}

#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct OilBarrels(BTreeMap<CellId, OilState>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OilBarrelState {
    Unlit,
    Burning,
}

impl fmt::Debug for OilBarrels {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OilBarrels")
            .field("sites", &"<redacted>")
            .finish()
    }
}

impl OilBarrels {
    pub(crate) fn new(sites: BTreeMap<CellId, OilState>) -> Option<Self> {
        (sites.len() <= MAX_OIL_BARRELS).then_some(Self(sites))
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (CellId, OilState)> + '_ {
        self.0.iter().map(|(&cell, &state)| (cell, state))
    }

    pub(crate) fn state(&self, cell: CellId) -> Option<OilState> {
        self.0.get(&cell).copied()
    }

    pub(crate) fn set_state(&mut self, cell: CellId, state: OilState) {
        *self
            .0
            .get_mut(&cell)
            .expect("oil transitions retain their site") = state;
    }

    pub(crate) fn unlit(&self, cell: CellId) -> bool {
        self.state(cell) == Some(OilState::Unlit)
    }

    pub(crate) fn burning(&self, cell: CellId) -> bool {
        self.state(cell) == Some(OilState::Burning)
    }
}
