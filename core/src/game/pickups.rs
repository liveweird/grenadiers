use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::arena::{Arena, Terrain};
use super::types::{CellId, Tick};

/// A permanent upgrade type carried by a pickup.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PickupKind {
    BombCapacity,
    BlastRange,
    MovementSpeed,
    Heart,
    Shield,
    GoldenFlame,
    ShortFuse,
    Diarrhea,
    SlowFoot,
    ShortFlame,
    LongFuse,
    ReverseControls,
}

impl PickupKind {
    pub(crate) const fn enabled_by(self, enabled: super::EnabledItems) -> bool {
        match self {
            Self::Heart => enabled.contains_heart(),
            Self::Shield => enabled.contains_shield(),
            Self::GoldenFlame => enabled.contains_golden_flame(),
            Self::ShortFuse => enabled.contains_short_fuse(),
            Self::Diarrhea => enabled.contains_diarrhea(),
            Self::SlowFoot => enabled.contains_slow_foot(),
            Self::ShortFlame => enabled.contains_short_flame(),
            Self::LongFuse => enabled.contains_long_fuse(),
            Self::ReverseControls => enabled.contains_reverse_controls(),
            _ => true,
        }
    }

    pub(crate) const fn timed_effect(self) -> Option<super::EffectKind> {
        match self {
            Self::Shield => Some(super::EffectKind::Shield),
            Self::GoldenFlame => Some(super::EffectKind::GoldenFlame),
            Self::ShortFuse => Some(super::EffectKind::ShortFuse),
            Self::Diarrhea => Some(super::EffectKind::Diarrhea),
            Self::SlowFoot => Some(super::EffectKind::SlowFoot),
            Self::ShortFlame => Some(super::EffectKind::ShortFlame),
            Self::LongFuse => Some(super::EffectKind::LongFuse),
            Self::ReverseControls => Some(super::EffectKind::ReverseControls),
            _ => None,
        }
    }
}

/// A typed pickup component stored in one spatial layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pickup {
    kind: PickupKind,
}

impl Pickup {
    #[must_use]
    pub const fn new(kind: PickupKind) -> Self {
        Self { kind }
    }

    #[must_use]
    pub const fn kind(self) -> PickupKind {
        self.kind
    }
}

/// The mutually exclusive visibility phase of a pickup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickupLayer {
    Concealed,
    Pending,
    Exposed,
}

/// A violation of the arena and pickup-layer placement invariants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickupPlacementError {
    DuplicateCell(CellId),
    NotOnFloor { cell: CellId, layer: PickupLayer },
    ConcealedWithoutBlock(CellId),
    RetainedBlock { cell: CellId, layer: PickupLayer },
}

#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct PickupLayers {
    pub(crate) concealed: BTreeMap<CellId, Pickup>,
    pub(crate) pending: BTreeMap<CellId, Pickup>,
    pub(crate) exposed: BTreeMap<CellId, Pickup>,
}

impl fmt::Debug for PickupLayers {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PickupLayers")
            .field("concealed_count", &self.concealed.len())
            .field("pending_count", &self.pending.len())
            .field("exposed", &self.exposed)
            .finish()
    }
}

impl PickupLayers {
    pub(crate) fn iter(&self) -> impl Iterator<Item = (CellId, PickupKind)> + '_ {
        self.concealed
            .iter()
            .chain(&self.pending)
            .chain(&self.exposed)
            .map(|(&cell, pickup)| (cell, pickup.kind()))
    }

    pub(crate) fn try_new(
        arena: &Arena,
        concealed: BTreeMap<CellId, PickupKind>,
        pending: BTreeMap<CellId, PickupKind>,
        exposed: BTreeMap<CellId, PickupKind>,
    ) -> Result<Self, PickupPlacementError> {
        let mut occupied = BTreeSet::new();
        for map in [&concealed, &pending, &exposed] {
            for &cell in map.keys() {
                if !occupied.insert(cell) {
                    return Err(PickupPlacementError::DuplicateCell(cell));
                }
            }
        }
        validate_layer(arena, &concealed, PickupLayer::Concealed)?;
        validate_layer(arena, &pending, PickupLayer::Pending)?;
        validate_layer(arena, &exposed, PickupLayer::Exposed)?;
        Ok(Self {
            concealed: into_components(concealed),
            pending: into_components(pending),
            exposed: into_components(exposed),
        })
    }

    pub(crate) fn expose_flame_free(&mut self, hazards: &BTreeMap<CellId, Tick>) {
        let ready: Vec<_> = self
            .pending
            .keys()
            .filter(|cell| !hazards.contains_key(cell))
            .copied()
            .collect();
        for cell in ready {
            let pickup = self
                .pending
                .remove(&cell)
                .expect("ready pickups came from the pending layer");
            let previous = self.exposed.insert(cell, pickup);
            debug_assert!(previous.is_none(), "pickup layers must remain exclusive");
        }
    }

    pub(crate) fn apply_explosion(
        &mut self,
        destroyed_blocks: &BTreeSet<CellId>,
        flames: &BTreeSet<CellId>,
    ) {
        self.exposed.retain(|cell, _| !flames.contains(cell));
        for cell in destroyed_blocks {
            if let Some(pickup) = self.concealed.remove(cell) {
                let previous = self.pending.insert(*cell, pickup);
                debug_assert!(previous.is_none(), "pickup layers must remain exclusive");
            }
        }
    }
}

fn validate_layer(
    arena: &Arena,
    pickups: &BTreeMap<CellId, PickupKind>,
    layer: PickupLayer,
) -> Result<(), PickupPlacementError> {
    for &cell in pickups.keys() {
        if arena.terrain(cell) != Some(Terrain::Floor) {
            return Err(PickupPlacementError::NotOnFloor { cell, layer });
        }
        match layer {
            PickupLayer::Concealed if !arena.blocks().contains(&cell) => {
                return Err(PickupPlacementError::ConcealedWithoutBlock(cell));
            }
            PickupLayer::Pending | PickupLayer::Exposed if arena.blocks().contains(&cell) => {
                return Err(PickupPlacementError::RetainedBlock { cell, layer });
            }
            _ => {}
        }
    }
    Ok(())
}

fn into_components(pickups: BTreeMap<CellId, PickupKind>) -> BTreeMap<CellId, Pickup> {
    pickups
        .into_iter()
        .map(|(cell, kind)| (cell, Pickup::new(kind)))
        .collect()
}
